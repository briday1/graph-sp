//! Execution-result caching for DAG nodes.
//!
//! Cache keys are content-addressed and intentionally include:
//! - stable node identity metadata
//! - a node code/version fingerprint
//! - a normalized representation of the node input payload
//! - an upstream dependency signature when transitive reuse is enabled
//!
//! The normalization helpers sort map keys and encode numeric values in a stable
//! bit-pattern-oriented format so equivalent payloads hash to the same cache key.

use crate::graph_data::GraphData;
use crate::node::Node;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Configures how aggressively node results may be reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheDepth {
    /// Disable cache reuse entirely for this run.
    None,
    /// Reuse direct node-level hits keyed by node identity, code fingerprint, and inputs.
    Shallow,
    /// Reuse node results only when upstream dependency signatures also match.
    Transitive,
}

impl Default for CacheDepth {
    fn default() -> Self {
        Self::Transitive
    }
}

impl CacheDepth {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Shallow => "shallow",
            Self::Transitive => "transitive",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "shallow" => Some(Self::Shallow),
            "transitive" => Some(Self::Transitive),
            _ => None,
        }
    }
}

/// Per-run cache controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheOptions {
    pub enabled: bool,
    pub depth: CacheDepth,
    pub namespace: String,
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            depth: CacheDepth::Transitive,
            namespace: "default".to_string(),
        }
    }
}

impl CacheOptions {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }

    pub fn with_depth(mut self, depth: CacheDepth) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Miss reasons surfaced by cache lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheMissReason {
    Disabled,
    NonCacheable,
    NotFound,
    Expired,
    CodeChanged,
    InputChanged,
    DependencyChanged,
    UnsupportedInput,
}

impl fmt::Display for CacheMissReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled => write!(f, "disabled"),
            Self::NonCacheable => write!(f, "non_cacheable"),
            Self::NotFound => write!(f, "not_found"),
            Self::Expired => write!(f, "expired"),
            Self::CodeChanged => write!(f, "code_changed"),
            Self::InputChanged => write!(f, "input_changed"),
            Self::DependencyChanged => write!(f, "dependency_changed"),
            Self::UnsupportedInput => write!(f, "unsupported_input"),
        }
    }
}

/// Per-run cache observability.
#[derive(Debug, Clone, Default)]
pub struct CacheRunStats {
    pub hits: usize,
    pub misses: usize,
    pub stores: usize,
    pub reason_counts: HashMap<CacheMissReason, usize>,
}

impl CacheRunStats {
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    pub fn record_miss(&mut self, reason: CacheMissReason) {
        self.misses += 1;
        *self.reason_counts.entry(reason).or_insert(0) += 1;
    }

    pub fn record_store(&mut self) {
        self.stores += 1;
    }

    pub fn summary(&self) -> String {
        let mut reasons: Vec<_> = self.reason_counts.iter().collect();
        reasons.sort_by_key(|(reason, _)| reason.to_string());
        let reason_summary = reasons
            .into_iter()
            .map(|(reason, count)| format!("{reason}={count}"))
            .collect::<Vec<_>>()
            .join(", ");
        if reason_summary.is_empty() {
            format!(
                "cache hits={}, misses={}, stores={}",
                self.hits, self.misses, self.stores
            )
        } else {
            format!(
                "cache hits={}, misses={}, stores={}, reasons=[{}]",
                self.hits, self.misses, self.stores, reason_summary
            )
        }
    }
}

/// Snapshot of backend-level storage statistics.
#[derive(Debug, Clone, Default)]
pub struct CacheBackendStats {
    pub entries: usize,
    pub max_entries: usize,
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub expirations: usize,
}

/// Configures the default in-memory cache backend.
#[derive(Debug, Clone)]
pub struct MemoryCacheConfig {
    pub max_entries: usize,
    pub ttl: Option<Duration>,
}

impl Default for MemoryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1_024,
            ttl: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheLookup {
    pub key: String,
    pub node_identity: String,
    pub code_fingerprint: String,
    pub input_fingerprint: String,
    pub dependency_signature: String,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub outputs: HashMap<String, GraphData>,
    pub node_identity: String,
    pub code_fingerprint: String,
    pub input_fingerprint: String,
    pub dependency_signature: String,
}

