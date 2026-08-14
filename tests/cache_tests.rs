use dagex::{
    CacheBackend, CacheDepth, CacheOptions, Graph, GraphData, MemoryCacheBackend, MemoryCacheConfig,
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

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
    assert_eq!(second.get("result").and_then(|data| data.as_int()), Some(30));
    assert_eq!(source_runs.load(Ordering::SeqCst), 1);
    assert_eq!(process_runs.load(Ordering::SeqCst), 1);
    assert_eq!(first.cache_stats.hits, 0);
    assert_eq!(second.cache_stats.hits, 2);
}

#[test]
fn cache_depth_none_disables_reuse() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut graph = Graph::new();
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
        graph.add(
            source_one,
            Some("Source"),
            None,
            Some(vec![("raw", "raw")]),
        );
    } else {
        graph.set_cache_version_for("Source", "source-v2");
        graph.add(
            source_two,
            Some("Source"),
            None,
            Some(vec![("raw", "raw")]),
        );
    }

    let normalize_counter = Arc::clone(&normalize_runs);
    graph.add(
        move |inputs: &HashMap<String, GraphData>| {
            normalize_counter.fetch_add(1, Ordering::SeqCst);
            let _ = inputs.get("raw").and_then(|data| data.as_int()).unwrap_or_default();
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

    let variant_functions: Vec<_> = [(2, Arc::clone(&variant_a_runs)), (3, Arc::clone(&variant_b_runs)), (4, Arc::clone(&variant_c_runs))]
        .into_iter()
        .map(|(multiplier, counter)| {
            move |inputs: &HashMap<String, GraphData>| {
                counter.fetch_add(1, Ordering::SeqCst);
                let value = inputs.get("x").and_then(|data| data.as_int()).unwrap_or_default();
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

    let dag = graph.build();
    let options = CacheOptions::default().with_namespace("sweep");
    dag.execute_detailed_with_options(true, Some(4), options.clone());
    dag.execute_detailed_with_options(true, Some(4), options);

    assert_eq!(source_runs.load(Ordering::SeqCst), 1);
    assert_eq!(variant_a_runs.load(Ordering::SeqCst), 1);
    assert_eq!(variant_b_runs.load(Ordering::SeqCst), 1);
    assert_eq!(variant_c_runs.load(Ordering::SeqCst), 1);
}
