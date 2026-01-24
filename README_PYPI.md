# dagex - Python Edition

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

### Example 01: Minimal Pipeline

The simplest possible pipeline: generator → transformer → aggregator.

**Output:**

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

⏱️  Runtime: 0.024ms
💾 Memory: Current: 0.05 KB, Peak: 0.05 KB

────────────────────────────────────────────────────────────
Results
────────────────────────────────────────────────────────────

✅ Final output: 25
   (Started with 10, doubled to 20, added 5 = 25)
```

### Example 02: Parallel vs Sequential Execution

Demonstrates the power of parallel execution.

**Key insight:** Three independent tasks run 3x faster in parallel!

**Output:**

```
⚡ Speedup: 2.98x faster with parallel execution!
```

### Example 03: Branching

Create independent execution paths that run in parallel.

**Mermaid diagram:**

```
graph TD
    0["Source"]
    1["PathA (+10)"]
    2["PathB (+20)"]
    3["Combine"]
    0 -->|x → x| 1
    0 -->|x → x| 2
    1 --> 3
    2 --> 3
```

**Result:** 50 → PathA(60) + PathB(70) → Combined(130)

### Example 04: Variants (Parameter Sweep)

Run multiple parameter configurations in parallel.

**Output:**

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

Access outputs from different parts of the graph.

**Output:**

```
📊 Accessing outputs:

Final context outputs:
   output: 351

Execution flow:
   Source: 100
   ProcessorA (branch A): 100 × 2 = 200
   ProcessorB (branch B): 100 + 50 = 150
   Combine: 200 + 150 + 1 = 351
```

### Example 06: Efficient Data Sharing

Python's reference counting means large data is automatically shared efficiently.

**Output:**

```
⏱️  Runtime: 303.121ms
💾 Memory: Current: 39054.78 KB, Peak: 39062.76 KB

📊 Consumer outputs (each processes different segments):
   ConsumerA (first 1000):  sum = 499500
   ConsumerB (next 1000):   sum = 1499500
   ConsumerC (next 1000):   sum = 2499500

✅ Reference-based data sharing successful!
```

## 🔧 API Reference

### Graph

Create and configure a computational graph:

```python
import dagex

graph = dagex.Graph()

# Add a node
graph.add(
    function,                    # Python callable
    label="NodeName",            # Optional label
    inputs=[("src", "dst")],     # Input mapping: broadcast → impl
    outputs=[("impl", "bcast")]  # Output mapping: impl → broadcast
)

# Create a branch
branch = dagex.Graph()
# ... add nodes to branch ...
branch_id = graph.branch(branch)

# Add variants (parameter sweep)
graph.variants(
    [func1, func2, func3],       # List of callables
    label="Variants",
    inputs=[("input", "x")],
    outputs=[("output", "results")]
)

# Build the DAG
dag = graph.build()
```

### DAG Execution

```python
# Execute the graph
context = dag.execute(
    parallel=True,      # Enable parallel execution
    max_threads=4       # Limit thread count (optional)
)

# Access results
result = context.get("output_name")
```

### Visualization

```python
# Generate Mermaid diagram
mermaid_text = dag.to_mermaid()
print(mermaid_text)
```

## 💡 Tips and Best Practices

### Data Types

- **Integers and floats** are passed efficiently
- **Lists and dicts** are passed by reference (no copying unless you modify them)
- **NumPy arrays** work well for large numerical data

### Performance

- Use `parallel=True` when you have independent nodes at the same level
- Set `max_threads` to control resource usage
- Large data structures are reference-counted, not copied

### Debugging

- Use `to_mermaid()` to visualize your graph structure
- Print intermediate results to understand data flow
- Start with `parallel=False` for easier debugging

## 🔗 Links

- **PyPI Package:** https://pypi.org/project/dagex
- **Repository:** https://github.com/briday1/graph-sp
- **Rust Documentation:** https://docs.rs/dagex
- **Issues:** https://github.com/briday1/graph-sp/issues

## 📄 License

MIT License - see LICENSE file for details.

---

<p align="center">Built with ❤️ — enjoy composing DAGs!</p>
