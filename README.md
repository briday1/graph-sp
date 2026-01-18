# graph-sp

graph-sp is a Rust-based dataflow/graph optimizer engine (MVP) designed to inspect graphs, compute port mappings, and execute nodes in parallel. It's intended to be extensible to cross-process dispatch and GPU backends in the future.

Core features in this repository (MVP):
- Directed dataflow graph with typed ports and explicit connections
- Graph inspection for port mapping and dependency analysis
- In-process execution with parallelism using rayon
- Python bindings via PyO3 and maturin so the library can be built into wheels (.whl)
- A small set of example builtin nodes (Source, Add) and a simple Python usage example

Quickstart (developer):

Prerequisites:
- Rust (stable toolchain) installed: https://www.rust-lang.org/tools/install
- Python 3.8+ and pip
- maturin: pip install maturin

Build a wheel for the active Python environment:

    maturin build --release

Install the built wheel (example):

    pip install target/wheels/graph_sp-0.1.0-cp39-*-manylinux*.whl

Or build and install in editable mode for development:

    maturin develop --release

Python example:

See examples/python_example.py for a short demonstration. In short, the Python binding exposes a Graph object that supports:
- add_source(name)
- add_add(name)  # builtin Add node
- connect(src_node, src_port, dst_node, dst_port)
- set_source_data(node_name, sequence_of_numbers)
- execute() -> dict mapping node_name -> list of outputs (each output is a Python list)

Contributing:
- This repository is an MVP scaffold. We welcome feature branches and PRs. For larger features (GPU backend, IPC/process dispatch), please open an issue to discuss design before a big implementation PR.

License: MIT
