<div align="center">
  <img src="https://raw.githubusercontent.com/briday1/graph-sp/main/assets/logo-banner.png" alt="dagex" width="600"/>
  <p><em>A pure Rust DAG executor with intelligent dependency resolution and parallel execution</em></p>

  [![Crates.io](https://img.shields.io/crates/v/dagex.svg)](https://crates.io/crates/dagex)
  [![Documentation](https://docs.rs/dagex/badge.svg)](https://docs.rs/dagex)
  [![License](https://img.shields.io/crates/l/dagex.svg)](LICENSE)
</div>

# dagex

**dagex** is a pure Rust DAG (Directed Acyclic Graph) executor that automatically resolves data dependencies and executes computational pipelines in parallel. Build complex workflows with simple, composable functions.

## ✨ Highlights

- 🚀 **Automatic parallelization** of independent nodes
- 🔄 **Dataflow-aware dependency resolution** (broadcast → impl variable mapping)
- 🌳 **Branching and merging** with branch-scoped outputs
- 🔀 **Parameter sweeps** (variants) for hyperparameter exploration
- 📊 **Mermaid visualization** of the DAG structure
- ⚡ **Zero-copy sharing** for large data via Arc
- 🐍 **Python bindings** for seamless integration

## 📦 Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
dagex = "2026.15"
```

### Python

```bash
pip install dagex
```

## 🎯 Quick Start

Here's a minimal example showing the core concepts:

```rust
use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    let mut graph = Graph::new();
    
    // Add a data source
    graph.add(
        Arc::new(|_| {
            let mut out = HashMap::new();
            out.insert("value".to_string(), GraphData::int(10));
            out
        }),
        Some("Source"),
        None,
        Some(vec![("value", "x")])
    );
    
    // Add a processor
    graph.add(
        Arc::new(|inputs: &HashMap<String, GraphData>| {
            let v = inputs.get("x").and_then(|d| d.as_int()).unwrap_or(0);
            let mut out = HashMap::new();
            out.insert("result".to_string(), GraphData::int(v * 2));
            out
        }),
        Some("Doubler"),
        Some(vec![("x", "x")]),
        Some(vec![("result", "output")])
    );
    
    let dag = graph.build();
    let context = dag.execute(false, None);
    
    println!("Result: {}", context.get("output").unwrap().as_int().unwrap());
    // Output: Result: 20
}
```

## 📚 Examples

All examples include:
- 📊 Mermaid DAG diagrams for visualization
- ⏱️ Runtime and memory measurements
- 📖 Narrative explanations of concepts

Run any example with:

```bash
# Rust
cargo run --example 01_minimal_pipeline --release
cargo run --example 02_parallel_vs_sequential --release
cargo run --example 03_branch_and_merge --release
cargo run --example 04_variants_sweep --release
cargo run --example 05_output_access --release
cargo run --example 06_graphdata_large_payload_arc_or_shared_data --release

# Python
python3 examples/py/01_minimal_pipeline.py
python3 examples/py/02_parallel_vs_sequential.py
python3 examples/py/03_branch_and_merge.py
python3 examples/py/04_variants_sweep.py
python3 examples/py/05_output_access.py
python3 examples/py/06_graphdata_large_payload_arc_or_shared_data.py
```

### Example 01: Minimal Pipeline

The simplest possible DAG: generator → transformer → aggregator.

**Rust output:**

```
════════════════════════════════════════════════════════════
  Example 01: Minimal Pipeline
════════════════════════════════════════════════════════════

📖 Story:
   This example shows the simplest possible DAG pipeline:
   A generator creates a number, a transformer doubles it,
   and a final node adds five to produce the result.


────────────────────────────────────────────────────────────
Mermaid Diagram
────────────────────────────────────────────────────────────

graph TD
    0["Generator"]
    1["Doubler"]
    2["AddFive"]
    0 -->|x → x| 1
    1 -->|y → y| 2


────────────────────────────────────────────────────────────
ASCII Visualization
────────────────────────────────────────────────────────────

  Generator → Doubler → AddFive
     (10)       (20)       (25)

────────────────────────────────────────────────────────────
Execution
────────────────────────────────────────────────────────────

⏱️  Runtime: 0.012ms
💾 Memory: RSS: 2080 kB

────────────────────────────────────────────────────────────
Results
────────────────────────────────────────────────────────────

✅ Final output: 25
   (Started with 10, doubled to 20, added 5 = 25)
```

### Example 02: Parallel vs Sequential Execution

Demonstrates the power of parallel execution for independent tasks.

**Key insight:** Three tasks that each take ~50ms run in ~150ms sequentially but only ~50ms in parallel—a **3x speedup**!

**Rust output:**

```
⚡ Speedup: 2.98x faster with parallel execution!
```

### Example 03: Branch and Merge

Fan-out (branching) and fan-in (merging) patterns for complex workflows.

**Mermaid diagram:**

```
graph TD
    0["Source"]
    1["PathA (+10)"]
    2["PathB (+20)"]
    5["Merge"]
    0 -->|x → x| 1
    0 -->|x → x| 2
    1 --> 5
    2 --> 5
```

**Result:** 50 → PathA(60) + PathB(70) → Merge(130)

### Example 04: Variants (Parameter Sweep)

Run multiple variants in parallel—perfect for hyperparameter tuning or A/B testing.

**Rust output:**

```
📊 Base value: 10

Detailed variant outputs:
  Variant 0 (×2): 20
  Variant 1 (×3): 30
  Variant 2 (×5): 50
  Variant 3 (×7): 70

✅ All 4 variants executed successfully!
```

### Example 05: Output Access

Access intermediate results and branch outputs, not just final values.

**Rust output:**

```
📊 Accessing different output levels:

1. Final context outputs:
   output: 351

2. Individual node outputs:
   Total nodes executed: 6

3. Branch-specific outputs:
   Total branches: 2
   Branch 1:
     result_a: 200
   Branch 2:
     result_b: 150
```

### Example 06: Zero-Copy Data Sharing

Large data is automatically wrapped in `Arc` for efficient sharing without copying.

**Key insight:** A 1M integer vector is created once and shared by reference across all consumers. No data duplication!

**Rust output:**

```
⏱️  Runtime: 1.658ms
💾 Memory: RSS: 10212 kB

📊 Consumer outputs (each processes different segments):
   ConsumerA (first 1000):  sum = 499500
   ConsumerB (next 1000):   sum = 1499500
   ConsumerC (next 1000):   sum = 2499500

✅ Zero-copy data sharing successful!
```

## 🔧 Core API

### Graph Builder

```rust
use dagex::{Graph, GraphData};
use std::sync::Arc;

let mut graph = Graph::new();

// Add a node
graph.add(
    Arc::new(function),      // Function handle
    Some("NodeLabel"),       // Optional label
    Some(vec![("in", "x")]), // Input mapping: broadcast → impl
    Some(vec![("out", "y")]) // Output mapping: impl → broadcast
);

// Create a branch
let branch_id = graph.branch(subgraph);

// Merge branches
graph.merge(
    merge_function,
    Some("Merge"),
    vec![(branch_id_a, "out_a", "in_a"), (branch_id_b, "out_b", "in_b")],
    Some(vec![("result", "final")])
);

// Add variants (parameter sweep)
graph.variants(
    vec![func1, func2, func3],
    Some("Variants"),
    Some(vec![("input", "x")]),
    Some(vec![("output", "results")])
);

// Build and execute
let dag = graph.build();
let context = dag.execute(parallel, max_threads);
```

### GraphData Types

```rust
GraphData::int(42)                    // i64
GraphData::float(3.14)                // f64
GraphData::string("hello")            // String
GraphData::int_vec(vec![1,2,3])       // Arc<Vec<i64>>
GraphData::float_vec(vec![1.0,2.0])   // Arc<Vec<f64>>
GraphData::map(HashMap::new())        // Nested data
```

### Execution

```rust
// Simple execution
let context = dag.execute(parallel: bool, max_threads: Option<usize>);
let result = context.get("output_name").unwrap().as_int().unwrap();

// Detailed execution (access per-node and per-branch outputs)
let exec_result = dag.execute_detailed(parallel, max_threads);
let final_context = exec_result.context;
let node_outputs = exec_result.node_outputs;
let branch_outputs = exec_result.branch_outputs;
```

## 🐍 Python Usage

See [`README_PYPI.md`](README_PYPI.md) for Python-specific documentation with examples and API reference.

## 🤝 Contributing

Contributions are welcome! Please:

1. Add tests for new features in `tests/`
2. Add examples under `examples/rs/` and `examples/py/`
3. Update documentation as needed
4. Run `cargo test` and verify examples work

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🔗 Links

- **Crate:** https://crates.io/crates/dagex
- **Documentation:** https://docs.rs/dagex
- **Repository:** https://github.com/briday1/graph-sp
- **Python Package:** https://pypi.org/project/dagex

---

<div align="center">Built with ❤️ in Rust — star the repo if you find it useful!</div>