#[derive(Debug, Clone)]
pub enum CacheLookupResult {
    Hit(HashMap<String, GraphData>),
    Miss(CacheMissReason),
}

/// Pluggable cache backend interface.
pub trait CacheBackend: Send + Sync {
    fn get(
        &self,
        namespace: &str,
        lookup: &CacheLookup,
    ) -> CacheLookupResult;

    fn put(&self, namespace: &str, key: String, entry: CacheEntry);

    fn stats(&self) -> CacheBackendStats;

    fn clear_all(&self);

    fn clear_namespace(&self, namespace: &str);
}

/// LRU/TTL in-memory backend.
#[derive(Debug, Clone)]
pub struct MemoryCacheBackend {
    inner: Arc<Mutex<MemoryCacheState>>,
}

#[derive(Debug)]
struct MemoryCacheState {
    config: MemoryCacheConfig,
    entries: HashMap<String, StoredEntry>,
    identity_index: HashMap<(String, String), IdentityFingerprint>,
    lru: VecDeque<(u64, String)>,
    access_clock: u64,
    stats: CacheBackendStats,
}

#[derive(Debug, Clone)]
struct StoredEntry {
    entry: CacheEntry,
    access_token: u64,
    expires_at: Option<Instant>,
}

#[derive(Debug, Clone)]
struct IdentityFingerprint {
    storage_key: String,
    code_fingerprint: String,
    input_fingerprint: String,
    dependency_signature: String,
}

impl MemoryCacheBackend {
    pub fn new(config: MemoryCacheConfig) -> Self {
        let max_entries = config.max_entries.max(1);
        Self {
            inner: Arc::new(Mutex::new(MemoryCacheState {
                stats: CacheBackendStats {
                    max_entries,
                    ..CacheBackendStats::default()
                },
                config: MemoryCacheConfig {
                    max_entries,
                    ttl: config.ttl,
                },
                entries: HashMap::new(),
                identity_index: HashMap::new(),
                lru: VecDeque::new(),
                access_clock: 0,
            })),
        }
    }

    fn storage_key(namespace: &str, key: &str) -> String {
        format!("{namespace}::{key}")
    }
}

impl MemoryCacheState {
    fn next_access_token(&mut self) -> u64 {
        self.access_clock = self.access_clock.saturating_add(1);
        self.access_clock
    }

    fn remove_entry(&mut self, storage_key: &str) {
        if let Some(stored) = self.entries.remove(storage_key) {
            let index_key = (
                storage_key
                    .splitn(2, "::")
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                stored.entry.node_identity.clone(),
            );
            if let Some(indexed) = self.identity_index.get(&index_key) {
                if indexed.storage_key == storage_key {
                    self.identity_index.remove(&index_key);
                }
            }
        }
    }

    fn evict_if_needed(&mut self) {
        while self.entries.len() > self.config.max_entries {
            let Some((token, storage_key)) = self.lru.pop_front() else {
                break;
            };
            let should_remove = self
                .entries
                .get(&storage_key)
                .map(|entry| entry.access_token == token)
                .unwrap_or(false);
            if should_remove {
                self.remove_entry(&storage_key);
                self.stats.evictions += 1;
            }
        }
        self.stats.entries = self.entries.len();
    }
}

