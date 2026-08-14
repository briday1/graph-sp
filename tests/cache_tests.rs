use dagex::{
    CacheBackend, CacheDepth, CacheOptions, Graph, GraphData, MemoryCacheBackend, MemoryCacheConfig,
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

fn output_int(key: &str, value: i64) -> HashMap<String, GraphData> {
    HashMap::from([(key.to_string(), GraphData::int(value))])
}

fn source_one(_: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    output_int("raw", 1)
}

fn source_two(_: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    output_int("raw", 2)
}

#[test]
fn repeated_runs_reuse_cached_outputs() {
    let source_runs = Arc::new(AtomicUsize::new(0));
    let process_runs = Arc::new(AtomicUsize::new(0));

    let mut graph = Graph::new();
    graph.set_cache_version_for("Source", "source-v1");
    graph.set_cache_version_for("Process", "process-v1");

    let source_counter = Arc::clone(&source_runs);
    graph.add(
        move |_| {
            source_counter.fetch_add(1, Ordering::SeqCst);
            output_int("value", 10)
        },
        Some("Source"),
        None,
        Some(vec![("value", "data")]),
    );

    let process_counter = Arc::clone(&process_runs);
    graph.add(
        move |inputs: &HashMap<String, GraphData>| {
            process_counter.fetch_add(1, Ordering::SeqCst);
            let value = inputs
                .get("input")
                .and_then(|data| data.as_int())
                .unwrap_or_default();
            output_int("result", value * 3)
        },
        Some("Process"),
        Some(vec![("data", "input")]),
        Some(vec![("result", "result")]),
    );

    let dag = graph.build();
    let options = CacheOptions::default().with_namespace("repeat-runs");

    let first = dag.execute_detailed_with_options(false, None, options.clone());
    let second = dag.execute_detailed_with_options(false, None, options);

    assert_eq!(first.get("result").and_then(|data| data.as_int()), Some(30));
    assert_eq!(
        second.get("result").and_then(|data| data.as_int()),
        Some(30)
    );
    assert_eq!(source_runs.load(Ordering::SeqCst), 1);
    assert_eq!(process_runs.load(Ordering::SeqCst), 1);
    assert_eq!(first.cache_stats.hits, 0);
    assert_eq!(second.cache_stats.hits, 2);
}

#[test]
fn cache_depth_none_disables_reuse() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut graph = Graph::new();
    graph.set_cache_version_for("Source", "source-v1");
    let counter = Arc::clone(&runs);

    graph.add(
        move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            output_int("value", 5)
        },
        Some("Source"),
        None,
        Some(vec![("value", "value")]),
    );

    let dag = graph.build();
    let options = CacheOptions::default()
        .with_depth(CacheDepth::None)
        .with_namespace("disabled");

    let first = dag.execute_detailed_with_options(false, None, options.clone());
    let second = dag.execute_detailed_with_options(false, None, options);

    assert_eq!(runs.load(Ordering::SeqCst), 2);
    assert_eq!(first.cache_stats.hits, 0);
    assert_eq!(second.cache_stats.hits, 0);
}

#[test]
fn nodes_without_explicit_version_tokens_are_not_cached() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut graph = Graph::new();
    let counter = Arc::clone(&runs);

    graph.add(
        move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            output_int("value", 5)
        },
        Some("Versionless"),
        None,
        Some(vec![("value", "value")]),
    );

    let dag = graph.build();
    let options = CacheOptions::default().with_namespace("versionless");

    let first = dag.execute_detailed_with_options(false, None, options.clone());
    let second = dag.execute_detailed_with_options(false, None, options);

    assert_eq!(runs.load(Ordering::SeqCst), 2);
    assert_eq!(
        first
            .node_cache_status
            .values()
            .next()
            .and_then(|status| status.reason),
        Some(dagex::CacheMissReason::MissingVersion)
    );
    assert_eq!(
        second
            .node_cache_status
            .values()
            .next()
            .and_then(|status| status.reason),
        Some(dagex::CacheMissReason::MissingVersion)
    );
}

