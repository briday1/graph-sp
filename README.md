# graph-sp

graph-sp is a pure Rust grid/node graph executor and optimizer. The project focuses on representing directed dataflow graphs, computing port mappings by graph inspection, and executing nodes efficiently in-process with parallel CPU execution. This repository currently contains a Rust-only implementation; previous plans for Python bindings have been removed.

Core features:
- Pure Rust library for graph modeling and execution
- Typed ports and explicit connections between nodes
- Graph inspection to compute data dependencies and minimal data movement
- In-process execution with parallelism using rayon
- Designed to be extendable: future additions may include cross-process dispatch, shared-memory IPC, and GPU backends

Quickstart (developer):

Prerequisites:
- Rust (stable toolchain) installed: https://www.rust-lang.org/tools/install

Build and run tests:

    cargo build --release
    cargo test

Run library examples / benches (if available):

    cargo run --example
