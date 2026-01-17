# graph-sp

A pure Rust graph executor supporting implicit node connections, branching, and config sweeps.

## Features

- **Implicit Node Connections**: Nodes are automatically connected based on execution order
- **Branching & Merging**: Create parallel execution paths with `.branch()` and merge them with `.merge()`
- **Config Sweeps**: Use `.variant_linspace()`, `.variant_logspace()`, or custom generators for parameter sweeps
- **DAG Optimization**: Automatic inspection and optimization of execution paths during `.build()`
- **Mermaid Visualization**: Generate diagrams with `dag.to_mermaid()`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
graph-sp = "0.1"
```

## Quick Start

```rust
use graph_sp::Graph;
use std::collections::HashMap;

let mut graph = Graph::new();

// Add nodes with implicit sequential connections
graph.add(
    |_| {
        let mut outputs = HashMap::new();
        outputs.insert("data".to_string(), "hello".to_string());
        outputs
    },
    Some("Source"),           // Optional label
    None,                     // Optional broadcast_vars
    Some(vec!["data"]),       // Optional output_vars
);

graph.add(
    |inputs| {
        let mut outputs = HashMap::new();
        if let Some(data) = inputs.get("data") {
            outputs.insert("result".to_string(), data.to_uppercase());
        }
        outputs
    },
    Some("Uppercase"),
    Some(vec!["data"]),       // Consumes "data" from context
    Some(vec!["result"]),     // Produces "result"
);

// Build and execute
let dag = graph.build();
let result = dag.execute();
println!("{}", result.get("result").unwrap()); // "HELLO"
```

## Core API

### `.add(function, label?, broadcast_vars?, output_vars?)`

Add a node to the graph with implicit connections.

- **function_handle**: The function to execute for this node
- **label**: Optional label for visualization
- **broadcast_vars**: Optional list of broadcast variables consumed from graph context
- **output_vars**: Optional list of output variables this node produces

Nodes are automatically connected to the previous node, creating a sequential pipeline.

### `.branch(subgraph)`

Insert a branching subgraph for parallel execution.

```rust
let mut graph = Graph::new();

graph.add(source_fn, Some("Source"), None, Some(vec!["value"]));

// Create two parallel branches
let mut branch_a = Graph::new();
branch_a.add(process_a, Some("Path A"), Some(vec!["value"]), Some(vec!["result_a"]));

let mut branch_b = Graph::new();
branch_b.add(process_b, Some("Path B"), Some(vec!["value"]), Some(vec!["result_b"]));

graph.branch(branch_a);
graph.branch(branch_b); // Sequential .branch() calls branch from the same node
```

### `.merge()`

Merge multiple branches back together. The next added node will depend on all branches.

```rust
graph.branch(branch_a);
graph.branch(branch_b);
graph.merge(); // Merge both branches

graph.add(combine_fn, Some("Combine"), 
    Some(vec!["result_a", "result_b"]), 
    Some(vec!["final"]));
```

### Variant Methods for Config Sweeps

#### `.variant_linspace(param_name, start, end, count)`

Create variants with linearly spaced parameter values.

```rust
// Create 10 variants with learning rates from 0.001 to 0.1
graph.variant_linspace("learning_rate", 0.001, 0.1, 10);
```

#### `.variant_logspace(param_name, start, end, count)`

Create variants with logarithmically spaced parameter values.

```rust
// Create 5 variants: 0.0001, 0.001, 0.01, 0.1
graph.variant_logspace("lr", 0.0001, 0.1, 4);
```

#### `.variant_geomspace(param_name, start, ratio, count)`

Create variants with geometric progression.

```rust
// Create 5 variants: 0.001, 0.01, 0.1, 1.0, 10.0
graph.variant_geomspace("learning_rate", 0.001, 10.0, 5);
```

#### `.variant_sweep(param_name, count, generator)`

Create variants using a custom generator function.

```rust
// Custom generator: powers of 2
graph.variant_sweep("batch_size", 4, |i| {
    format!("{}", 2_u32.pow(i as u32 + 3)) // 8, 16, 32, 64
});
```

## DAG Inspection

The `.build()` method performs implicit inspection:

- Full graph traversal
- Topological sort for execution order
- Dependency level computation
- Identification of parallelizable operations

```rust
let dag = graph.build();

// Get statistics
let stats = dag.stats();
println!("Nodes: {}", stats.node_count);
println!("Depth: {} levels", stats.depth);
println!("Max Parallelism: {} nodes", stats.max_parallelism);

// Execute
let result = dag.execute();

// Visualize as Mermaid diagram
println!("{}", dag.to_mermaid());
```

## Examples

Run the comprehensive example showing all features:

```bash
cargo run --example all_features
```

This demonstrates:
1. Simple sequential pipeline
2. Branching with merge
3. Variants with linspace
4. Variants with logspace
5. Custom variant sweep
6. Mermaid visualization

## Architecture

The library consists of:

- **`Graph`** - Builder for constructing graphs with implicit connections
- **`Dag`** - Optimized DAG representation for execution
- **`Node`** - Individual computation units
- **`ExecutionContext`** - HashMap storing variable values during execution

## License

MIT
