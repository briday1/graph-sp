# graph-sp

A comprehensive Rust-based DAG (Directed Acyclic Graph) execution engine with Python bindings. Built for performance with true parallelization, flawless graph inspection, and port routing optimization.

## Features

- **🚀 Parallel Execution**: Automatic parallelization of independent nodes using tokio
- **🔌 Port-based Architecture**: Type-safe communication between nodes through ports
- **🔍 Graph Inspection**: Comprehensive analysis, visualization, and optimization tools
- **✅ Cycle Detection**: Built-in DAG validation with detailed error reporting
- **🐍 Python Bindings**: Easy-to-use Python API via PyO3
- **📊 Rich Data Types**: Support for primitives, collections, JSON, and binary data

## Architecture

### Core Components

1. **Data Types** (`src/core/data.rs`)
   - `GraphData`: Port container with HashMap storage
   - `PortData`: Enum supporting multiple data types (Bool, Int, Float, String, Bytes, JSON, List, Map)
   - `Port`: Port configuration with required/optional support

2. **Graph Structure** (`src/core/graph.rs`)
   - `Graph`: DAG representation using petgraph
   - `Node`: Node with input/output ports and execution function
   - `Edge`: Port-to-port connections
   - Built-in topological sorting and cycle detection

3. **Executor** (`src/executor/mod.rs`)
   - Parallel execution engine using tokio
   - Automatic concurrency management
   - Dependency resolution and task scheduling

4. **Inspector** (`src/inspector/mod.rs`)
   - Graph analysis and statistics
   - Optimization suggestions
   - Text-based visualization

5. **Python Bindings** (`src/python/mod.rs`)
   - PyO3-based Python API
   - Async execution support
   - Automatic type conversion

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
graph-sp = "0.1.0"
tokio = { version = "1.35", features = ["full"] }
```

### Python

With Python bindings (requires `python` feature):

```bash
pip install graph-sp
```

Or build from source:

```bash
cargo build --release --features python
```

## Usage

### Rust Example

```rust
use graph_sp::core::{Graph, Node, NodeConfig, Port, PortData, Edge};
use graph_sp::executor::Executor;
use graph_sp::inspector::Inspector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a graph
    let mut graph = Graph::new();

    // Source node: outputs a constant
    let source_config = NodeConfig::new(
        "source",
        "Source Node",
        vec![],
        vec![Port::new("output", "Output")],
        Arc::new(|_inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("output".to_string(), PortData::Int(10));
            Ok(outputs)
        }),
    );

    // Transform node: doubles the input
    let transform_config = NodeConfig::new(
        "transform",
        "Transform Node",
        vec![Port::new("input", "Input")],
        vec![Port::new("output", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("input") {
                outputs.insert("output".to_string(), PortData::Int(val * 2));
            }
            Ok(outputs)
        }),
    );

    // Add nodes to graph
    graph.add_node(Node::new(source_config))?;
    graph.add_node(Node::new(transform_config))?;

    // Connect nodes
    graph.add_edge(Edge::new("source", "output", "transform", "input"))?;

    // Validate graph
    graph.validate()?;

    // Analyze graph
    use graph_sp::Inspector;
    let analysis = Inspector::analyze(&graph);
    println!("Graph Analysis: {}", analysis.summary());

    // Execute graph
    let executor = Executor::new();
    let result = executor.execute(&mut graph).await?;

    // Get results
    if let Some(PortData::Int(val)) = result.get_output("transform", "output") {
        println!("Result: {}", val); // Output: 20
    }

    Ok(())
}
```

### Python Example

```python
import graph_sp
import asyncio

async def main():
    # Create a graph
    graph = graph_sp.Graph()

    # Define node functions
    def source_fn(inputs):
        return {"output": 10}

    def double_fn(inputs):
        return {"output": inputs["input"] * 2}

    # Add nodes
    graph.add_node(
        "source",
        "Source Node",
        [],  # no input ports
        [graph_sp.Port("output", "Output")],
        source_fn
    )

    graph.add_node(
        "doubler",
        "Doubler Node",
        [graph_sp.Port("input", "Input")],
        [graph_sp.Port("output", "Output")],
        double_fn
    )

    # Connect nodes
    graph.add_edge("source", "output", "doubler", "input")

    # Validate
    graph.validate()

    # Analyze
    analysis = graph.analyze()
    print(f"Graph has {analysis.node_count} nodes and {analysis.edge_count} edges")

    # Execute
    executor = graph_sp.Executor()
    result = await executor.execute(graph)

    # Get result
    output = result.get_output("doubler", "output")
    print(f"Result: {output}")  # Output: 20

asyncio.run(main())
```

## Graph Inspection

```rust
use graph_sp::{Inspector, Graph};

// Analyze graph
let analysis = Inspector::analyze(&graph);
println!("Nodes: {}", analysis.node_count);
println!("Edges: {}", analysis.edge_count);
println!("Depth: {}", analysis.depth);
println!("Width: {}", analysis.width);

// Visualize graph structure
let visualization = Inspector::visualize(&graph)?;
println!("{}", visualization);

// Get optimization suggestions
let optimizations = Inspector::suggest_optimizations(&graph);
for opt in optimizations {
    println!("{}: {}", opt.optimization_type, opt.description);
}
```

## Testing

Run the test suite:

```bash
cargo test
```

Run tests with Python bindings:

```bash
cargo test --features python
```

## Performance

- **Parallel Execution**: Automatically parallelizes independent nodes
- **Zero-copy Data**: Efficient data sharing between nodes using Arc
- **Async Runtime**: Built on tokio for high-performance async execution
- **Optimized Graph Operations**: Uses petgraph for efficient graph algorithms

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

- [ ] Advanced port routing optimization algorithms
- [ ] Graph serialization/deserialization
- [ ] More optimization suggestions
- [ ] WebAssembly support
- [ ] Distributed execution support
- [ ] Real-time graph monitoring and debugging
