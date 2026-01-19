# graph-sp

graph-sp is a pure Rust grid/node graph executor and optimizer. The project focuses on representing directed dataflow graphs, computing port mappings by graph inspection, and executing nodes efficiently in-process with parallel CPU execution.

## Core Features

- **Implicit Node Connections**: Nodes automatically connect based on execution order
- **Parallel Branching**: Create fan-out execution paths with `.branch()`
- **Configuration Variants**: Use `.variant()` to create parameter sweeps
- **DAG Analysis**: Automatic inspection and optimization of execution paths
- **Mermaid Visualization**: Generate diagrams with `.to_mermaid()`
- **In-process Execution**: Parallel execution using rayon

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
graph-sp = "0.1.0"

# Optional: For radar signal processing examples with ndarray and FFT support
[features]
radar_examples = ["graph-sp/radar_examples"]
```

For radar signal processing with ndarray and complex number support, enable the `radar_examples` feature.

### Python

The library can also be used from Python via PyO3 bindings:

```bash
pip install pygraph-sp
```

Or build from source:

```bash
pip install maturin
maturin build --release --features python
pip install target/wheels/pygraph_sp-*.whl
```

## Quick Start

### Rust

#### Basic Sequential Pipeline

```rust
use graph_sp::{Graph, GraphData};
use std::collections::HashMap;

fn data_source(_: &HashMap<String, GraphData>, _: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    result.insert("value".to_string(), GraphData::int(42));
    result
}

fn multiply(inputs: &HashMap<String, GraphData>, _: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    if let Some(val) = inputs.get("x").and_then(|d| d.as_int()) {
        result.insert("doubled".to_string(), GraphData::int(val * 2));
    }
    result
}

fn main() {
    let mut graph = Graph::new();
    
    // Add source node
    graph.add(data_source, Some("DataSource"), None, Some(vec![("value", "data")]));
    
    // Add processing node
    graph.add(multiply, Some("Multiply"), Some(vec![("data", "x")]), Some(vec![("doubled", "result")]));
    
    let dag = graph.build();
    let context = dag.execute(false, None);
    
    println!("Result: {}", context.get("result").unwrap().to_string_repr());
}
```

### Python

#### Basic Sequential Pipeline

```python
import graph_sp

def data_source(inputs, variant_params):
    return {"value": "42"}

def multiply(inputs, variant_params):
    val = int(inputs.get("x", "0"))
    return {"doubled": str(val * 2)}

# Create graph
graph = graph_sp.PyGraph()

# Add source node
graph.add(
    function=data_source,
    label="DataSource",
    inputs=None,
    outputs=[("value", "data")]
)

# Add processing node
graph.add(
    function=multiply,
    label="Multiply",
    inputs=[("data", "x")],
    outputs=[("doubled", "result")]
)

# Build and execute
dag = graph.build()
context = dag.execute()

print(f"Result: {context['result']}")
```

**Mermaid visualization output:**

```mermaid
graph TD
    0["DataSource"]
    1["Multiply"]
    0 -->|data → x| 1
```

### Parallel Branching (Fan-Out)

```rust
let mut graph = Graph::new();

// Source node
graph.add(source_fn, Some("Source"), None, Some(vec![("data", "data")]));

// Create parallel branches
graph.branch();
graph.add(stats_fn, Some("Statistics"), Some(vec![("data", "input")]), Some(vec![("mean", "stats")]));

graph.branch();
graph.add(model_fn, Some("MLModel"), Some(vec![("data", "input")]), Some(vec![("prediction", "model")]));

graph.branch();
graph.add(viz_fn, Some("Visualization"), Some(vec![("data", "input")]), Some(vec![("plot", "viz")]));

let dag = graph.build();
```

**Mermaid visualization output:**

```mermaid
graph TD
    0["Source"]
    1["Statistics"]
    2["MLModel"]
    3["Visualization"]
    0 -->|data → input| 1
    0 -->|data → input| 2
    0 -->|data → input| 3
    style 1 fill:#e1f5ff
    style 2 fill:#e1f5ff
    style 3 fill:#e1f5ff