#[test]
fn clear_cache_namespace_forces_reexecution() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut graph = Graph::new();
    graph.set_cache_version_for("Source", "source-v1");
    let counter = Arc::clone(&runs);

    graph.add(
        move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            output_int("value", 11)
        },
        Some("Source"),
        None,
        Some(vec![("value", "value")]),
    );

    let dag = graph.build();
    let options = CacheOptions::default().with_namespace("clear-me");

    let first = dag.execute_detailed_with_options(false, None, options.clone());
    let second = dag.execute_detailed_with_options(false, None, options.clone());
    dag.clear_cache_namespace("clear-me");
    let third = dag.execute_detailed_with_options(false, None, options);

    assert_eq!(first.cache_stats.hits, 0);
    assert_eq!(second.cache_stats.hits, 1);
    assert_eq!(third.cache_stats.hits, 0);
    assert_eq!(runs.load(Ordering::SeqCst), 2);
}

#[test]
fn cache_key_input_subset_allows_revision_based_reuse_for_large_inputs() {
    let consume_runs = Arc::new(AtomicUsize::new(0));
    let mut graph = Graph::new();
    graph.set_cache_version_for("Source", "source-v1");
    graph.set_cache_version_for("Consumer", "consumer-v1");
    graph.set_cache_key_inputs_for("Consumer", vec!["revision"]);

    graph.add(
        |_| {
            let mut outputs = HashMap::new();
            outputs.insert("data".to_string(), GraphData::int_vec(vec![1, 2, 3, 4]));
            outputs.insert("revision".to_string(), GraphData::string("rev-1"));
            outputs
        },
        Some("Source"),
        None,
        Some(vec![("data", "data"), ("revision", "revision")]),
    );

    let consume_counter = Arc::clone(&consume_runs);
    graph.add(
        move |inputs: &HashMap<String, GraphData>| {
            consume_counter.fetch_add(1, Ordering::SeqCst);
            let revision = inputs
                .get("revision")
                .and_then(|data| data.as_string())
                .unwrap_or_default();
            let len = inputs
                .get("data")
                .and_then(|data| data.as_int_vec())
                .map(|values| values.len())
                .unwrap_or_default() as i64;
            output_int("result", len + revision.len() as i64)
        },
        Some("Consumer"),
        Some(vec![("data", "data"), ("revision", "revision")]),
        Some(vec![("result", "result")]),
    );

    let dag = graph.build();
    let options = CacheOptions::default().with_namespace("revision-inputs");

    let first = dag.execute_detailed_with_options(false, None, options.clone());
    let second = dag.execute_detailed_with_options(false, None, options);

    assert_eq!(first.get("result").and_then(|data| data.as_int()), Some(9));
    assert_eq!(second.get("result").and_then(|data| data.as_int()), Some(9));
    assert_eq!(consume_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn non_cacheable_nodes_always_execute() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut graph = Graph::new();
    graph.set_cacheable_for("SideEffect", false);
    let counter = Arc::clone(&runs);

    graph.add(
        move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            output_int("value", 7)
        },
        Some("SideEffect"),
        None,
        Some(vec![("value", "value")]),
    );

    let dag = graph.build();
    let options = CacheOptions::default().with_namespace("non-cacheable");

    let first = dag.execute_detailed_with_options(false, None, options.clone());
    let second = dag.execute_detailed_with_options(false, None, options);

    assert_eq!(runs.load(Ordering::SeqCst), 2);
    assert_eq!(
        first
            .node_cache_status
            .values()
            .next()
            .and_then(|status| status.reason),
        Some(dagex::CacheMissReason::NonCacheable)
    );
    assert_eq!(
        second
            .node_cache_status
            .values()
            .next()
            .and_then(|status| status.reason),
        Some(dagex::CacheMissReason::NonCacheable)
    );
}