impl CacheBackend for MemoryCacheBackend {
    fn get(
        &self,
        namespace: &str,
        lookup: &CacheLookup,
    ) -> CacheLookupResult {
        let storage_key = Self::storage_key(namespace, &lookup.key);
        let mut state = self.inner.lock().unwrap();

        let expired = state
            .entries
            .get(&storage_key)
            .and_then(|entry| entry.expires_at)
            .map(|deadline| Instant::now() >= deadline)
            .unwrap_or(false);

        if expired {
            state.remove_entry(&storage_key);
            state.stats.expirations += 1;
            state.stats.misses += 1;
            state.stats.entries = state.entries.len();
            return CacheLookupResult::Miss(CacheMissReason::Expired);
        }

        let next_token = state.next_access_token();
        if let Some(stored) = state.entries.get_mut(&storage_key) {
            stored.access_token = next_token;
            let outputs = stored.entry.outputs.clone();
            let _ = stored;
            state.lru.push_back((next_token, storage_key));
            state.stats.hits += 1;
            return CacheLookupResult::Hit(outputs);
        }

        state.stats.misses += 1;
        let identity_key = (namespace.to_string(), lookup.node_identity.clone());
        let reason = match state.identity_index.get(&identity_key) {
            Some(identity) if identity.storage_key != storage_key => {
                if identity.code_fingerprint != lookup.code_fingerprint {
                    CacheMissReason::CodeChanged
                } else if identity.input_fingerprint != lookup.input_fingerprint {
                    CacheMissReason::InputChanged
                } else if identity.dependency_signature != lookup.dependency_signature {
                    CacheMissReason::DependencyChanged
                } else {
                    CacheMissReason::NotFound
                }
            }
            Some(identity) if !state.entries.contains_key(&identity.storage_key) => {
                state.identity_index.remove(&identity_key);
                CacheMissReason::NotFound
            }
            _ => CacheMissReason::NotFound,
        };

        CacheLookupResult::Miss(reason)
    }

    fn put(&self, namespace: &str, key: String, entry: CacheEntry) {
        let storage_key = Self::storage_key(namespace, &key);
        let mut state = self.inner.lock().unwrap();
        let token = state.next_access_token();
        let expires_at = state.config.ttl.map(|ttl| Instant::now() + ttl);
        let identity_key = (namespace.to_string(), entry.node_identity.clone());
        let fingerprint = IdentityFingerprint {
            storage_key: storage_key.clone(),
            code_fingerprint: entry.code_fingerprint.clone(),
            input_fingerprint: entry.input_fingerprint.clone(),
            dependency_signature: entry.dependency_signature.clone(),
        };

        state.entries.insert(
            storage_key.clone(),
            StoredEntry {
                entry,
                access_token: token,
                expires_at,
            },
        );
        state.identity_index.insert(identity_key, fingerprint);
        state.lru.push_back((token, storage_key));
        state.stats.entries = state.entries.len();
        state.evict_if_needed();
    }

    fn stats(&self) -> CacheBackendStats {
        self.inner.lock().unwrap().stats.clone()
    }

    fn clear_all(&self) {
        let mut state = self.inner.lock().unwrap();
        state.entries.clear();
        state.identity_index.clear();
        state.lru.clear();
        state.stats.entries = 0;
    }

    fn clear_namespace(&self, namespace: &str) {
        let mut state = self.inner.lock().unwrap();
        let prefix = format!("{namespace}::");
        let keys: Vec<String> = state
            .entries
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .cloned()
            .collect();
        for key in keys {
            state.remove_entry(&key);
        }
        state.lru.retain(|(_, key)| !key.starts_with(&prefix));
        state.stats.entries = state.entries.len();
    }
}

/// Minimal disk-backed scaffold kept separate from the in-memory implementation.
#[derive(Debug, Clone)]
pub struct FileCacheBackend {
    root: PathBuf,
}

impl FileCacheBackend {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}

impl CacheBackend for FileCacheBackend {
    fn get(
        &self,
        _namespace: &str,
        _lookup: &CacheLookup,
    ) -> CacheLookupResult {
        let _ = &self.root;
        CacheLookupResult::Miss(CacheMissReason::NotFound)
    }

    fn put(&self, _namespace: &str, _key: String, _entry: CacheEntry) {
        let _ = &self.root;
    }

    fn stats(&self) -> CacheBackendStats {
        CacheBackendStats::default()
    }

    fn clear_all(&self) {}

    fn clear_namespace(&self, _namespace: &str) {}
}

