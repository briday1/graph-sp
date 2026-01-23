<div align="center">
  <img src="https://raw.githubusercontent.com/briday1/graph-sp/main/assets/logo-banner.png" alt="dagex" width="600"/>
  <p><em>A pure Rust DAG executor with intelligent dependency resolution and parallel execution</em></p>

  [![Crates.io](https://img.shields.io/crates/v/dagex.svg)](https://crates.io/crates/dagex)
  [![Documentation](https://docs.rs/dagex/badge.svg)](https://docs.rs/dagex)
  [![License](https://img.shields.io/crates/l/dagex.svg)](LICENSE)
</div>

# dagex

**dagex** is a pure Rust DAG (Directed Acyclic Graph) executor that automatically resolves data dependencies and executes computational pipelines in parallel. This README walks through the core concepts with short, runnable Rust snippets, Mermaid visualizations and sample outputs.

## Highlights

- 🚀 Automatic parallelization of independent nodes
- 🔄 Dataflow-aware dependency resolution (broadcast → impl variable mapping)
- 🌳 Branching and merging with branch-scoped outputs
- 🔀 Parameter sweeps (variants)
- 📊 Mermaid visualization of the DAG
- ⚡ Zero-copy sharing for large data via Arc

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dagex = "2026.13"
```

## How to read these examples

- Code is Rust and uses the `dagex` API in this repository
- Each example prints a small Mermaid diagram and representative execution output
- These snippets are minimal; full examples are in `examples/rs/`

---

## 1) Minimal pipeline — sequential

This shows the simplest dataflow: a generator, a transformer, and a final aggregator.

```rust
use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::sync::Arc;

fn generate(_: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut o = HashMap::new();
    o.insert("n".to_string(), GraphData::int(10));
    o
}

fn double(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let v = inputs.get("x").and_then(|d| d.as_int()).unwrap_or(0);
    let mut o = HashMap::new();
    o.insert("y".to_string(), GraphData::int(v * 2));
    o
}

fn main() {
    let mut g = Graph::new();
    g.add(Arc::new(generate), Some("Source"), None, Some(vec![("n", "x")]));
    g.add(Arc::new(double), Some("Double"), Some(vec![("x", "x")]), Some(vec![("y", "out")]));

    let dag = g.build();
    println!("\n📊 Mermaid:\n{}\n", dag.to_mermaid());
    let ctx = dag.execute(false, None);
    println!("Result: {}", ctx.get("out").unwrap().as_int().unwrap());
}
```

Mermaid (example):

```
graph TD
    0["Source"]
    1["Double"]
    0 -->|n → x| 1
```

Output:

```
Result: 20
```

---

## 2) Parallel execution — independent workers

When nodes at the same level have no dependencies between them, they execute in parallel. This saves wall-clock time for slow tasks.

Key API: `dag.execute(true, Some(max_threads))` where `true` enables level-parallel execution.

Snippet outline:

- source produces `value`
- TaskA, TaskB, TaskC consume `value` and run independently
- results are stored as `result_a`, `result_b`, `result_c`

Expected behavior: A ~3× speedup when tasks are similar and independent.

---

## 3) Branching and merging

Fan-out (branch): create independent subgraphs that run in parallel.
Fan-in (merge): combine branch-specific outputs. The merge API maps `(branch_id, broadcast, impl)` so you can safely merge identical broadcast names from different branches.

Mermaid (branch+merge example):

```
graph TD
    0["Source"]
    1["BranchA"]
    2["BranchB"]
    3["Merge"]
    0 -->|data → x| 1
    0 -->|data → x| 2
    1 --> 3
    2 --> 3
```

Key implementation note: branch outputs are stored with branch-scoped keys internally so two branches can both produce `result` without clobbering each other.

---

## 4) Parameter sweeps (variants)

Variants let you create many nodes with the same structure but different captured parameters. The graph builder will attach them to the same frontier and the executor will schedule them at the same level when possible.

Example: run multipliers [2,3,5] in parallel and collect results.

---

## 5) Data model

`GraphData` provides typed containers: int, float, string, vectors, and Arc-backed large vectors for efficient sharing. Use `as_int()`, `as_float_vec()`, etc., to extract values in node functions.

---

## 6) Advanced API surfaces

- `Graph::add(function_handle, label, inputs, outputs)` — register a node
- `Graph::branch(subgraph)` — attach a branch; returns `branch_id`
- `Graph::merge(merge_fn, label, inputs, outputs)` — merge multiple branches
- `Graph::variants(vec<NodeFunction>, ...)` — add multiple variant nodes
- `graph.build()` → `Dag`
- `Dag::execute(parallel, max_threads)` → context (HashMap)
- `Dag::execute_detailed()` → `ExecutionResult` with `context`, `node_outputs`, `branch_outputs`

---

## Examples & demos

Full, runnable examples are under `examples/rs/`. Try them live:

```bash
cargo run --example comprehensive_demo --release
```

Mermaid diagrams (captured from `cargo run --example comprehensive_demo`) — use these to visualize the examples:

Demo 1 (Minimal pipeline):

```
graph TD
        0["Source"]
        1["Double"]
        0 -->|n → x| 1
```

Demo 2 (Parallel branching) — simplified mermaid:

```
graph TD
        0["Source"]
        1["Statistics"]
        2["MLModel"]
        3["Visualization"]
        0 -->|data → input| 1
        0 -->|data → input| 2
        0 -->|data → input| 3
```

Demo 3 (Branching + Merging):

```
graph TD
        0["Source"]
        1["PathA (+10)"]
        2["PathB (+20)"]
        5["Merge"]
        0 -->|data → x| 1
        0 -->|data → x| 2
        1 --> 5
        2 --> 5
```

Demo 4 (Variants):

```
graph TD
        0["DataSource"]
        1["ScaleLR (v0)"]
        2["ScaleLR (v1)"]
        3["ScaleLR (v2)"]
        4["ScaleLR (v3)"]
        0 -->|data → input| 1
        0 -->|data → input| 2
        0 -->|data → input| 3
        0 -->|data → input| 4
```

Complex graph (all features):

```
graph TD
        0["Ingest"]
        1["Preprocess"]
        2["Stats"]
        3["ML"]
        6["Combine"]
        7["Format"]
        0 -->|data → raw| 1
        1 -->|clean_data → data| 2
        1 -->|clean_data → data| 3
        2 --> 6
        3 --> 6
        6 -->|final_report → report| 7
```

## Python bindings and PyPI

This repository exposes the same functionality to Python; see `README_PYPI.md` for the PyPI-oriented guide.

---

## Contributing

Contributions welcome. If you fix or extend branching/merging semantics or add new GraphData types, please add tests in `tests/` and examples under `examples/`.

## License

MIT

---

<div align="center">Built with ❤️ in Rust — star the repo if you like it!</div>
