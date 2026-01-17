# Implicit Edge Mapping

## Overview

Graph-sp supports **implicit edge mapping**, which allows you to build graphs without explicit `add_edge()` calls. Edges are automatically created by matching port names between nodes.

## How It Works

The `auto_connect()` method:
1. Iterates through all nodes in the graph
2. Matches output port names with input port names
3. Creates edges where port names match exactly
4. Respects the graph structure to avoid creating cycles

### Matching Strategy

- **Exact name matching**: Output port "data" connects to input port "data"
- **Case-sensitive**: Port names must match exactly
- **Multiple connections**: If multiple nodes have the same input port name, all will be connected to the matching output
- **Fan-out/fan-in patterns**: Automatically detected and connected

## Usage

### Rust

```rust
use graph_sp::core::{Graph, Node, NodeConfig, Port};

let mut graph = Graph::new();

// Add nodes with matching port names
let source = NodeConfig::new(
    "source",
    "Data Source",
    vec![],
    vec![Port::new("data", "Data Output")],
    function
);

let processor = NodeConfig::new(
    "processor",
    "Processor",
    vec![Port::new("data", "Data Input")],  // Matches "data" output!
    vec![Port::new("result", "Result")],
    function
);

graph.add_node(Node::new(source))?;
graph.add_node(Node::new(processor))?;

// Auto-connect based on port names
let edges_created = graph.auto_connect()?;
println!("Created {} edges", edges_created);
```

### Python

```python
import graph_sp

graph = graph_sp.Graph()

# Add nodes with matching port names
graph.add_node(
    "source",
    "Data Source",
    [],
    [graph_sp.Port("data", "Data Output")],
    source_fn
)

graph.add_node(
    "processor",
    "Processor",
    [graph_sp.Port("data", "Data Input")],  # Matches "data" output!
    [graph_sp.Port("result", "Result")],
    processor_fn
)

# Auto-connect based on port names
edges_created = graph.auto_connect()
print(f"Created {edges_created} edges")
```

## Examples

### Example 1: Simple Pipeline

```
source (output: "data")
  ↓ auto-connected
processor (input: "data", output: "result")
  ↓ auto-connected
sink (input: "result")
```

Result: 2 edges created automatically.

### Example 2: Parallel Branches (Fan-out/Fan-in)

```
source (output: "value")
  ↓ auto-connected to both branches
branch_a (input: "value", output: "out_a")
branch_b (input: "value", output: "out_b")
  ↓ auto-connected to merger
merger (inputs: "out_a", "out_b")
```

Result: 4 edges created automatically (2 fan-out + 2 fan-in).

## Best Practices

### 1. Use Descriptive Port Names

✅ **Good:**
```rust
Port::new("user_data", "User Data")
Port::new("processed_results", "Processed Results")
```

❌ **Bad:**
```rust
Port::new("data", "Data")  // Too generic
Port::new("x", "X")        // Not descriptive
```

### 2. Avoid Name Collisions

If you don't want ports to auto-connect, use unique names:

```rust
// These won't auto-connect
Port::new("input_a", "Input A")
Port::new("input_b", "Input B")
```

### 3. Verify Connections

Always validate after auto-connecting:

```rust
graph.auto_connect()?;
graph.validate()?;  // Check for cycles and errors
```

### 4. Mix Explicit and Implicit

You can combine both approaches:

```rust
// Auto-connect some edges
graph.auto_connect()?;

// Add specific edges explicitly
graph.add_edge(Edge::new("node1", "special", "node2", "custom"))?;
```

## When to Use

### Use Implicit Edge Mapping When:
- ✅ Building simple, linear pipelines
- ✅ Port names naturally describe data flow
- ✅ You want less boilerplate code
- ✅ Graph structure is obvious from port names

### Use Explicit Edge Mapping When:
- ✅ Complex routing logic required
- ✅ Multiple ports with similar data types
- ✅ Need fine-grained control over connections
- ✅ Port names don't naturally match

## Performance

- **Minimal overhead**: Auto-connect runs in O(N×M) where N is nodes and M is average ports per node
- **One-time operation**: Only runs when explicitly called
- **No runtime cost**: After connection, execution is identical to explicit edges

## See Also

- [Mermaid Visualization](MERMAID_VISUALIZATION.md) - See how auto-connected graphs are visualized
- [Examples](../examples/) - Complete working examples
- [API Documentation](https://docs.rs/graph-sp) - Full API reference