pub(crate) fn build_cache_lookup(
    node: &Node,
    normalized_input: &str,
    dependency_signature: &str,
) -> CacheLookup {
    let node_identity = build_node_identity(node);
    let code_fingerprint = hash_canonical(node.code_fingerprint.as_bytes());
    let input_fingerprint = hash_canonical(normalized_input.as_bytes());
    let dependency_signature = dependency_signature.to_string();
    let materialized = format!(
        "node={node_identity}\ncode={code_fingerprint}\ninput={input_fingerprint}\ndeps={dependency_signature}"
    );
    CacheLookup {
        key: hash_canonical(materialized.as_bytes()),
        node_identity,
        code_fingerprint,
        input_fingerprint,
        dependency_signature,
    }
}

pub(crate) fn build_cache_entry(
    lookup: &CacheLookup,
    outputs: HashMap<String, GraphData>,
) -> CacheEntry {
    CacheEntry {
        outputs,
        node_identity: lookup.node_identity.clone(),
        code_fingerprint: lookup.code_fingerprint.clone(),
        input_fingerprint: lookup.input_fingerprint.clone(),
        dependency_signature: lookup.dependency_signature.clone(),
    }
}

pub(crate) fn normalize_inputs(
    inputs: &HashMap<String, GraphData>,
) -> String {
    normalize_named_payload(inputs)
}

pub(crate) fn dependency_signature(
    depth: CacheDepth,
    node: &Node,
    dependency_signatures: &HashMap<usize, String>,
) -> String {
    match depth {
        CacheDepth::None => "cache:none".to_string(),
        CacheDepth::Shallow => "cache:shallow".to_string(),
        CacheDepth::Transitive => {
            let mut pairs: Vec<String> = node
                .dependencies
                .iter()
                .map(|dep| {
                    let signature = dependency_signatures
                        .get(dep)
                        .cloned()
                        .unwrap_or_else(|| "missing".to_string());
                    format!("{dep}:{signature}")
                })
                .collect();
            pairs.sort();
            hash_canonical(pairs.join("|").as_bytes())
        }
    }
}

fn build_node_identity(node: &Node) -> String {
    let label = node.label.clone().unwrap_or_else(|| format!("node-{}", node.id));
    let inputs = normalize_string_map(&node.input_mapping);
    let outputs = normalize_string_map(&node.output_mapping);
    let variant_params = normalize_graph_data_map(&node.variant_params);
    format!(
        "id={};label={};branch={:?};variant={:?};is_branch={};inputs={};outputs={};variant_params={}",
        node.id,
        label,
        node.branch_id,
        node.variant_index,
        node.is_branch,
        inputs,
        outputs,
        variant_params
    )
}

fn normalize_string_map(map: &HashMap<String, String>) -> String {
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
    let mut result = String::from("{");
    for (idx, (key, value)) in entries.into_iter().enumerate() {
        if idx > 0 {
            result.push(',');
        }
        result.push_str(key);
        result.push('=');
        result.push_str(value);
    }
    result.push('}');
    result
}

fn normalize_named_payload(
    payload: &HashMap<String, GraphData>,
) -> String {
    normalize_graph_data_map(payload)
}

fn normalize_graph_data_map(
    payload: &HashMap<String, GraphData>,
) -> String {
    let mut entries: Vec<_> = payload.iter().collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));

    let mut result = String::from("{");
    for (idx, (key, value)) in entries.into_iter().enumerate() {
        if idx > 0 {
            result.push(',');
        }
        result.push_str(key);
        result.push('=');
        result.push_str(&normalize_graph_data(value));
    }
    result.push('}');
    result
}

pub(crate) fn normalize_graph_data(value: &GraphData) -> String {
    match value {
        GraphData::Int(v) => format!("i:{v}"),
        GraphData::Float(v) => format!("f:{:016x}", v.to_bits()),
        GraphData::String(v) => format!("s:{v:?}"),
        GraphData::FloatVec(values) => format!("fv:len={}", values.len()),
        GraphData::IntVec(values) => format!("iv:len={}", values.len()),
        GraphData::Map(values) => format!("m:{}", normalize_graph_data_map(values)),
        GraphData::None => "none".to_string(),
        #[cfg(feature = "radar_examples")]
        GraphData::Complex(value) => format!(
            "c:{:016x}:{:016x}",
            value.re.to_bits(),
            value.im.to_bits()
        ),
        #[cfg(feature = "radar_examples")]
        GraphData::FloatArray(values) => format!("fa:len={}", values.len()),
        #[cfg(feature = "radar_examples")]
        GraphData::ComplexArray(values) => format!("ca:len={}", values.len()),
        #[cfg(feature = "python")]
        GraphData::PyObject(_) => "py:ref".to_string(),
    }
}

