#!/usr/bin/env python3
"""
Build README files from actual demo outputs.

This script:
1. Runs all examples (Rust & Python)
2. Captures their outputs
3. Parses outputs for diagrams, metrics, and results
4. Generates README.md and README_PYPI.md with embedded outputs
"""

import subprocess
import sys
import re
from pathlib import Path
from typing import Dict, List, Tuple, Optional


class ExampleOutput:
    """Holds parsed output from an example."""
    
    def __init__(self, name: str):
        self.name = name
        self.mermaid_diagram = ""
        self.performance_sequential = ""
        self.performance_parallel = ""
        self.results = ""
        self.speedup = ""


def run_rust_example(example_name: str, repo_root: Path) -> str:
    """Run a Rust example and capture its output."""
    print(f"  Running Rust example: {example_name}")
    try:
        result = subprocess.run(
            ["cargo", "run", "--example", example_name, "--release"],
            cwd=repo_root,
            capture_output=True,
            text=True,
            timeout=120
        )
        # Only return stdout, not stderr which contains cargo build messages
        return result.stdout
    except subprocess.TimeoutExpired:
        print(f"    WARNING: {example_name} timed out")
        return ""
    except Exception as e:
        print(f"    ERROR: Failed to run {example_name}: {e}")
        return ""


def run_python_example(example_file: Path, repo_root: Path) -> str:
    """Run a Python example and capture its output."""
    print(f"  Running Python example: {example_file.name}")
    try:
        result = subprocess.run(
            ["python3", str(example_file)],
            cwd=repo_root,
            capture_output=True,
            text=True,
            timeout=120
        )
        # Only return stdout, not stderr
        return result.stdout
    except subprocess.TimeoutExpired:
        print(f"    WARNING: {example_file.name} timed out")
        return ""
    except Exception as e:
        print(f"    ERROR: Failed to run {example_file.name}: {e}")
        return ""


def extract_mermaid_diagram(output: str) -> str:
    """Extract Mermaid diagram from output."""
    # Look for content between "Mermaid Diagram" and the next section separator
    pattern = r"Mermaid Diagram\s*─+\s*\n(.*?)(?:\n─+|\n═+|\Z)"
    match = re.search(pattern, output, re.DOTALL)
    if match:
        content = match.group(1).strip()
        # Remove any leading/trailing graph TD if present
        lines = content.split('\n')
        # Filter out empty lines and keep only the graph content
        lines = [line.strip() for line in lines if line.strip()]
        return '\n'.join(lines)
    return ""


def extract_performance(output: str, execution_type: str) -> Tuple[str, str]:
    """Extract performance metrics and speedup."""
    # Look for Sequential or Parallel execution section
    # Need to match the section header with dashes below it
    if execution_type == "sequential":
        # Try both patterns with and without "(parallel=...)"
        patterns = [
            r"─+\s*\nSequential Execution\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)",
            r"─+\s*\nSequential Execution \(parallel=false\)\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)",
            r"─+\s*\nSequential Execution \(parallel=False\)\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)",
            r"─+\s*\nSequential \(parallel=false\)\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)",
            r"─+\s*\nSequential \(parallel=False\)\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)"
        ]
    else:
        patterns = [
            r"─+\s*\nParallel Execution\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)",
            r"─+\s*\nParallel Execution \(parallel=true\)\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)",
            r"─+\s*\nParallel Execution \(parallel=True\)\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)",
            r"─+\s*\nParallel \(parallel=true\)\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)",
            r"─+\s*\nParallel \(parallel=True\)\s*\n─+\s*\n(.*?)(?:\n─+|\n═+|\Z)"
        ]
    
    for pattern in patterns:
        match = re.search(pattern, output, re.DOTALL | re.IGNORECASE)
        if match:
            content = match.group(1).strip()
            # Extract just the performance lines (Runtime, Memory, Speedup)
            perf_lines = []
            speedup = ""
            for line in content.split('\n'):
                line = line.strip()
                if line.startswith('⏱️') or line.startswith('💾') or line.startswith('⚡'):
                    if line.startswith('⚡'):
                        speedup = line
                    perf_lines.append(line)
            return '\n'.join(perf_lines), speedup
    return "", ""