fn build_depth_graph(
    backend: Arc<dyn CacheBackend>,
    source_variant: usize,
    normalize_runs: Arc<AtomicUsize>,
    consume_runs: Arc<AtomicUsize>,
) -> dagex::Dag {
    let mut graph = Graph::new();
    graph.with_cache_backend(backend);
    graph.set_cache_version_for("Normalize", "normalize-v1");
    graph.set_cache_version_for("Consume", "consume-v1");

    if source_variant == 1 {
        graph.set_cache_version_for("Source", "source-v1");
        graph.add(source_one, Some("Source"), None, Some(vec![("raw", "raw")]));
    } else {
        graph.set_cache_version_for("Source", "source-v2");
        graph.add(source_two, Some("Source"), None, Some(vec![("raw", "raw")]));
    }

    let normalize_counter = Arc::clone(&normalize_runs);
    graph.add(
        move |inputs: &HashMap<String, GraphData>| {
            normalize_counter.fetch_add(1, Ordering::SeqCst);
            let _ = inputs
                .get("raw")
                .and_then(|data| data.as_int())
                .unwrap_or_default();
            output_int("normalized", 42)
        },
        Some("Normalize"),
        Some(vec![("raw", "raw")]),
        Some(vec![("normalized", "normalized")]),
    );

    let consume_counter = Arc::clone(&consume_runs);
    graph.add(
        move |inputs: &HashMap<String, GraphData>| {
            consume_counter.fetch_add(1, Ordering::SeqCst);
            let value = inputs
                .get("value")
                .and_then(|data| data.as_int())
                .unwrap_or_default();
            output_int("result", value + 1)
        },
        Some("Consume"),
        Some(vec![("normalized", "value")]),
        Some(vec![("result", "result")]),
    );

    graph.build()
}

#[test]
fn shallow_cache_reuses_direct_hits_but_transitive_requires_dependency_match() {
    let backend_shallow: Arc<dyn CacheBackend> =
        Arc::new(MemoryCacheBackend::new(MemoryCacheConfig::default()));
    let backend_transitive: Arc<dyn CacheBackend> =
        Arc::new(MemoryCacheBackend::new(MemoryCacheConfig::default()));

    let shallow_normalize_runs = Arc::new(AtomicUsize::new(0));
    let shallow_consume_runs = Arc::new(AtomicUsize::new(0));
    let shallow_first = build_depth_graph(
        Arc::clone(&backend_shallow),
        1,
        Arc::clone(&shallow_normalize_runs),
        Arc::clone(&shallow_consume_runs),
    );
    let shallow_second = build_depth_graph(
        Arc::clone(&backend_shallow),
        2,
        Arc::clone(&shallow_normalize_runs),
        Arc::clone(&shallow_consume_runs),
    );

    let shallow_options = CacheOptions::default()
        .with_namespace("depth-shallow")
        .with_depth(CacheDepth::Shallow);
    shallow_first.execute_detailed_with_options(false, None, shallow_options.clone());
    shallow_second.execute_detailed_with_options(false, None, shallow_options);

    assert_eq!(shallow_normalize_runs.load(Ordering::SeqCst), 2);
    assert_eq!(shallow_consume_runs.load(Ordering::SeqCst), 1);

    let transitive_normalize_runs = Arc::new(AtomicUsize::new(0));
    let transitive_consume_runs = Arc::new(AtomicUsize::new(0));
    let transitive_first = build_depth_graph(
        Arc::clone(&backend_transitive),
        1,
        Arc::clone(&transitive_normalize_runs),
        Arc::clone(&transitive_consume_runs),
    );
    let transitive_second = build_depth_graph(
        Arc::clone(&backend_transitive),
        2,
        Arc::clone(&transitive_normalize_runs),
        Arc::clone(&transitive_consume_runs),
    );

    let transitive_options = CacheOptions::default()
        .with_namespace("depth-transitive")
        .with_depth(CacheDepth::Transitive);
    transitive_first.execute_detailed_with_options(false, None, transitive_options.clone());
    transitive_second.execute_detailed_with_options(false, None, transitive_options);

    assert_eq!(transitive_normalize_runs.load(Ordering::SeqCst), 2);
    assert_eq!(transitive_consume_runs.load(Ordering::SeqCst), 2);
}

