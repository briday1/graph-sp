# graph-sp

A high-performance DAG (Directed Acyclic Graph) execution engine with true parallel execution, built in Rust with Python bindings.

[![CI](https://github.com/briday1/graph-sp/workflows/CI/badge.svg)](https://github.com/briday1/graph-sp/actions)
[![PyPI](https://img.shields.io/pypi/v/graph-sp.svg)](https://pypi.org/project/graph-sp/)
[![Crates.io](https://img.shields.io/crates/v/graph-sp.svg)](https://crates.io/crates/graph-sp)

## Features

- **⚡ True Parallel Execution**: Automatic parallelization of independent nodes (44% faster for fan-out patterns)
- **🔌 Port-based Architecture**: Type-safe data flow between nodes via named ports
- **🌿 Branching & Nested Graphs**: Create isolated subgraphs for experiments and variants
- **🔀 Merge Operations**: Combine outputs from multiple branches with custom merge functions
- **🔄 Variants & Config Sweeps**: Automated parameter variation with cartesian product support
- **🐍 Python & Rust APIs**: Full feature parity across both languages
- **🔍 Graph Inspection**: Analysis, visualization, and Mermaid diagram generation
- **✅ Cycle Detection**: Built-in DAG validation with detailed error reporting
- **📊 Rich Data Types**: Primitives, collections, JSON, nested objects, and binary data
- **🎯 Zero-Copy Optimization**: Efficient data sharing using Arc

## Quick Start

### Python

Install from PyPI:

```bash
pip install pygraph-sp
```

Simple example:

```python
import pygraph_sp as gs

# Create a graph
graph = gs.Graph()

# Add nodes with Python functions using the simplified API
def data_source(inputs):
    return {"output": [1, 2, 3, 4, 5]}

def multiply_by_2(inputs):
    return {"output": [x * 2 for x in inputs["input"]]}

# Add nodes - function name becomes the node ID by default
graph.add(
    data_source,
    label="Data Source",
    outputs=["output"]
)

graph.add(
    multiply_by_2,
    label="Multiply by 2",
    inputs=["input"],
    outputs=["output"]
)

# Connect nodes using function names
graph.add_edge("data_source", "output", "multiply_by_2", "input")

# Execute with parallel processing
executor = gs.Executor()
result = executor.execute(graph)

print(result.get_output("multiply_by_2", "output"))  # [2, 4, 6, 8, 10]
```

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
graph-sp = "0.1"
tokio = { version = "1", features = ["full"] }
```

Simple example:

```rust
use graph_sp::{Graph, Node, NodeConfig, Port, PortData, Edge, Executor};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::new();

    // Add source node
    let source = NodeConfig::new(
        "source", "Data Source",
        vec![],
        vec![Port::simple("output")],
        Arc::new(|_: &HashMap<String, PortData>| {
            Ok(HashMap::from([
                ("output".to_string(), PortData::List(vec![
                    PortData::Int(1), PortData::Int(2), PortData::Int(3)
                ]))
            ]))
        }),
    );

    graph.add(Node::new(source))?;

    // Execute
    let executor = Executor::new();
    let result = executor.execute(&mut graph).await?;

    Ok(())
}
```

## Installation

### Python

**From PyPI (recommended):**

```bash
pip install pygraph-sp
```

**From source:**

```bash
# Clone the repository
git clone https://github.com/briday1/graph-sp.git
cd graph-sp

# Install maturin
pip install maturin

# Build and install
maturin develop --release --features python
```

### Rust

**From crates.io:**

```toml
[dependencies]
graph-sp = "0.1"
```

**From source:**

```bash
git clone https://github.com/briday1/graph-sp.git
cd graph-sp
cargo build --release
```

## Core Concepts

### Ports

Nodes communicate through typed ports with separate broadcast and implementation names:

```python
# Python - Simple ports (broadcast_name == impl_name)
graph.add(
    my_function,
    inputs=["data", "config"],
    outputs=["result"]
)

# Advanced: Separate broadcast and implementation names
# Useful for connecting nodes with different parameter names
graph.add(
    process_data,
    inputs=[("external_data", "internal_param")],  # (broadcast, impl)
    outputs=[("result", "output_value")]
)

# Most explicit: Use Port objects
input_port = gs.Port("broadcast_name", "impl_name", "Display Name")
graph.add(my_function, inputs=[input_port], outputs=[...])
```

```rust
// Rust - Simple port (both names the same)
let port = Port::simple("data");

// Separate broadcast and implementation names
let port = Port::new("external_data", "internal_param");
```

**Port Name Types:**
- **broadcast_name**: External name used for connecting nodes via edges
- **impl_name**: Internal parameter name used in function signatures
- **display_name**: Human-readable name for visualizations (defaults to broadcast_name)

### Data Types

Support for multiple data types:

- **Primitives**: Int, Float, String, Bool, None
- **Collections**: List, Map (nested HashMap)
- **Structured**: JSON (arbitrary structures)
- **Binary**: Bytes (raw binary data)

```python
# Python - nested objects work seamlessly
user = {
    "name": "Alice",
    "age": 30,
    "address": {
        "city": "NYC",
        "zip": "10001"
    },
    "hobbies": ["reading", "coding"]
}
```

### Parallel Execution

The engine automatically parallelizes independent branches:

```python
# This creates a fan-out pattern with 3 parallel branches
#        source
#       /   |   \
#   slow  fast  medium    <- These run in parallel!
#       \   |   /
#        merger

# Execution time: ~500ms (parallel) vs ~900ms (sequential)
# 44% speedup achieved automatically!
```

### Graph Inspection

```python
# Analyze graph structure
analysis = graph.analyze()
print(f"Nodes: {analysis.node_count}")
print(f"Depth: {analysis.depth}")
print(f"Width: {analysis.width}")  # Parallelization potential

# Generate Mermaid diagram
mermaid = graph.to_mermaid()
print(mermaid)  # GitHub-compatible markdown
```

### Branching & Variants (New in v0.2.0)

**Branches** allow you to create isolated subgraphs:

```python
# Create experimental branches
graph.create_branch("experiment_a")
graph.create_branch("experiment_b")

# Check branches
print(graph.branch_names())  # ["experiment_a", "experiment_b"]
print(graph.has_branch("experiment_a"))  # True
```

**Variants** enable config sweeps and hyperparameter tuning:

```python
# Python - Create variants with different learning rates
def learning_rate_gen(i):
    rates = [0.001, 0.01, 0.1]
    return gs.PortData(rates[i])

branches = graph.create_variants(
    name_prefix="lr",
    count=3,
    param_name="learning_rate",
    variant_function=learning_rate_gen,
    parallel=True  # Enable parallel execution
)
# Creates: lr_0 (0.001), lr_1 (0.01), lr_2 (0.1)
```

```rust
// Rust - Same functionality
use graph_sp::core::{VariantConfig, VariantFunction};

let variant_fn: VariantFunction = Arc::new(|i: usize| {
    PortData::Float((i as f64 + 1.0) * 0.01)
});

let config = VariantConfig::new("lr", 3, "learning_rate", variant_fn);
let branches = graph.create_variants(config)?;
```

**Merge** combines outputs from multiple branches:

```python
# Python - Merge outputs from variant branches
graph.merge_branches(
    node_id="select_best",
    branches=variant_branches,
    port="accuracy"  # Merges to "accuracies" (pluralized)
)

# Custom merge function (e.g., average)
def compute_average(values):
    return gs.PortData(sum(values) / len(values))

graph.merge_branches(
    node_id="summarize",
    branches=variant_branches,
    port="score",
    merge_function=compute_average
)
```

```rust
// Rust - Same functionality
use graph_sp::core::MergeConfig;

let merge_config = MergeConfig::new(
    vec!["branch_a".to_string(), "branch_b".to_string()],
    "result".to_string()
);
graph.merge("merge_node", merge_config)?;

// Custom merge function
let max_fn = Arc::new(|inputs: Vec<&PortData>| -> Result<PortData> {
    let max = inputs.iter()
        .filter_map(|d| if let PortData::Int(v) = d { Some(*v) } else { None })
        .max()
        .unwrap_or(0);
    Ok(PortData::Int(max))
});

let config = MergeConfig::new(branches, "score".to_string())
    .with_merge_fn(max_fn);
```

**Nested Variants** create cartesian products:

```rust
// 2 learning rates × 3 batch sizes = 6 total configurations
let lr_config = VariantConfig::new("lr", 2, "learning_rate", lr_fn);
let lr_branches = graph.create_variants(lr_config)?;

for lr_branch in &lr_branches {
    let branch = graph.get_branch_mut(lr_branch)?;
    let batch_config = VariantConfig::new("batch", 3, "batch_size", batch_fn);
    branch.create_variants(batch_config)?;
}
```

## Examples

### Python Examples

Located in `python_examples/`:

- **simple_pipeline.py**: Basic 3-node pipeline with graph analysis
- **port_mapping_demo.py**: Port name separation (broadcast vs implementation)
- **variants_demo.py**: Hyperparameter sweeps, custom merge, nested variants
- **complex_objects.py**: Demonstrates nested objects, JSON, and lists
- **parallel_execution.py**: Shows 3-branch parallel execution with timing
- **branching_example.py**: Demonstrates branch creation and management

Run an example:

```bash
cd python_examples
python simple_pipeline.py
python variants_demo.py
```

### Rust Examples

Located in `examples/`:

- **simple_pipeline.rs**: 4-node data processing pipeline
- **complex_objects.rs**: All PortData types with nested structures
- **parallel_execution.rs**: Fan-out/fan-in pattern with performance analysis
- **branching_and_variants.rs**: Comprehensive demo of branches, merge, and variants

Run an example:

```bash
cargo run --example simple_pipeline
cargo run --example parallel_execution
cargo run --example branching_and_variants
```

## Performance

Measured with 3-branch parallel execution example:

| Metric | Sequential | Parallel | Improvement |
|--------|-----------|----------|-------------|
| **Rust** | 900ms | 500ms | 44% faster |
| **Python** | 900ms | 502ms | 44% faster |

The executor identifies dependency levels and executes all independent nodes concurrently using `tokio::task::spawn_blocking`.

## Architecture

### Core Components

1. **Data Model** (`src/core/data.rs`)
   - `PortData`: Enum for all supported types
   - `GraphData`: HashMap for port-to-port storage
   - Support for nested structures via recursive variants

2. **Graph** (`src/core/graph.rs`)
   - petgraph-backed DAG representation
   - Topological sorting and cycle detection
   - Type-safe port connections

3. **Executor** (`src/executor/mod.rs`)
   - Dependency-level grouping
   - Concurrent execution with `tokio::task::spawn_blocking`
   - Automatic concurrency management

4. **Inspector** (`src/inspector/mod.rs`)
   - Graph statistics (depth, width, sources, sinks)
   - Mermaid diagram generation
   - Optimization suggestions

5. **Python Bindings** (`src/python/mod.rs`)
   - PyO3-based wrappers
   - GIL-aware parallel execution
   - Automatic type conversion

## Building & Testing

### Rust

```bash
# Run tests
cargo test

# Run with all features
cargo test --all-features

# Build release
cargo build --release

# Run examples
cargo run --example simple_pipeline
```

### Python

```bash
# Install development dependencies
pip install maturin pytest

# Build Python bindings
maturin develop --features python

# Run Python examples
python python_examples/simple_pipeline.py
```

### Build for Multiple Platforms

```bash
# Linux (using Docker for manylinux compatibility)
docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin build --release --features python

# macOS
maturin build --release --features python

# Windows
maturin build --release --features python
```

## Documentation

- **Python API**: See `python_examples/README.md` for detailed Python usage
- **Rust API**: Run `cargo doc --open` for full API documentation
- **Port Data Types**: See `docs/PORT_DATA_TYPES.md` for supported types

## Publishing

### PyPI

Wheels are automatically built and published to PyPI on version tags:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Builds wheels for:
- Linux (manylinux)
- macOS (Intel & ARM)
- Windows

### Crates.io

```bash
cargo publish
```

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass (`cargo test --all-features`)
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Roadmap

- [x] True parallel execution
- [x] Python bindings with PyPI distribution
- [x] Mermaid diagram generation
- [x] Comprehensive examples
- [ ] Distributed execution support
- [ ] Graph serialization/deserialization
- [ ] WebAssembly support
- [ ] Real-time monitoring dashboard
- [ ] Advanced optimization algorithms