def extract_results(output: str) -> str:
    """Extract results section from output."""
    # Look for Results section
    pattern = r"Results\s*─+\s*\n(.*?)(?:\n═+|\Z)"
    match = re.search(pattern, output, re.DOTALL)
    if match:
        content = match.group(1).strip()
        # Clean up the results section
        lines = []
        for line in content.split('\n'):
            line = line.strip()
            # Skip empty lines, section separators, and cargo/compiler output
            if line and not line.startswith('─') and not line.startswith('═'):
                # Skip cargo build output
                if not line.startswith('Compiling') and \
                   not line.startswith('Finished') and \
                   not line.startswith('Running') and \
                   not 'target/release' in line:
                    lines.append(line)
        return '\n'.join(lines)
    return ""


def parse_example_output(output: str, example_name: str) -> ExampleOutput:
    """Parse example output to extract relevant sections."""
    result = ExampleOutput(example_name)
    
    result.mermaid_diagram = extract_mermaid_diagram(output)
    result.performance_sequential, _ = extract_performance(output, "sequential")
    result.performance_parallel, result.speedup = extract_performance(output, "parallel")
    result.results = extract_results(output)
    
    return result


def build_example_section(
    example_num: int,
    title: str,
    description: str,
    syntax_block: str,
    language: str,
    parsed_output: ExampleOutput
) -> str:
    """Build a markdown section for an example."""
    section = f"""### Example {example_num:02d}: {title}

{description}

**Syntax:**
```{language}
{syntax_block}
```

**Mermaid Diagram:**
```mermaid
{parsed_output.mermaid_diagram}
```

**Performance (Sequential):**
```
{parsed_output.performance_sequential}
```

**Performance (Parallel):**
```
{parsed_output.performance_parallel}
```

**Output:**
```
{parsed_output.results}
```

"""
    return section


def get_example_metadata() -> List[Tuple[int, str, str]]:
    """Return metadata for each example: (number, title, description)."""
    return [
        (1, "Minimal Pipeline", "The simplest possible DAG: generator → transformer → aggregator.\n\n**Description:**\nShows a basic 3-node pipeline where each node depends on the previous one. Demonstrates the fundamental dataflow concept."),
        (2, "Parallel vs Sequential Execution", "Demonstrates the power of parallel execution for independent tasks.\n\n**Description:**\nShows three independent tasks (A, B, C) that each simulate I/O-bound work. When executed sequentially, tasks run one after another. When executed in parallel, independent tasks run simultaneously, demonstrating significant speedup."),
        (3, "Branch and Merge", "Fan-out (branching) and fan-in (merging) patterns for complex workflows.\n\n**Description:**\nDemonstrates creating independent branches that process data in parallel, then merging their outputs. Each branch contains its own subgraph that can have multiple nodes."),
        (4, "Variants (Parameter Sweep)", "Run multiple variants in parallel—perfect for hyperparameter tuning or A/B testing.\n\n**Description:**\nDemonstrates running multiple nodes with the same structure but different parameters. All variants execute at the same level in the DAG, enabling efficient parallel exploration of parameter spaces."),
        (5, "Output Access", "Access intermediate results and branch outputs, not just final values.\n\n**Description:**\nDemonstrates how to access different levels of output: final context outputs, individual node outputs, and branch-specific outputs. Uses `execute_detailed()` instead of `execute()` to get comprehensive execution information."),
        (6, "Zero-Copy Data Sharing", "Large data is automatically wrapped in `Arc` for efficient sharing without copying.\n\n**Description:**\nDemonstrates efficient memory handling for large datasets. GraphData automatically wraps large vectors (int_vec, float_vec) in Arc, enabling multiple nodes to read the same data without duplication.")
    ]