#[test]
fn repeated_variant_sweeps_skip_reexecution() {
    let source_runs = Arc::new(AtomicUsize::new(0));
    let variant_a_runs = Arc::new(AtomicUsize::new(0));
    let variant_b_runs = Arc::new(AtomicUsize::new(0));
    let variant_c_runs = Arc::new(AtomicUsize::new(0));

    let mut graph = Graph::new();
    graph.set_cache_version_for("Source", "source-v1");
    let source_counter = Arc::clone(&source_runs);
    graph.add(
        move |_| {
            source_counter.fetch_add(1, Ordering::SeqCst);
            output_int("seed", 4)
        },
        Some("Source"),
        None,
        Some(vec![("seed", "seed")]),
    );

    let variant_functions: Vec<_> = [
        (2, Arc::clone(&variant_a_runs)),
        (3, Arc::clone(&variant_b_runs)),
        (4, Arc::clone(&variant_c_runs)),
    ]
    .into_iter()
    .map(|(multiplier, counter)| {
        move |inputs: &HashMap<String, GraphData>| {
            counter.fetch_add(1, Ordering::SeqCst);
            let value = inputs
                .get("x")
                .and_then(|data| data.as_int())
                .unwrap_or_default();
            output_int("scaled", value * multiplier)
        }
    })
    .collect();

    graph.variants(
        variant_functions,
        Some("Sweep"),
        Some(vec![("seed", "x")]),
        Some(vec![("scaled", "scaled")]),
    );
    graph.set_cache_version_for("Sweep (v0)", "sweep-v0");
    graph.set_cache_version_for("Sweep (v1)", "sweep-v1");
    graph.set_cache_version_for("Sweep (v2)", "sweep-v2");

    let dag = graph.build();
    let options = CacheOptions::default().with_namespace("sweep");
    dag.execute_detailed_with_options(true, Some(4), options.clone());
    dag.execute_detailed_with_options(true, Some(4), options);

    assert_eq!(source_runs.load(Ordering::SeqCst), 1);
    assert_eq!(variant_a_runs.load(Ordering::SeqCst), 1);
    assert_eq!(variant_b_runs.load(Ordering::SeqCst), 1);
    assert_eq!(variant_c_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn concurrent_execute_calls_share_inflight_computation() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut graph = Graph::new();
    graph.set_cache_version_for("SlowSource", "slow-v1");
    let counter = Arc::clone(&runs);

    graph.add(
        move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(50));
            output_int("value", 99)
        },
        Some("SlowSource"),
        None,
        Some(vec![("value", "value")]),
    );

    let dag = Arc::new(graph.build());
    let options = CacheOptions::default().with_namespace("inflight");

    let dag_a = Arc::clone(&dag);
    let options_a = options.clone();
    let first = thread::spawn(move || dag_a.execute_with_options(false, None, options_a));

    let dag_b = Arc::clone(&dag);
    let second = thread::spawn(move || dag_b.execute_with_options(false, None, options));

    assert_eq!(
        first
            .join()
            .unwrap()
            .get("value")
            .and_then(|data| data.as_int()),
        Some(99)
    );
    assert_eq!(
        second
            .join()
            .unwrap()
            .get("value")
            .and_then(|data| data.as_int()),
        Some(99)
    );
    assert_eq!(runs.load(Ordering::SeqCst), 1);
}

fn build_caf_cfar_graph(
    backend: Arc<dyn CacheBackend>,
    source_runs: Arc<AtomicUsize>,
    caf_runs: Arc<AtomicUsize>,
    cfar_runs: Arc<AtomicUsize>,
    cfar_version: &str,
    cfar_offset: i64,
) -> dagex::Dag {
    let mut graph = Graph::new();
    graph.with_cache_backend(backend);
    graph.set_cache_version_for("Source", "source-v1");
    graph.set_cache_version_for("CAF", "caf-v1");
    graph.set_cache_version_for("CFAR", cfar_version);

    let source_counter = Arc::clone(&source_runs);
    graph.add(
        move |_| {
            source_counter.fetch_add(1, Ordering::SeqCst);
            output_int("raw", 10)
        },
        Some("Source"),
        None,
        Some(vec![("raw", "raw")]),
    );

    let caf_counter = Arc::clone(&caf_runs);
    graph.add(
        move |inputs: &HashMap<String, GraphData>| {
            caf_counter.fetch_add(1, Ordering::SeqCst);
            let raw = inputs
                .get("x")
                .and_then(|value| value.as_int())
                .unwrap_or_default();
            output_int("caf", raw * 2)
        },
        Some("CAF"),
        Some(vec![("raw", "x")]),
        Some(vec![("caf", "caf")]),
    );

    let cfar_counter = Arc::clone(&cfar_runs);
    graph.add(
        move |inputs: &HashMap<String, GraphData>| {
            cfar_counter.fetch_add(1, Ordering::SeqCst);
            let caf = inputs
                .get("x")
                .and_then(|value| value.as_int())
                .unwrap_or_default();
            output_int("det", caf + cfar_offset)
        },
        Some("CFAR"),
        Some(vec![("caf", "x")]),
        Some(vec![("det", "det")]),
    );

    graph.build()
}

