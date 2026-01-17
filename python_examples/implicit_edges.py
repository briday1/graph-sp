"""
Implicit edge mapping example using graph-sp Python bindings.

This example demonstrates how to build graphs WITHOUT explicit add_edge() calls.
Edges are automatically created by matching port names between nodes.
"""

import pygraphsp as gs


def main():
    print("=== graph-sp Python Example: Implicit Edge Mapping ===\n")
    
    graph = graph_sp.Graph()
    
    # Example 1: Simple pipeline with matching port names
    print("Example 1: Simple Pipeline (auto-connected)")
    print("-" * 50)
    
    # Source node with output port "data"
    def source_fn(inputs):
        """Generate initial data"""
        print("[source] Generating data...")
        return {"data": [1, 2, 3, 4, 5]}
    
    # Processor node with input port "data" and output port "result"
    def processor_fn(inputs):
        """Process the data"""
        print("[processor] Processing data...")
        data = inputs["data"]
        return {"result": [x * 2 for x in data]}
    
    # Sink node with input port "result"
    def sink_fn(inputs):
        """Display the result"""
        print("[sink] Receiving result...")
        result = inputs["result"]
        print(f"[sink] Final result: {result}")
        return {}
    
    # Add nodes with matching port names
    graph.add_node(
        "source",
        "Data Source",
        [],  # no inputs
        [graph_sp.Port("data", "Data Output")],
        source_fn
    )
    
    graph.add_node(
        "processor",
        "Data Processor",
        [graph_sp.Port("data", "Data Input")],  # matches source's output port!
        [graph_sp.Port("result", "Result Output")],
        processor_fn
    )
    
    graph.add_node(
        "sink",
        "Result Sink",
        [graph_sp.Port("result", "Result Input")],  # matches processor's output port!
        [],  # no outputs
        sink_fn
    )
    
    # NO add_edge() calls! Auto-connect instead
    edges_created = graph.auto_connect()
    print(f"✓ Auto-connected {edges_created} edges based on port name matching!\n")
    
    # Validate
    print("Validating graph...")
    graph.validate()
    print("✓ Graph is valid (no cycles detected)\n")
    
    # Analyze
    print("=== Graph Analysis ===")
    analysis = graph.analyze()
    print(f"Nodes: {analysis.node_count}")
    print(f"Edges: {analysis.edge_count}")
    print(f"Depth: {analysis.depth}")
    print(f"Summary: {analysis.summary()}\n")
    
    # Visualize
    print("=== Mermaid Diagram ===")
    mermaid = graph.to_mermaid()
    print(mermaid)
    
    # Execute
    print("=== Executing Graph ===")
    executor = graph_sp.Executor()
    executor.execute(graph)
    print("✓ Execution completed!\n")
    
    # Example 2: Parallel branches with implicit connections
    print("\nExample 2: Parallel Branches (auto-connected)")
    print("-" * 50)
    
    graph2 = graph_sp.Graph()
    
    # Source with output "value"
    def source2_fn(inputs):
        print("[source] Generating value...")
        return {"value": 100}
    
    # Branch A: input "value", output "branch_a_out"
    def branch_a_fn(inputs):
        print("[branch_a] Processing...")
        return {"branch_a_out": inputs["value"] * 2}
    
    # Branch B: input "value", output "branch_b_out"
    def branch_b_fn(inputs):
        print("[branch_b] Processing...")
        return {"branch_b_out": inputs["value"] + 50}
    
    # Merger: inputs "branch_a_out" and "branch_b_out", output "final"
    def merger_fn(inputs):
        print("[merger] Merging...")
        a = inputs.get("branch_a_out", 0)
        b = inputs.get("branch_b_out", 0)
        return {"final": a + b}
    
    # Collector: input "final"
    def collector_fn(inputs):
        print(f"[collector] Final result: {inputs['final']}")
        return {}
    
    # Add nodes with matching port names for implicit connections
    graph2.add_node(
        "source",
        "Value Source",
        [],
        [graph_sp.Port("value", "Value")],
        source2_fn
    )
    
    graph2.add_node(
        "branch_a",
        "Branch A\\n(×2)",  # Note: \n will render as line break in Mermaid!
        [graph_sp.Port("value", "Input")],
        [graph_sp.Port("branch_a_out", "Output")],
        branch_a_fn
    )
    
    graph2.add_node(
        "branch_b",
        "Branch B\\n(+50)",  # Multi-line label
        [graph_sp.Port("value", "Input")],
        [graph_sp.Port("branch_b_out", "Output")],
        branch_b_fn
    )
    
    graph2.add_node(
        "merger",
        "Result Merger",
        [
            graph_sp.Port("branch_a_out", "Branch A"),
            graph_sp.Port("branch_b_out", "Branch B")
        ],
        [graph_sp.Port("final", "Final Result")],
        merger_fn
    )
    
    graph2.add_node(
        "collector",
        "Result Collector",
        [graph_sp.Port("final", "Final")],
        [],
        collector_fn
    )
    
    # Auto-connect based on port names
    edges_created2 = graph2.auto_connect()
    print(f"✓ Auto-connected {edges_created2} edges!\n")
    
    # Validate
    graph2.validate()
    print("✓ Graph is valid!\n")
    
    # Show Mermaid with parallel groups and multi-line labels
    print("=== Mermaid Diagram (with parallel groups & multi-line labels) ===")
    mermaid2 = graph2.to_mermaid()
    print(mermaid2)
    
    # Execute
    print("=== Executing Graph ===")
    executor.execute(graph2)
    print("✓ Execution completed!\n")
    
    print("=== Example Complete ===")
    print("\nKey Features Demonstrated:")
    print("  ✓ Implicit edge mapping (no add_edge() needed)")
    print("  ✓ Port name matching for automatic connections")
    print("  ✓ Multi-line labels in Mermaid (\\n → <br/>)")
    print("  ✓ Parallel group detection and visualization")
    print("  ✓ Fan-out/fan-in patterns auto-detected")


if __name__ == "__main__":
    main()