pub(crate) fn hash_canonical(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn sample_outputs(value: i64) -> HashMap<String, GraphData> {
        HashMap::from([("value".to_string(), GraphData::int(value))])
    }

    #[test]
    fn normalization_is_stable_for_maps() {
        let left = HashMap::from([
            ("b".to_string(), GraphData::int(2)),
            ("a".to_string(), GraphData::float(1.5)),
        ]);
        let right = HashMap::from([
            ("a".to_string(), GraphData::float(1.5)),
            ("b".to_string(), GraphData::int(2)),
        ]);

        assert_eq!(normalize_inputs(&left), normalize_inputs(&right));
    }

    #[test]
    fn namespace_isolated_entries_do_not_collide() {
        let backend = MemoryCacheBackend::new(MemoryCacheConfig::default());
        let lookup = CacheLookup {
            key: "key".to_string(),
            node_identity: "node".to_string(),
            code_fingerprint: "code".to_string(),
            input_fingerprint: "input".to_string(),
            dependency_signature: "dep".to_string(),
        };
        backend.put("alpha", lookup.key.clone(), build_cache_entry(&lookup, sample_outputs(1)));

        assert!(matches!(
            backend.get("beta", &lookup),
            CacheLookupResult::Miss(CacheMissReason::NotFound)
        ));
        assert!(matches!(backend.get("alpha", &lookup), CacheLookupResult::Hit(_)));
    }

    #[test]
    fn ttl_expires_entries() {
        let backend = MemoryCacheBackend::new(MemoryCacheConfig {
            max_entries: 8,
            ttl: Some(Duration::from_millis(5)),
        });
        let lookup = CacheLookup {
            key: "key".to_string(),
            node_identity: "node".to_string(),
            code_fingerprint: "code".to_string(),
            input_fingerprint: "input".to_string(),
            dependency_signature: "dep".to_string(),
        };
        backend.put("ns", lookup.key.clone(), build_cache_entry(&lookup, sample_outputs(1)));
        thread::sleep(Duration::from_millis(10));

        assert!(matches!(
            backend.get("ns", &lookup),
            CacheLookupResult::Miss(CacheMissReason::Expired)
        ));
    }

    #[test]
    fn lru_eviction_removes_oldest_entry() {
        let backend = MemoryCacheBackend::new(MemoryCacheConfig {
            max_entries: 2,
            ttl: None,
        });

        let first = CacheLookup {
            key: "first".to_string(),
            node_identity: "node-a".to_string(),
            code_fingerprint: "code".to_string(),
            input_fingerprint: "input-1".to_string(),
            dependency_signature: "dep".to_string(),
        };
        let second = CacheLookup {
            key: "second".to_string(),
            node_identity: "node-b".to_string(),
            code_fingerprint: "code".to_string(),
            input_fingerprint: "input-2".to_string(),
            dependency_signature: "dep".to_string(),
        };
        let third = CacheLookup {
            key: "third".to_string(),
            node_identity: "node-c".to_string(),
            code_fingerprint: "code".to_string(),
            input_fingerprint: "input-3".to_string(),
            dependency_signature: "dep".to_string(),
        };

        backend.put("ns", first.key.clone(), build_cache_entry(&first, sample_outputs(1)));
        backend.put("ns", second.key.clone(), build_cache_entry(&second, sample_outputs(2)));
        let _ = backend.get("ns", &second);
        backend.put("ns", third.key.clone(), build_cache_entry(&third, sample_outputs(3)));

        assert!(matches!(
            backend.get("ns", &first),
            CacheLookupResult::Miss(CacheMissReason::NotFound)
        ));
        assert!(matches!(backend.get("ns", &second), CacheLookupResult::Hit(_)));
        assert!(matches!(backend.get("ns", &third), CacheLookupResult::Hit(_)));
    }
}
