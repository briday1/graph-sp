# Python Examples for graph-sp

This directory contains Python example scripts demonstrating the graph-sp Python bindings.

## Building the Python Bindings

The Python bindings are built using PyO3. To build them:

```bash
# From the project root
cargo build --release --features python
```

This will create a shared library file in `target/release/` with a name like:
- Linux: `libgraph_sp.so`
- macOS: `libgraph_sp.dylib`
- Windows: `graph_sp.dll`

## Installing the Python Module

### Option 1: Copy the shared library

Rename and copy the shared library to make it importable by Python:

```bash
# On Linux
cp target/release/libgraph_sp.so graph_sp.so

# On macOS
cp target/release/libgraph_sp.dylib graph_sp.so

# On Windows
cp target/release/graph_sp.dll graph_sp.pyd
```

Then you can import it in Python scripts in the same directory.

### Option 2: Use maturin (Recommended for development)

Install maturin:
```bash
pip install maturin
```

Build and install in development mode:
```bash
maturin develop --features python
```

This will build and install the module directly into your current Python environment.

## Running the Examples

Once the Python bindings are installed:

```bash
python python_examples/simple_pipeline.py
```

## Example Output

The Python examples should produce output similar to the Rust examples, demonstrating:
- Graph construction and validation
- Node execution with data flow
- Result retrieval and display

## Current Limitation

**Note**: The current implementation has the Python bindings code in place, but the async executor pattern used (`executor.execute()` returning a future) requires additional setup with `pyo3-asyncio` to work properly with Python's asyncio. The bindings compile successfully but may require a synchronous wrapper or async runtime integration for full Python compatibility.

The Rust API is fully functional and tested. The Python bindings serve as a foundation for Python integration and can be extended with proper async support in future updates.

## Available Examples

- `simple_pipeline.py` - Basic pipeline showing node creation, connection, and execution

## Requirements

- Python 3.7+
- Rust toolchain (for building)
- PyO3 (included in Cargo.toml)