def get_rust_syntax_blocks() -> Dict[int, str]:
    """Return syntax examples for Rust."""
    return {
        1: """use dagex::{Graph, GraphData};

let mut graph = Graph::new();

// Functions are automatically wrapped for thread-safe parallel execution
graph.add(
    generate,  // Just pass the function directly
    Some("Generator"),
    None,
    Some(vec![("number", "x")])
);""",
        2: """use dagex::{Graph, GraphData};

// All tasks are automatically wrapped for thread-safe parallel execution
graph.add(task_a, Some("TaskA"), /* ... */);
graph.add(task_b, Some("TaskB"), /* ... */);
graph.add(task_c, Some("TaskC"), /* ... */);

// Execute with parallel=false or parallel=true
let context_seq = dag.execute(false, None);  // Sequential
let context_par = dag.execute(true, Some(4)); // Parallel with 4 threads""",
        3: """use dagex::{Graph, GraphData};

// Create branches
let mut branch_a = Graph::new();
branch_a.add(path_a, Some("PathA (+10)"), /* ... */);
let branch_a_id = graph.branch(branch_a);

let mut branch_b = Graph::new();
branch_b.add(path_b, Some("PathB (+20)"), /* ... */);
let branch_b_id = graph.branch(branch_b);

// Merge branches - combine outputs from multiple branches
graph.merge(
    merge_function,  // Function automatically wrapped for thread safety
    Some("Merge"),
    vec![
        (branch_a_id, "result", "from_a"),
        (branch_b_id, "result", "from_b"),
    ],
    Some(vec![("combined", "final")])
);""",
        4: """use dagex::{Graph, GraphData};

// Factory function to create variants with different parameters
fn make_multiplier(factor: i64) -> impl Fn(&HashMap<String, GraphData>) -> HashMap<String, GraphData> + Send + Sync + 'static {
    move |inputs: &HashMap<String, GraphData>| {
        let value = inputs.get("x").and_then(|d| d.as_int()).unwrap_or(0);
        let mut outputs = HashMap::new();
        outputs.insert("result".to_string(), GraphData::int(value * factor));
        outputs
    }
}

// Create multiple variants
let factors = vec![2, 3, 5, 7];
let variant_nodes: Vec<_> = factors.iter()
    .map(|&f| make_multiplier(f))
    .collect();

// Add all variants at once - functions automatically wrapped for thread safety
graph.variants(
    variant_nodes,
    Some("Multiplier"),
    Some(vec![("x", "x")]),
    Some(vec![("result", "results")])
);""",
        5: """use dagex::{Graph, GraphData};

// Execute with detailed output
let result = dag.execute_detailed(true, Some(4));

// Access different output levels:
// 1. Final context outputs (global broadcast space)
let final_output = result.context.get("output");

// 2. Per-node outputs (each node's raw output)
for (node_id, outputs) in result.node_outputs.iter() {
    println!("Node {}: {} outputs", node_id, outputs.len());
}

// 3. Branch-specific outputs (scoped to branches)
for (branch_id, outputs) in result.branch_outputs.iter() {
    println!("Branch {}: {:?}", branch_id, outputs);
}""",
        6: """use dagex::{Graph, GraphData};

// Create large data - automatically wrapped in Arc by GraphData::int_vec
fn create_large_data(_inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let large_vec: Vec<i64> = (0..1_000_000).collect();
    let mut outputs = HashMap::new();
    // int_vec automatically wraps the Vec in Arc for zero-copy sharing
    outputs.insert("large_data".to_string(), GraphData::int_vec(large_vec));
    outputs
}

// Functions are automatically wrapped for thread safety
graph.add(create_large_data, Some("CreateLargeData"), /* ... */);

// Multiple consumers access the same Arc<Vec<i64>> - no copying!
graph.add(consumer_a, Some("ConsumerA"), /* ... */);
graph.add(consumer_b, Some("ConsumerB"), /* ... */);
graph.add(consumer_c, Some("ConsumerC"), /* ... */);"""
    }


