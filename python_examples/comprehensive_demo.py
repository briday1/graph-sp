"""
Comprehensive Example: All Features with Mermaid Visualization

This example demonstrates:
- Basic graph creation with add()
- Branch creation and isolation
- Mermaid diagram output

Note: Full merge and variant features are available in Rust API.
Python bindings currently support basic branch operations.
"""

import pygraphsp as gs


def main():
    print("╔══════════════════════════════════════════════════════════════════╗")
    print("║  Graph-SP: Comprehensive Feature Demonstration (Python)         ║")
    print("║  Showing: Basic Graphs, Branching, and Mermaid Diagrams        ║")
    print("╚══════════════════════════════════════════════════════════════════╝\n")

    # ========================================================================
    # EXAMPLE 1: Simple Pipeline with Basic Nodes
    # ========================================================================
    print("═══════════════════════════════════════════════════════════════════")
    print("Example 1: Basic Pipeline (add, edges, mermaid)")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph1 = graph_sp.Graph()

    # Create a simple 3-node pipeline
    def source_fn(inputs):
        return {"data": [10, 20, 30]}

    def processor_fn(inputs):
        data = inputs["input"]
        doubled = [x * 2 for x in data]
        return {"output": doubled}

    def sink_fn(inputs):
        return {}

    graph1.add(
        "source",
        "Data Source",
        [],
        [graph_sp.Port("data", "Output Data")],
        source_fn
    )

    graph1.add(
        "processor",
        "Data Processor",
        [graph_sp.Port("input", "Input Data")],
        [graph_sp.Port("output", "Processed Data")],
        processor_fn
    )

    graph1.add(
        "sink",
        "Result Sink",
        [graph_sp.Port("input", "Final Data")],
        [],
        sink_fn
    )

    graph1.add_edge("source", "data", "processor", "input")
    graph1.add_edge("processor", "output", "sink", "input")

    print(f"📊 Graph Structure:")
    print(f"   Nodes: {graph1.node_count()}")
    print(f"   Edges: {graph1.edge_count()}")
    print()

    print("🎨 Mermaid Diagram:")
    print(graph1.to_mermaid())

    # ========================================================================
    # EXAMPLE 2: Branching - Multiple Isolated Experiments
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("Example 2: Branching (isolated subgraphs)")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph2 = graph_sp.Graph()

    # Create two experimental branches
    graph2.create_branch("experiment_a")
    graph2.create_branch("experiment_b")

    print("📊 Branch Information:")
    print(f"   Branches created: {graph2.branch_names()}")
    print(f"   Has 'experiment_a': {graph2.has_branch('experiment_a')}")
    print(f"   Has 'experiment_b': {graph2.has_branch('experiment_b')}")
    print()

    print("🎨 Main Graph Mermaid (showing branches):")
    print(graph2.to_mermaid())

    # ========================================================================
    # EXAMPLE 3: Multiple Nodes and Complex Connections
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("Example 3: Complex Pipeline with Multiple Nodes")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph3 = graph_sp.Graph()

    # Source node
    def data_source(inputs):
        return {"numbers": [1, 2, 3, 4, 5]}

    # Multiplier node
    def multiplier(inputs):
        numbers = inputs["input"]
        return {"output": [x * 2 for x in numbers]}

    # Sum calculator node
    def sum_calculator(inputs):
        numbers = inputs["input"]
        return {
            "sum": sum(numbers),
            "count": len(numbers)
        }

    # Average calculator node
    def avg_calculator(inputs):
        total = inputs["sum"]
        count = inputs["count"]
        return {"average": total / count if count > 0 else 0}

    graph3.add("source", "Data Source", [], [graph_sp.Port("numbers", "Numbers")], data_source)
    graph3.add("doubler", "Multiplier (x2)", [graph_sp.Port("input", "Input")], [graph_sp.Port("output", "Output")], multiplier)
    graph3.add("summer", "Sum Calculator", [graph_sp.Port("input", "Input")], [graph_sp.Port("sum", "Sum"), graph_sp.Port("count", "Count")], sum_calculator)
    graph3.add("averager", "Average Calculator", [graph_sp.Port("sum", "Sum"), graph_sp.Port("count", "Count")], [graph_sp.Port("average", "Average")], avg_calculator)

    graph3.add_edge("source", "numbers", "doubler", "input")
    graph3.add_edge("doubler", "output", "summer", "input")
    graph3.add_edge("summer", "sum", "averager", "sum")
    graph3.add_edge("summer", "count", "averager", "count")

    print(f"📊 Graph Structure:")
    print(f"   Nodes: {graph3.node_count()}")
    print(f"   Edges: {graph3.edge_count()}")
    print()

    analysis = graph3.analyze()
    print(f"📊 Graph Analysis:")
    print(f"   Depth: {analysis.depth}")
    print(f"   Width: {analysis.width}")
    print(f"   {analysis.summary()}")
    print()

    print("🎨 Mermaid Diagram:")
    print(graph3.to_mermaid())

    # ========================================================================
    # EXAMPLE 4: Fan-out Pattern (Parallel Branches)
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("Example 4: Fan-out Pattern (for parallel execution)")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph4 = graph_sp.Graph()

    # Source
    def source(inputs):
        return {"value": 100}

    # Three parallel branches
    def branch_a(inputs):
        return {"result": inputs["input"] * 2}

    def branch_b(inputs):
        return {"result": inputs["input"] * 3}

    def branch_c(inputs):
        return {"result": inputs["input"] * 4}

    # Merge (just receives inputs)
    def merger(inputs):
        results = []
        for key in ["a", "b", "c"]:
            if key in inputs:
                results.append(inputs[key])
        return {"merged": results}

    graph4.add("source", "Data Source", [], [graph_sp.Port("value", "Value")], source)
    graph4.add("branch_a", "Branch A (×2)", [graph_sp.Port("input", "Input")], [graph_sp.Port("result", "Result")], branch_a)
    graph4.add("branch_b", "Branch B (×3)", [graph_sp.Port("input", "Input")], [graph_sp.Port("result", "Result")], branch_b)
    graph4.add("branch_c", "Branch C (×4)", [graph_sp.Port("input", "Input")], [graph_sp.Port("result", "Result")], branch_c)
    graph4.add("merger", "Merge Results", [graph_sp.Port("a", "From A"), graph_sp.Port("b", "From B"), graph_sp.Port("c", "From C")], [graph_sp.Port("merged", "Merged")], merger)

    graph4.add_edge("source", "value", "branch_a", "input")
    graph4.add_edge("source", "value", "branch_b", "input")
    graph4.add_edge("source", "value", "branch_c", "input")
    graph4.add_edge("branch_a", "result", "merger", "a")
    graph4.add_edge("branch_b", "result", "merger", "b")
    graph4.add_edge("branch_c", "result", "merger", "c")

    print(f"📊 Fan-out Pattern:")
    print(f"   Source: 1 node")
    print(f"   Parallel branches: 3 nodes")
    print(f"   Merge: 1 node")
    print(f"   This pattern enables parallel execution!")
    print()

    print("🎨 Mermaid Diagram:")
    print(graph4.to_mermaid())

    # ========================================================================
    # EXAMPLE 5: Branches with Naming
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("Example 5: Multiple Named Branches (simulation of variants)")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph5 = graph_sp.Graph()

    # Simulate variant-like branches by creating multiple named branches
    learning_rates = ["lr_0", "lr_1", "lr_2", "lr_3", "lr_4"]
    for lr_name in learning_rates:
        graph5.create_branch(lr_name)

    print("📊 Simulated Variant Structure:")
    print(f"   Branches created: {len(graph5.branch_names())}")
    print(f"   Branch names: {graph5.branch_names()}")
    print("   (In Rust, these would be created via create_variants())")
    print()

    print("🎨 Mermaid Diagram (should show '5 variants' if using lr_ pattern):")
    print(graph5.to_mermaid())

    # ========================================================================
    # EXAMPLE 6: Execution
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("Example 6: Execute and Get Results")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph6 = graph_sp.Graph()

    def source6(inputs):
        return {"value": 42}

    def double6(inputs):
        return {"result": inputs["input"] * 2}

    graph6.add("src", "Source", [], [graph_sp.Port("value", "Value")], source6)
    graph6.add("proc", "Doubler", [graph_sp.Port("input", "Input")], [graph_sp.Port("result", "Result")], double6)
    graph6.add_edge("src", "value", "proc", "input")

    print("📊 Executing Graph...")
    executor = graph_sp.Executor()
    result = executor.execute(graph6)

    print(f"   Success: {result.is_success()}")
    print(f"   Source output: {result.get_output('src', 'value')}")
    print(f"   Processor output: {result.get_output('proc', 'result')}")
    print()

    print("🎨 Mermaid Diagram:")
    print(graph6.to_mermaid())

    # ========================================================================
    # Summary
    # ========================================================================
    print("\n╔══════════════════════════════════════════════════════════════════╗")
    print("║  Summary of Features Demonstrated (Python)                      ║")
    print("╚══════════════════════════════════════════════════════════════════╝")
    print()
    print("✅ Basic API: add() for nodes, add_edge() for connections")
    print("✅ Branching: create_branch() for named subgraphs")
    print("✅ Branch queries: has_branch(), branch_names()")
    print("✅ Execution: Executor with get_output() for results")
    print("✅ Analysis: Graph statistics and analysis")
    print("✅ Mermaid: Complete visualization with diagrams")
    print()
    print("📖 Legend:")
    print("   🔵 Blue nodes      = Source nodes (no inputs)")
    print("   🟣 Purple nodes    = Sink nodes (no outputs)")
    print("   🟠 Orange nodes    = Processing nodes")
    print("   🟢 Green hexagons  = Variant groups (when using lr_N naming)")
    print()
    print("Note: Full merge() and create_variants() APIs are available in Rust.")
    print("      Python bindings will be extended in future updates.")
    print()


if __name__ == "__main__":
    main()