```

**DAG Statistics:**
- Nodes: 4
- Depth: 2 levels
- Max Parallelism: 3 nodes (all branches execute in parallel)

### Parameter Sweep with Variants

```rust
use graph_sp::{Graph, Linspace};

let mut graph = Graph::new();

// Source node
graph.add(source_fn, Some("DataSource"), None, Some(vec![("value", "data")]));

// Create variants for different learning rates
let learning_rates = vec![0.001, 0.01, 0.1, 1.0];
graph.variant("learning_rate", learning_rates);
graph.add(scale_fn, Some("ScaleLR"), Some(vec![("data", "input")]), Some(vec![("scaled", "output")]));

let dag = graph.build();
```

**Mermaid visualization output:**

```mermaid
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
    style 1 fill:#e1f5ff
    style 2 fill:#e1f5ff
    style 3 fill:#e1f5ff
    style 4 fill:#e1f5ff
    style 1 fill:#ffe1e1
    style 2 fill:#e1ffe1
    style 3 fill:#ffe1ff
    style 4 fill:#ffffe1
```

**DAG Statistics:**
- Nodes: 5
- Depth: 2 levels
- Max Parallelism: 4 nodes
- Variants: 4 (all execute in parallel)

## Radar Signal Processing Example

This example demonstrates a complete radar signal processing pipeline using GraphData with ndarray arrays and complex numbers. The pipeline implements:

1. **LFM Pulse Generation** - Creates a Linear Frequency Modulation chirp signal
2. **Pulse Stacking** - Accumulates multiple pulses with Doppler shifts
3. **Range Compression** - FFT-based matched filtering
4. **Doppler Compression** - Creates Range-Doppler map

### Rust Implementation

```rust
use graph_sp::{Graph, GraphData};
use ndarray::Array1;
use num_complex::Complex;
use std::collections::HashMap;

// LFM pulse generator node
fn lfm_generator(_inputs: &HashMap<String, GraphData>, params: &HashMap<String, GraphData>) 
    -> HashMap<String, GraphData> {
    let num_samples = params.get("num_samples")
        .and_then(|d| d.as_int())
        .unwrap_or(256) as usize;
    
    let bandwidth = params.get("bandwidth")
        .and_then(|d| d.as_float())
        .unwrap_or(100e6); // 100 MHz
    
    let pulse_width = params.get("pulse_width")
        .and_then(|d| d.as_float())
        .unwrap_or(1e-6); // 1 microsecond
    
    // Generate LFM chirp signal
    let sample_rate = 100e6;
    let chirp_rate = bandwidth / pulse_width;
    let mut signal = Array1::<Complex<f64>>::zeros(num_samples);
    
    // ... signal generation code ...
    
    let mut output = HashMap::new();
    output.insert("pulse".to_string(), GraphData::complex_array(signal));
    output.insert("num_samples".to_string(), GraphData::int(num_samples as i64));
    output
}

// Stack pulses node
fn stack_pulses(inputs: &HashMap<String, GraphData>, params: &HashMap<String, GraphData>) 
    -> HashMap<String, GraphData> {
    let num_pulses = params.get("num_pulses")
        .and_then(|d| d.as_int())
        .unwrap_or(128) as usize;
    
    // Get input pulse as ComplexArray
    let pulse = inputs.get("pulse")
        .and_then(|d| d.as_complex_array())
        .unwrap().clone();
    
    // Stack with Doppler shifts
    // ... stacking logic ...
    
    let mut output = HashMap::new();
    output.insert("stacked".to_string(), GraphData::complex_array(stacked_data));
    output.insert("num_pulses".to_string(), GraphData::int(num_pulses as i64));
    output
}