def get_python_syntax_blocks() -> Dict[int, str]:
    """Return syntax examples for Python."""
    return {
        1: """import dagex

graph = dagex.Graph()

# Add nodes to the pipeline
graph.add(
    generate,                    # Python callable
    label="Generator",
    inputs=None,                 # No inputs (source node)
    outputs=[("number", "x")]    # Output mapping: impl → broadcast
)

graph.add(
    double,
    label="Doubler",
    inputs=[("x", "x")],         # Input mapping: broadcast → impl
    outputs=[("result", "y")]
)

# Build and execute
dag = graph.build()
context = dag.execute(parallel=False)  # Sequential
context = dag.execute(parallel=True)   # Parallel""",
        2: """import dagex

# Add independent tasks
graph.add(task_a, label="TaskA", inputs=[("input", "input")], outputs=[("result_a", "a")])
graph.add(task_b, label="TaskB", inputs=[("input", "input")], outputs=[("result_b", "b")])
graph.add(task_c, label="TaskC", inputs=[("input", "input")], outputs=[("result_c", "c")])

# Build and execute
dag = graph.build()

# Sequential vs parallel
context_seq = dag.execute(parallel=False)
context_par = dag.execute(parallel=True, max_threads=4)""",
        3: """import dagex

# Create branches
branch_a = dagex.Graph()
branch_a.add(path_a_func, label="PathA (+10)", ...)
branch_a_id = graph.branch(branch_a)

branch_b = dagex.Graph()
branch_b.add(path_b_func, label="PathB (+20)", ...)
branch_b_id = graph.branch(branch_b)

# Merge branches
graph.merge(
    merge_func,
    label="Merge",
    branch_inputs=[
        (branch_a_id, "result", "from_a"),
        (branch_b_id, "result", "from_b"),
    ],
    outputs=[("combined", "final")]
)""",
        4: """import dagex

# Create variant functions with different parameters
def make_multiplier(factor):
    def multiplier(inputs):
        value = inputs.get("x", 0)
        return {"result": value * factor}
    return multiplier

# Create multiple variants
factors = [2, 3, 5, 7]
variant_funcs = [make_multiplier(f) for f in factors]

# Add all variants at once
graph.variants(
    variant_funcs,
    label="Multiplier",
    inputs=[("x", "x")],
    outputs=[("result", "results")]
)""",
        5: """import dagex

# Execute with detailed output
result = dag.execute_detailed(parallel=True, max_threads=4)

# Access different output levels:
# 1. Final context outputs
final_output = result.context.get("output")

# 2. Per-node outputs
for node_id, outputs in result.node_outputs.items():
    print(f"Node {node_id}: {len(outputs)} outputs")

# 3. Branch-specific outputs
for branch_id, outputs in result.branch_outputs.items():
    print(f"Branch {branch_id}: {outputs}")""",
        6: """import dagex
import numpy as np

# Create large data
def create_large_data(_inputs):
    # Large numpy array - efficiently shared
    large_array = list(range(1_000_000))
    return {"large_data": large_array}

graph.add(create_large_data, label="CreateLargeData", ...)

# Multiple consumers access the same data - minimal copying
graph.add(consumer_a, label="ConsumerA", ...)
graph.add(consumer_b, label="ConsumerB", ...)
graph.add(consumer_c, label="ConsumerC", ...)"""
    }


def generate_readme_md(parsed_outputs: Dict[str, ExampleOutput], repo_root: Path) -> str:
    """Generate the complete README.md content."""
    
    # Header section (preserved from original)
    header = '''<div align="center">
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

fn main() {
    let mut graph = Graph::new();
    
    // Add a data source - functions are automatically wrapped for thread safety
    graph.add(
        |_| {
            let mut out = HashMap::new();
            out.insert("value".to_string(), GraphData::int(10));
            out
        },
        Some("Source"),
        None,
        Some(vec![("value", "x")])
    );
    
    // Add a processor
    graph.add(
        |inputs: &HashMap<String, GraphData>| {
            let v = inputs.get("x").and_then(|d| d.as_int()).unwrap_or(0);
            let mut out = HashMap::new();
            out.insert("result".to_string(), GraphData::int(v * 2));
            out
        },
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

'''
    
    # Build example sections
    examples_section = ""
    metadata = get_example_metadata()
    syntax_blocks = get_rust_syntax_blocks()
    
    for num, title, description in metadata:
        example_key = f"rs_{num:02d}"
        if example_key in parsed_outputs:
            examples_section += build_example_section(
                num, title, description,
                syntax_blocks[num], "rust",
                parsed_outputs[example_key]
            )
    
    # Core API section (preserved from original)
    api_section = '''
## 🔧 Core API

### Graph Builder

```rust
use dagex::{Graph, GraphData};

let mut graph = Graph::new();

// Add a node - function is automatically wrapped for thread-safe parallel execution
graph.add(
    function,                // Function (automatically wrapped in Arc internally)
    Some("NodeLabel"),       // Optional label
    Some(vec![("in", "x")]), // Input mapping: broadcast → impl
    Some(vec![("out", "y")]) // Output mapping: impl → broadcast
);

// Create a branch
let branch_id = graph.branch(subgraph);

// Merge branches - function is automatically wrapped for thread safety
graph.merge(
    merge_function,
    Some("Merge"),
    vec![(branch_id_a, "out_a", "in_a"), (branch_id_b, "out_b", "in_b")],
    Some(vec![("result", "final")])
);

// Add variants (parameter sweep) - functions automatically wrapped
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

<div align="center">Built with ❤️ in Rust — star the repo if you find it useful!</div>'''
    
    return header + examples_section + api_section


