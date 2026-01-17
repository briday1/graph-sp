"""
Simple pipeline example using graph-sp Python bindings.

This example demonstrates basic usage of the graph-sp library from Python,
showing how to create nodes, connect them, and execute the graph.

To run this example:
1. Install the package: pip install pygraph-sp
2. Run: python simple_pipeline.py
"""

import pygraphsp as gs


def main():
    print("=== graph-sp Python Example: Simple Pipeline ===\n")
    
    # Create a graph
    graph = gs.Graph()

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
    
    # Add nodes using new simplified API
    graph.add(
        source_fn,
        label="Source Node",
        outputs=["output"]
    )

    graph.add(
        double_fn,
        label="Doubler Node",
        inputs=["input"],
        outputs=["output"]
    )
    
    graph.add(
        add_five_fn,
        label="Add Five Node",
        inputs=["input"],
        outputs=["output"]
    )

    # Auto-connect nodes with matching port names
    graph.auto_connect()

    print("✓ Graph built successfully!\n")

    # Validate
    graph.validate()
    print("✓ Graph is valid (no cycles detected)\n")

    # Execute
    print("=== Executing Graph ===")
    executor = gs.Executor()
    result = executor.execute(graph)
    
    print("✓ Execution completed successfully!\n")

    # Get results
    print("=== Results ===")
    source_output = result.get_output("source_fn", "output")
    doubler_output = result.get_output("double_fn", "output")
    adder_output = result.get_output("add_five_fn", "output")
    
    print(f"Source output: {source_output}")
    print(f"After doubling: {doubler_output}")
    print(f"After adding 5: {adder_output}")
    print(f"\nPipeline: 10 -> ×2 -> +5 = {adder_output}")
    print("\n=== Example Complete ===")


if __name__ == "__main__":
    main()
