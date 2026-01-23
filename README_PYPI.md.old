Real run (captured from `/Users/brian.day/git/graph-sp/.venv/bin/python examples/py/python_demo.py`):

```
======================================================================
```
Creating graph...
Adding source node...
Adding processor node...
Adding formatter node...

````markdown
Mermaid (example):

```
graph TD
    0["Source"]
    1["Processor"]
    2["Formatter"]
    0 -->|data → input| 1
    1 -->|final → value| 2
```
```bash
pip install dagex
```

Quick overview:

- Build a graph by adding functions (callables)
- Inputs/outputs are mapped by names (broadcast → function param)
- Branching and variants are supported
- Use `to_mermaid()` to visualize the DAG
- Execution returns a context-like mapping with results

---

## Minimal Python example

```python
import dagex

# Data source
def generate(_):
    return {"n": 7}

# Processor
def double(inputs):
    v = inputs.get("x", 0)
    return {"y": v * 2}

# Build graph
g = dagex.Graph()
g.add(generate, label="Source", inputs=None, outputs=[("n", "x")])
g.add(double, label="Double", inputs=[("x", "x")], outputs=[("y", "out")])

# Visualize and run
dag = g.build()
print('\nMermaid:\n', dag.to_mermaid())
context = dag.execute(parallel=False)
print('Result:', context.get('out'))
```

Expected output:

```
Result: 14
```

---

## Branching & Merging (Python)

```python
import dagex

# source
def src(_):
    return {"base": 50}

# branch functions
def add10(inputs):
    return {"result": inputs.get("x", 0) + 10}

def add20(inputs):
    return {"result": inputs.get("x", 0) + 20}

# graph
g = dagex.Graph()
g.add(src, label='Source', inputs=None, outputs=[('base', 'x')])

b1 = dagex.Graph()
b1.add(add10, label='A', inputs=[('x','x')], outputs=[('result','result')])

b2 = dagex.Graph()
b2.add(add20, label='B', inputs=[('x','x')], outputs=[('result','result')])

id1 = g.branch(b1)
id2 = g.branch(b2)

# merge maps branch-specific result -> local names
g.merge(lambda inputs: {"combined": inputs.get('from_a', 0) + inputs.get('from_b', 0)},
        label='Merge',
        inputs=[(id1, 'result', 'from_a'), (id2, 'result', 'from_b')],
        outputs=[('combined', 'final')])

dag = g.build()
print(dag.to_mermaid())
res = dag.execute(parallel=True)
print('Final:', res.get('final'))
```

Expected output:

```
Final: 130
```

---

## Variants (parameter sweep)

```python
import dagex

# Source
def src(_):
    return {"base": 10}

# Variant function builder (captures factor)
def make_mul(factor):
    def mul(inputs):
        v = inputs.get('x', 0)
        return {'result': v * factor}
    return mul

g = dagex.Graph()
g.add(src, label='Source', inputs=None, outputs=[('base','x')])

factors = [2,3,5]
funcs = [make_mul(f) for f in factors]
# wrap callables appropriately; the Python binding accepts callables directly
g.variants(funcs, label='Multiply', inputs=[('x','x')], outputs=[('result','final')])

dag = g.build()
print(dag.to_mermaid())
ctx = dag.execute(parallel=True)
print('Final:', ctx.get('final'))
```

---

## Notes & Tips

- The Python API mirrors the Rust API closely. When in doubt, consult the Rust README for conceptual diagrams.
- For large data, prefer providing structures that can be shared (e.g. numpy arrays wrapped appropriately) to avoid copies.
- Use `dag.execute(parallel=True, max_threads=4)` to limit thread usage.

---

## Where to get help

- Docs: https://docs.rs/dagex
- Issues / PRs: https://github.com/briday1/graph-sp

---

<p align="center">Built with ❤️ — enjoy composing DAGs!</p>