#[test]
fn equivalent_rebuilt_graphs_share_backend_and_reuse_upstream_nodes() {
    let backend: Arc<dyn CacheBackend> =
        Arc::new(MemoryCacheBackend::new(MemoryCacheConfig::default()));
    let source_runs = Arc::new(AtomicUsize::new(0));
    let caf_runs = Arc::new(AtomicUsize::new(0));
    let cfar_runs = Arc::new(AtomicUsize::new(0));

    let first = build_caf_cfar_graph(
        Arc::clone(&backend),
        Arc::clone(&source_runs),
        Arc::clone(&caf_runs),
        Arc::clone(&cfar_runs),
        "cfar-v1",
        1,
    );
    let second = build_caf_cfar_graph(
        Arc::clone(&backend),
        Arc::clone(&source_runs),
        Arc::clone(&caf_runs),
        Arc::clone(&cfar_runs),
        "cfar-v2",
        2,
    );

    let options = CacheOptions::default().with_namespace("shared-caf-cfar");
    let first_result = first.execute_detailed_with_options(false, None, options.clone());
    let second_result = second.execute_detailed_with_options(false, None, options);

    assert_eq!(
        first_result.get("det").and_then(|value| value.as_int()),
        Some(21)
    );
    assert_eq!(
        second_result.get("det").and_then(|value| value.as_int()),
        Some(22)
    );
    assert_eq!(source_runs.load(Ordering::SeqCst), 1);
    assert_eq!(caf_runs.load(Ordering::SeqCst), 1);
    assert_eq!(cfar_runs.load(Ordering::SeqCst), 2);
    assert_eq!(
        second_result
            .node_cache_status
            .get(&0)
            .map(|status| status.hit),
        Some(true)
    );
    assert_eq!(
        second_result
            .node_cache_status
            .get(&1)
            .map(|status| status.hit),
        Some(true)
    );
    assert_eq!(
        second_result
            .node_cache_status
            .get(&2)
            .and_then(|status| status.reason),
        Some(dagex::CacheMissReason::CodeChanged)
    );
}

#[test]
fn clearing_a_specific_node_marks_following_miss_as_invalidated() {
    let mut graph = Graph::new();
    graph.set_cache_version_for("Source", "source-v1");
    graph.add(
        |_| output_int("value", 12),
        Some("Source"),
        None,
        Some(vec![("value", "value")]),
    );
    let dag = graph.build();
    let options = CacheOptions::default().with_namespace("node-invalidation");

    dag.execute_detailed_with_options(false, None, options.clone());
    dag.clear_cache_node("node-invalidation", 0, Some("source-v1"));
    let second = dag.execute_detailed_with_options(false, None, options);

    assert_eq!(
        second
            .node_cache_status
            .get(&0)
            .and_then(|status| status.reason),
        Some(dagex::CacheMissReason::Invalidated)
    );
}

#[test]
fn backend_can_be_injected_via_build_and_execute_paths() {
    let shared_backend: Arc<dyn CacheBackend> =
        Arc::new(MemoryCacheBackend::new(MemoryCacheConfig::default()));
    let run_counter = Arc::new(AtomicUsize::new(0));

    let mut graph = Graph::new();
    graph.set_cache_version_for("Source", "source-v1");
    let counter = Arc::clone(&run_counter);
    graph.add(
        move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            output_int("value", 5)
        },
        Some("Source"),
        None,
        Some(vec![("value", "value")]),
    );

    let dag = graph.build_with_cache_backend(Arc::clone(&shared_backend));
    let options = CacheOptions::default().with_namespace("build-execute-injection");
    dag.execute_detailed_with_options(false, None, options.clone());
    let second = dag.execute_detailed_with_backend_options(
        false,
        None,
        options,
        Arc::clone(&shared_backend),
    );

    assert_eq!(run_counter.load(Ordering::SeqCst), 1);
    assert_eq!(second.cache_stats.hits, 1);
}