def generate_readme_pypi(parsed_outputs: Dict[str, ExampleOutput], repo_root: Path) -> str:
    """Generate the complete README_PYPI.md content."""
    
    # Header section (preserved from original)
    header = '''# dagex - Python Edition

A pure Rust DAG executor with Python bindings for building and executing complex computational workflows.

## 🚀 Quick Start

```bash
pip install dagex
```

## 📖 Overview

**dagex** provides a powerful yet simple API for building directed acyclic graphs (DAGs) of computational tasks. Key features:

- **Automatic dependency resolution** based on data flow
- **Parallel execution** of independent nodes
- **Branching** for creating independent subgraphs
- **Variants** for parameter sweeps and A/B testing
- **Mermaid diagrams** for visualizing your pipeline

### Python Parallel Execution & the GIL

Python's Global Interpreter Lock (GIL) means that pure Python computations cannot achieve true parallelism. However, **dagex enables true parallel execution** when your node functions perform operations that release the GIL, such as:

- **I/O operations**: File reads/writes, network calls, database queries
- **NumPy/SciPy operations**: Most numerical computations in these libraries release the GIL
- **C extensions**: Custom C/Rust extensions that release the GIL
- **Sleep/wait operations**: Simulating blocking operations

The examples in this package use `time.sleep()` to demonstrate parallelization benefits, as sleep operations release the GIL and allow other threads to run concurrently.

## 🎯 Basic Example

```python
import dagex

def generate(_inputs):
    return {"n": 7}

def double(inputs):
    v = inputs.get("x", 0)
    return {"y": v * 2}

# Build graph
g = dagex.Graph()
g.add(generate, label="Source", inputs=None, outputs=[("n", "x")])
g.add(double, label="Double", inputs=[("x", "x")], outputs=[("y", "out")])

# Execute
dag = g.build()
print(dag.to_mermaid())  # Visualize
context = dag.execute(parallel=False)
print('Result:', context.get('out'))  # Result: 14
```

## 📚 Examples

All examples can be run directly:

```bash
python3 examples/py/01_minimal_pipeline.py
python3 examples/py/02_parallel_vs_sequential.py
python3 examples/py/03_branch_and_merge.py
python3 examples/py/04_variants_sweep.py
python3 examples/py/05_output_access.py
python3 examples/py/06_graphdata_large_payload_arc_or_shared_data.py
```

'''
    
    # Build example sections
    examples_section = ""
    metadata = get_example_metadata()
    syntax_blocks = get_python_syntax_blocks()
    
    for num, title, description in metadata:
        example_key = f"py_{num:02d}"
        if example_key in parsed_outputs:
            examples_section += build_example_section(
                num, title, description,
                syntax_blocks[num], "python",
                parsed_outputs[example_key]
            )
    
    # API section for Python
    api_section = '''
## 🔧 Python API

### Building a Graph

```python
import dagex

# Create graph
graph = dagex.Graph()

# Add a node
graph.add(
    function,                       # Python callable
    label="NodeLabel",              # Optional label
    inputs=[("broadcast", "impl")], # Input mapping
    outputs=[("impl", "broadcast")] # Output mapping
)

# Create branches
branch_graph = dagex.Graph()
# ... add nodes to branch_graph ...
branch_id = graph.branch(branch_graph)

# Merge branches
graph.merge(
    merge_function,
    label="Merge",
    branch_inputs=[
        (branch_id_a, "out_a", "in_a"),
        (branch_id_b, "out_b", "in_b")
    ],
    outputs=[("result", "final")]
)

# Add variants
graph.variants(
    [func1, func2, func3],
    label="Variants",
    inputs=[("input", "x")],
    outputs=[("output", "results")]
)

# Build and execute
dag = graph.build()
context = dag.execute(parallel=False)
context = dag.execute(parallel=True, max_threads=4)
```

### Data Types

Python values are automatically converted to GraphData:

```python
# Return Python dictionaries from node functions
def my_node(inputs):
    value = inputs.get("x", 0)  # Access inputs
    return {
        "int_val": 42,
        "float_val": 3.14,
        "str_val": "hello",
        "list_val": [1, 2, 3],
        "nested": {"a": 1, "b": 2}
    }
```

### Execution

```python
# Simple execution
context = dag.execute(parallel=False)  # Sequential
context = dag.execute(parallel=True, max_threads=4)  # Parallel

# Access results
result = context.get("output_name")

# Detailed execution
result = dag.execute_detailed(parallel=True, max_threads=4)
final_context = result.context
node_outputs = result.node_outputs
branch_outputs = result.branch_outputs
```

## 📄 License

MIT License

## 🔗 Links

- **Python Package:** https://pypi.org/project/dagex
- **Documentation:** https://docs.rs/dagex
- **Repository:** https://github.com/briday1/graph-sp
- **Rust Crate:** https://crates.io/crates/dagex'''
    
    return header + examples_section + api_section