fn main() {
    let mut graph = Graph::new();
    
    // Add LFM generator
    graph.add(
        lfm_generator,
        Some("LFMGenerator"),
        None,
        Some(vec![("pulse", "lfm_pulse"), ("num_samples", "num_samples")])
    );
    
    // Add pulse stacking
    graph.add(
        stack_pulses,
        Some("StackPulses"),
        Some(vec![("lfm_pulse", "pulse")]),
        Some(vec![("stacked", "stacked_data"), ("num_pulses", "num_pulses")])
    );
    
    // Add range compression
    graph.add(
        range_compress,
        Some("RangeCompress"),
        Some(vec![("stacked_data", "data"), ("lfm_pulse", "reference")]),
        Some(vec![("compressed", "compressed_data")])
    );
    
    // Add Doppler compression
    graph.add(
        doppler_compress,
        Some("DopplerCompress"),
        Some(vec![
            ("compressed_data", "data"),
            ("num_pulses", "num_pulses"),
            ("num_samples", "num_samples")
        ]),
        Some(vec![
            ("range_doppler", "range_doppler_map"),
            ("peak_value", "peak"),
            ("peak_doppler_bin", "peak_doppler"),
            ("peak_range_bin", "peak_range")
        ])
    );
    
    let dag = graph.build();
    let context = dag.execute(false, None);
    
    // Display results
    if let Some(peak) = context.get("peak").and_then(|d| d.as_float()) {
        println!("Peak magnitude: {:.2}", peak);
    }
    if let Some(doppler) = context.get("peak_doppler").and_then(|d| d.as_int()) {
        println!("Peak Doppler bin: {}", doppler);
    }
    if let Some(range) = context.get("peak_range").and_then(|d| d.as_int()) {
        println!("Peak Range bin: {}", range);
    }
}
```

**Run the example:**

```bash
cargo run --example radar_demo --features radar_examples
```

**Mermaid visualization output:**

```mermaid
graph TD
    0["LFMGenerator"]
    1["StackPulses"]
    2["RangeCompress"]
    3["DopplerCompress"]
    0 -->|lfm_pulse → pulse| 1
    1 -->|stacked_data → data| 2
    2 -->|compressed_data → data| 3
```

**DAG Statistics:**
- Nodes: 4
- Depth: 4 levels
- Max Parallelism: 1 node

**Execution Output:**

```
LFMGenerator: Generated 256 sample LFM pulse
StackPulses: Stacked 128 pulses with Doppler shifts
RangeCompress: Performed matched filtering on 32768 samples
DopplerCompress: Created Range-Doppler map of shape (128, 256)
  Peak at Doppler bin 13, Range bin 255
  Magnitude: 11974.31

Peak magnitude: 11974.31
Peak Doppler bin: 13
Peak Range bin: 255
```

### Python Implementation

```python
import graph_sp
import numpy as np

def lfm_generator(inputs, variant_params):
    """Generate LFM pulse with rectangular envelope."""
    num_samples = 256
    bandwidth = 100e6  # 100 MHz
    pulse_width = 1e-6  # 1 microsecond
    sample_rate = 100e6
    
    # Generate LFM chirp
    chirp_rate = bandwidth / pulse_width
    signal = np.zeros(num_samples, dtype=complex)
    
    # ... signal generation code ...
    
    # Return numpy array directly (no conversion needed)
    return {
        "pulse": signal,  # Can pass numpy arrays directly
        "num_samples": num_samples
    }

def stack_pulses(inputs, variant_params):
    """Stack multiple pulses with Doppler shifts."""
    num_pulses = 128
    
    # Get pulse data directly as complex array (implicit handling)
    pulse_data = inputs.get("pulse", [])
    pulse = np.array(pulse_data, dtype=complex)
    
    # Stack with Doppler shifts
    # ... stacking logic ...
    
    # Return numpy array directly (no conversion needed)
    return {
        "stacked": stacked,  # Can pass numpy arrays directly
        "num_pulses": num_pulses
    }

# Create graph
graph = graph_sp.PyGraph()

# Add nodes
graph.add(
    function=lfm_generator,
    label="LFMGenerator",
    inputs=None,
    outputs=[("pulse", "lfm_pulse"), ("num_samples", "num_samples")]
)

