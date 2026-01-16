"""
Simple pipeline example using graph-sp Python bindings.

This example demonstrates basic usage of the graph-sp library from Python,
showing how to create nodes, connect them, and execute the graph.

To run this example:
1. Build the Python bindings: cargo build --release --features python
2. Copy the compiled library to your Python path
3. Run: python simple_pipeline.py
"""

import graph_sp


def main():
    print("=== graph-sp Python Example: Simple Pipeline ===\n")
    
    # Create a graph
    graph = graph_sp.Graph()

    # Define node functions
    def source_fn(inputs):
        """Source node that generates initial value"""
        return {"output": 10}

    def double_fn(inputs):
        """Double node that multiplies input by 2"""
        return {"output": inputs["input"] * 2}
    
    def add_five_fn(inputs):
        """Add node that adds 5 to input"""
        return {"output": inputs["input"] + 5}

    print("Building graph...")
    
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
    
    graph.add_node(
        "adder",
        "Add Five Node",
        [graph_sp.Port("input", "Input")],
        [graph_sp.Port("output", "Output")],
        add_five_fn
    )

    # Connect nodes
    graph.add_edge("source", "output", "doubler", "input")
    graph.add_edge("doubler", "output", "adder", "input")

    print("✓ Graph built successfully!\n")

    # Validate
    graph.validate()
    print("✓ Graph is valid (no cycles detected)\n")

    # Analyze
    analysis = graph.analyze()
    print(f"=== Graph Analysis ===")
    print(f"Node count: {analysis.node_count}")
    print(f"Edge count: {analysis.edge_count}")
    print(f"Depth: {analysis.depth}")
    print(f"Width: {analysis.width}")
    print(f"Summary: {analysis.summary()}\n")

    # Visualize
    print("=== Graph Structure ===")
    visualization = graph.visualize()
    print(visualization)

    # Generate Mermaid diagram
    print("=== Mermaid Diagram ===")
    mermaid = graph.to_mermaid()
    print(mermaid)

    # Execute
    print("=== Executing Graph ===")
    executor = graph_sp.Executor()
    result = executor.execute(graph)
    
    print("✓ Execution completed successfully!\n")

    # Get results
    print("=== Results ===")
    source_output = result.get_output("source", "output")
    doubler_output = result.get_output("doubler", "output")
    adder_output = result.get_output("adder", "output")
    
    print(f"Source output: {source_output}")
    print(f"After doubling: {doubler_output}")
    print(f"After adding 5: {adder_output}")
    print(f"\nPipeline: 10 -> ×2 -> +5 = {adder_output}")
    print("\n=== Example Complete ===")


if __name__ == "__main__":
    main()