def main():
    """Main script execution."""
    print("═" * 60)
    print("  Building READMEs from Demo Outputs")
    print("═" * 60)
    print()
    
    # Get repository root
    repo_root = Path(__file__).parent.parent
    print(f"Repository root: {repo_root}")
    print()
    
    # Collect all outputs
    parsed_outputs: Dict[str, ExampleOutput] = {}
    
    # Run Rust examples
    print("Running Rust examples...")
    print("─" * 60)
    for i in range(1, 7):
        example_name = [
            "01_minimal_pipeline",
            "02_parallel_vs_sequential",
            "03_branch_and_merge",
            "04_variants_sweep",
            "05_output_access",
            "06_graphdata_large_payload_arc_or_shared_data"
        ][i - 1]
        
        output = run_rust_example(example_name, repo_root)
        if output:
            parsed = parse_example_output(output, example_name)
            parsed_outputs[f"rs_{i:02d}"] = parsed
    
    print()
    
    # Setup Python environment first
    print("Setting up Python environment...")
    print("─" * 60)
    try:
        # Install the package in development mode
        subprocess.run(
            ["pip", "install", "-e", "."],
            cwd=repo_root,
            check=True,
            capture_output=True
        )
        print("  Python package installed successfully")
    except subprocess.CalledProcessError as e:
        print(f"  Warning: Failed to install Python package: {e}")
        print("  Python examples may fail")
    
    print()
    
    # Run Python examples
    print("Running Python examples...")
    print("─" * 60)
    for i in range(1, 7):
        example_name = [
            "01_minimal_pipeline",
            "02_parallel_vs_sequential",
            "03_branch_and_merge",
            "04_variants_sweep",
            "05_output_access",
            "06_graphdata_large_payload_arc_or_shared_data"
        ][i - 1]
        
        example_file = repo_root / "examples" / "py" / f"{example_name}.py"
        if example_file.exists():
            output = run_python_example(example_file, repo_root)
            if output:
                parsed = parse_example_output(output, example_name)
                parsed_outputs[f"py_{i:02d}"] = parsed
    
    print()
    
    # Generate READMEs
    print("Generating README files...")
    print("─" * 60)
    
    readme_md = generate_readme_md(parsed_outputs, repo_root)
    readme_pypi = generate_readme_pypi(parsed_outputs, repo_root)
    
    # Write files
    readme_path = repo_root / "README.md"
    readme_pypi_path = repo_root / "README_PYPI.md"
    
    with open(readme_path, 'w') as f:
        f.write(readme_md)
    print(f"  ✅ Generated: {readme_path}")
    
    with open(readme_pypi_path, 'w') as f:
        f.write(readme_pypi)
    print(f"  ✅ Generated: {readme_pypi_path}")
    
    print()
    print("═" * 60)
    print("  README generation complete!")
    print("═" * 60)


if __name__ == "__main__":
    main()