graph.add(
    function=stack_pulses,
    label="StackPulses",
    inputs=[("lfm_pulse", "pulse")],
    outputs=[("stacked", "stacked_data"), ("num_pulses", "num_pulses")]
)

graph.add(
    function=range_compress,
    label="RangeCompress",
    inputs=[("stacked_data", "data"), ("lfm_pulse", "reference")],
    outputs=[("compressed", "compressed_data")]
)

graph.add(
    function=doppler_compress,
    label="DopplerCompress",
    inputs=[
        ("compressed_data", "data"),
        ("num_pulses", "num_pulses"),
        ("num_samples", "num_samples")
    ],
    outputs=[
        ("range_doppler", "range_doppler_map"),
        ("peak_value", "peak"),
        ("peak_doppler_bin", "peak_doppler"),
        ("peak_range_bin", "peak_range")
    ]
)

# Build and execute
dag = graph.build()
context = dag.execute()

print(f"Peak magnitude: {context['peak']}")
print(f"Peak Doppler bin: {context['peak_doppler']}")
print(f"Peak Range bin: {context['peak_range']}")
```

**Run the example:**

```bash
python examples/python_radar_demo.py
```

### Key Features Demonstrated

- **Native Type Support**: Uses `GraphData::complex_array()` for signal data, `GraphData::int()` for metadata
- **No String Conversions**: Numeric data stays in native format (i64, f64, Complex<f64>)
- **Implicit Complex Number Handling**: Python complex numbers (numpy.complex128, built-in complex) are automatically converted to/from GraphData::Complex without manual real/imag splitting
- **Direct Numpy Array Support**: Pass numpy ndarrays directly without `.tolist()` conversion - automatic detection and conversion
- **Type Safety**: Accessor methods (`.as_complex_array()`, `.as_int()`, `.as_float()`) provide safe type extraction
- **Complex Signal Processing**: Full FFT-based radar processing with ndarray integration

### Adding Plotting Nodes

Plotting and visualization functions can be added as terminal nodes that take input but produce no output:

```rust
fn plot_range_doppler(inputs: &HashMap<String, GraphData>, _params: &HashMap<String, GraphData>) 
    -> HashMap<String, GraphData> {
    // Extract data for plotting
    if let Some(map) = inputs.get("range_doppler").and_then(|d| d.as_complex_array()) {
        // Generate plot (save to file, display, etc.)
        println!("Generating Range-Doppler map plot...");
        // ... plotting code using matplotlib, plotters, etc. ...
    }
    
    // No outputs - this is a terminal/visualization node
    HashMap::new()
}

