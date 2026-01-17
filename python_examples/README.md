# pygraph-sp Python Package

Python bindings for the graph-sp DAG execution engine with true parallel execution.

For complete documentation, visit the [main repository](https://github.com/briday1/graph-sp).

## Installation

```bash
pip install pygraph-sp
```

## Quick Start

```python
import pygraphsp as gs

# Create a graph
graph = gs.Graph()

# Define functions
def data_source(inputs):
    return {"output": [1, 2, 3, 4, 5]}

def multiply_by_2(inputs):
    return {"output": [x * 2 for x in inputs["input"]]}

# Add nodes with simplified API
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

# Auto-connect based on matching port names
graph.auto_connect()

# Execute
executor = gs.Executor()
result = executor.execute(graph)

print(result.get_output("multiply_by_2", "output"))  # [2, 4, 6, 8, 10]
```

## Examples

This directory contains complete Python examples:

- **simple_pipeline.py**: Basic 3-node pipeline with the new simplified API
- **parallel_execution.py**: Fan-out/fan-in with 3 parallel branches (44% speedup)
- **variants_demo.py**: Hyperparameter sweeps and parameter variations
- **port_mapping_demo.py**: Advanced port mapping with broadcast/impl names
- **complex_objects.py**: Nested objects, JSON, and lists
- **branching_example.py**: Branch creation and management

### Running Examples

```bash
cd python_examples

# Simple pipeline with new API
python simple_pipeline.py

# Parallel execution (shows speedup)
python parallel_execution.py

# Hyperparameter sweeps
python variants_demo.py
```

## Features

- ⚡ **True Parallel Execution**: Independent nodes run concurrently (44% faster)
- 🔌 **Simplified API**: `graph.add(function, label=, inputs=, outputs=)`
- 🎯 **Auto-Connect**: Automatically connects nodes with matching port names
- 🔄 **Variants & Sweeps**: Built-in hyperparameter sweep support
- 📊 **Rich Data Types**: Primitives, lists, nested dicts, JSON, binary data
- ✅ **Cycle Detection**: Built-in DAG validation

## API Overview

### Creating Graphs (New Simplified API)

```python
import pygraphsp as gs

# Create a new graph
graph = gs.Graph()

# Define your function
def my_processor(inputs):
    # inputs is a dict with your port names as keys
    value = inputs["input_data"]
    return {"result": value * 2}

# Add node with simplified API
graph.add(
    my_processor,           # Your function
    label="My Processor",   # Display name (optional)
    inputs=["input_data"],  # List of input port names
    outputs=["result"]      # List of output port names
)

# Auto-connect nodes with matching port names
graph.auto_connect()

# Or manually connect if needed
# graph.add_edge("source_fn", "data", "my_processor", "input_data")

# Validate graph (checks for cycles)
graph.validate()
```

**Note:** For advanced cases where parameter names differ from port names, see `port_mapping_demo.py` for explicit mapping examples.

### Executing Graphs

```python
# Create executor
executor = gs.Executor()

# Execute graph (automatically parallelizes independent nodes)
result = executor.execute(graph)

# Get outputs
output_value = result.get_output("my_processor", "result")
print(f"Result: {output_value}")

# Check success
if result.is_success():
    print("✓ Execution successful!")
```

### Graph Analysis

```python
# Analyze graph structure
analysis = gs.Inspector.analyze(graph)
print(f"Nodes: {analysis.node_count}")
print(f"Depth: {analysis.depth}")
print(f"Width: {analysis.width}")

# Generate visualization
mermaid = gs.Inspector.to_mermaid(graph)
print(mermaid)
```

### Variants & Parameter Sweeps

```python
# Create parameter variations
def learning_rate_gen(i):
    rates = [0.001, 0.01, 0.1]
    return gs.PortData(rates[i])

variant_branches = graph.create_variants(
    name_prefix="lr",
    count=3,
    param_name="learning_rate",
    variant_function=learning_rate_gen,
    parallel=True
)

# Merge results from all variants
graph.merge_branches(
    node_id="best_model",
    branches=variant_branches,
    port="accuracy"
)
```

See `variants_demo.py` for complete examples.

## Data Types

All Python types are automatically converted:

| Python Type | graph-sp Type |
|------------|---------------|
| `int` | `Int` |
| `float` | `Float` |
| `str` | `String` |
| `bool` | `Bool` |
| `None` | `None` |
| `list` | `List` |
| `dict` | `Map` |
| JSON-serializable | `Json` |
| `bytes` | `Bytes` |

### Complex Data Structures

```python
# Nested objects work seamlessly
user = {
    "name": "Alice",
    "age": 30,
    "address": {
        "city": "NYC",
        "zip": "10001"
    },
    "hobbies": ["reading", "coding", "hiking"]
}

# Lists of any type
numbers = [1, 2, 3, 4, 5]
mixed = [1, "hello", 3.14, True, None]

# JSON structures
product = {
    "id": "laptop-001",
    "specs": {
        "cpu": "Intel i7",
        "ram": "16GB"
    },
    "available": True,
    "price": 999.99
}
```

## Parallel Execution

The executor automatically identifies and parallelizes independent branches:

```python
import pygraphsp as gs
import time

# Fan-out pattern: 3 branches run in parallel
#
#         source
#        /  |  \
#     slow fast medium    <- Execute concurrently!
#        \  |  /
#        merger
#
# Sequential time: 900ms (500 + 100 + 300)
# Parallel time: 500ms (max branch time)
# Speedup: 44% faster!

graph = gs.Graph()

def source_fn(inputs):
    return {"value": 100}

def slow_branch(inputs):
    time.sleep(0.5)  # 500ms
    return {"a": inputs["input"] * 2}

def fast_branch(inputs):
    time.sleep(0.1)  # 100ms
    return {"b": inputs["input"] + 50}

def medium_branch(inputs):
    time.sleep(0.3)  # 300ms
    return {"c": inputs["input"] // 2}

def merger(inputs):
    return {"result": inputs["a"] + inputs["b"] + inputs["c"]}

# Add nodes with simplified API
graph.add(source_fn, label="Source", outputs=["value"])
graph.add(slow_branch, label="Slow", inputs=["input"], outputs=["a"])
graph.add(fast_branch, label="Fast", inputs=["input"], outputs=["b"])
graph.add(medium_branch, label="Medium", inputs=["input"], outputs=["c"])
graph.add(merger, label="Merger", inputs=["a", "b", "c"], outputs=["result"])

# Auto-connect based on matching port names
graph.auto_connect()

# Execute - branches run in parallel!
executor = gs.Executor()
result = executor.execute(graph)
print(result.get_output("merger", "result"))  # 300
```

## Building from Source

If you want to build from source instead of using PyPI:

```bash
# Clone repository
git clone https://github.com/briday1/graph-sp.git
cd graph-sp

# Install maturin
pip install maturin

# Build and install
maturin develop --release --features python
```

## Documentation

- **Full Documentation**: https://github.com/briday1/graph-sp
- **Port Data Types**: See `docs/PORT_DATA_TYPES.md` in the repository
- **Expected Output**: See `EXPECTED_OUTPUT.md` in this directory

## Performance

Measured with the `parallel_execution.py` example:

- **Sequential execution**: ~900ms
- **Parallel execution**: ~502ms
- **Speedup**: 44% faster

The executor uses Rust's tokio runtime for true concurrent execution while properly managing Python's GIL.

## License

MIT License

## Links

- **GitHub**: https://github.com/briday1/graph-sp
- **PyPI**: https://pypi.org/project/pygraph-sp/
- **Crates.io**: https://crates.io/crates/graph-sp
- **Issues**: https://github.com/briday1/graph-sp/issues