// Add to graph
graph.add(
    plot_range_doppler,
    Some("PlotRangeDoppler"),
    Some(vec![("range_doppler_map", "range_doppler")]),
    None  // No outputs for visualization nodes
);
```

This pattern allows visualization and logging nodes to be integrated into the pipeline without affecting data flow.

## API Overview

### Rust API

### Graph Construction

- `Graph::new()` - Create a new graph
- `graph.add(fn, name, inputs, outputs)` - Add a node
  - `fn`: Node function with signature `fn(&HashMap<String, GraphData>, &HashMap<String, GraphData>) -> HashMap<String, GraphData>`
  - `name`: Optional node name
  - `inputs`: Optional vector of `(broadcast_var, impl_var)` tuples for input mappings
  - `outputs`: Optional vector of `(impl_var, broadcast_var)` tuples for output mappings
- `graph.branch()` - Create a new parallel branch
- `graph.variant(param_name, values)` - Create parameter sweep variants
- `graph.build()` - Build the DAG

### DAG Operations

- `dag.execute()` - Execute the graph and return execution context
- `dag.stats()` - Get DAG statistics (nodes, depth, parallelism, branches, variants)
- `dag.to_mermaid()` - Generate Mermaid diagram representation

### Python API

The Python bindings provide a similar API with proper GIL handling:

#### Graph Construction

- `PyGraph()` - Create a new graph
- `graph.add(function, label, inputs, outputs)` - Add a node
  - `function`: Python callable with signature `fn(inputs: dict, variant_params: dict) -> dict`
  - `label`: Optional node name (str)
  - `inputs`: Optional list of `(broadcast_var, impl_var)` tuples or dict
  - `outputs`: Optional list of `(impl_var, broadcast_var)` tuples or dict
- `graph.branch(subgraph)` - Create a new parallel branch with a subgraph
- `graph.build()` - Build the DAG and return a PyDag

#### DAG Operations

- `dag.execute()` - Execute the graph and return execution context (dict)
- `dag.execute_parallel()` - Execute with parallel execution where possible (dict)
- `dag.to_mermaid()` - Generate Mermaid diagram representation (str)

#### GIL Handling

The Python bindings are designed with proper GIL handling:

- **GIL Release**: The Rust executor runs without holding the GIL, allowing true parallelism
- **GIL Acquisition**: Python callables used as node functions acquire the GIL only during their execution
- **Thread Safety**: The bindings use `pyo3::prepare_freethreaded_python()` (via auto-initialize) for multi-threaded safety

This means that while Python functions execute sequentially (due to the GIL), the Rust graph traversal and coordination happens in parallel without GIL contention.

## Development

### Rust Development

Prerequisites:
- Rust (stable toolchain) installed: https://www.rust-lang.org/tools/install

Build and run tests:

```bash
cargo build --release
cargo test
```

Run examples:

```bash
cargo run --example comprehensive_demo
cargo run --example parallel_execution_demo
cargo run --example variant_demo_full
cargo run --example radar_demo --features radar_examples
```

### Python Development

Prerequisites:
- Python 3.8+ installed
- Rust toolchain installed

Build Python bindings:

```bash
# Create virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install maturin
pip install maturin==1.2.0

# Build and install in development mode
maturin develop --release --features python

# Run Python example
python examples/python_demo.py
```

Build wheel for distribution:

```bash
maturin build --release --features python
# Wheel will be in target/wheels/
```

## Publishing

This repository is configured with GitHub Actions workflows to automatically publish to [crates.io](https://crates.io) and [PyPI](https://pypi.org) when a release tag is pushed.

### Required Repository Secrets

To enable automatic publishing, the repository owner must configure the following secrets in GitHub Settings → Secrets and variables → Actions:

- **`CRATES_IO_TOKEN`**: Your crates.io API token (obtain from https://crates.io/me)
- **`PYPI_API_TOKEN`**: Your PyPI API token (obtain from https://pypi.org/manage/account/token/)

### Publishing Process

The publish workflow (`.github/workflows/publish.yml`) will automatically run when:

1. A tag matching `v*` is pushed (e.g., `v0.1.0`, `v1.0.0`)
2. The workflow is manually triggered via workflow_dispatch

**Creating a release:**

```bash
# Ensure version numbers in Cargo.toml and pyproject.toml are correct
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

The workflow will:

1. **Build Python wheels** for Python 3.8-3.11 on Linux, macOS, and Windows
2. **Upload wheel artifacts** to the GitHub Actions run (always, even without secrets)
3. **Publish to PyPI** (only if `PYPI_API_TOKEN` is set) - prebuilt wheels mean end users do not need Rust
4. **Publish to crates.io** (only if `CRATES_IO_TOKEN` is set)

**Important notes:**

- Installing from PyPI with `pip install pygraph-sp` will **not require Rust** on the target machine because prebuilt platform-specific wheels are published
- Both crates.io and PyPI will reject duplicate version numbers - update versions before tagging
- The workflow will continue even if tokens are not set, allowing you to download artifacts for manual publishing
- For local testing, you can build wheels with `maturin build --release --features python`

### Manual Publishing

If you prefer to publish manually or need to publish from a local machine:

**To crates.io:**

```bash
cargo publish --token YOUR_CRATES_IO_TOKEN
```

**To PyPI:**

```bash
# Install maturin
pip install maturin==1.2.0

# Build and publish wheels
maturin publish --username __token__ --password YOUR_PYPI_API_TOKEN --features python
```
