"""
Comprehensive Example: Showing Both Implicit and Explicit Edge Mapping (Python)

This example demonstrates ALL currently available features in Python:
- Implicit edge mapping (if implemented in Python bindings)
- Explicit edge mapping with add_edge()
- Branching
- Complete mermaid diagram output

Note: Variants and merge features are available in Rust.
Python bindings currently support basic operations and will be extended.
"""

import graph_sp


def main():
    print("╔══════════════════════════════════════════════════════════════════╗")
    print("║  Graph-SP: Complete Feature Demonstration (Python)              ║")
    print("║  Edge Mapping + Branching + Mermaid Diagrams                    ║")
    print("╚══════════════════════════════════════════════════════════════════╝\n")

    # ========================================================================
    # PART 1: EXPLICIT EDGE MAPPING (Current Python API)
    # ========================================================================
    print("═══════════════════════════════════════════════════════════════════")
    print("PART 1: EXPLICIT EDGE MAPPING (add_edge required)")
    print("═══════════════════════════════════════════════════════════════════\n")

    print("In Python, you specify connections explicitly with add_edge()\n")

    graph1 = graph_sp.Graph()

    # Example 1: Simple pipeline with explicit edges
    print("Example 1: Pipeline with explicit edge specification")
    print("----------------------------------------------------")

    def source_fn(inputs):
        return {"data": 42}

    def processor_fn(inputs):
        return {"result": inputs["data"] * 2}

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
        [graph_sp.Port("data", "Input Data")],
        [graph_sp.Port("result", "Processed")],
        processor_fn
    )

    graph1.add(
        "sink",
        "Result Sink",
        [graph_sp.Port("result", "Final")],
        [],
        sink_fn
    )

    # Explicitly connect the nodes
    graph1.add_edge("source", "data", "processor", "data")
    graph1.add_edge("processor", "result", "sink", "result")

    print(f"Added 3 nodes")
    print(f"✅ Edges explicitly added: {graph1.edge_count()}")
    print(f"   source (data) -> processor (data)")
    print(f"   processor (result) -> sink (result)")
    print()

    print("🎨 Mermaid Diagram:")
    print(graph1.to_mermaid())

    # ========================================================================
    # PART 2: COMPLEX PIPELINE
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("PART 2: COMPLEX PIPELINE (Multiple connections)")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph2 = graph_sp.Graph()

    def data_source(inputs):
        return {"numbers": [1, 2, 3, 4, 5]}

    def doubler(inputs):
        return {"doubled": [x * 2 for x in inputs["numbers"]]}

    def summer(inputs):
        nums = inputs["doubled"]
        return {
            "sum": sum(nums),
            "count": len(nums)
        }

    def averager(inputs):
        return {"average": inputs["sum"] / inputs["count"] if inputs["count"] > 0 else 0}

    graph2.add("src", "Data Source", [], [graph_sp.Port("numbers", "Numbers")], data_source)
    graph2.add("dbl", "Doubler", [graph_sp.Port("numbers", "Input")], [graph_sp.Port("doubled", "Doubled")], doubler)
    graph2.add("sum", "Summer", [graph_sp.Port("doubled", "Input")], [graph_sp.Port("sum", "Sum"), graph_sp.Port("count", "Count")], summer)
    graph2.add("avg", "Averager", [graph_sp.Port("sum", "Sum"), graph_sp.Port("count", "Count")], [graph_sp.Port("average", "Average")], averager)

    # Connect the pipeline
    graph2.add_edge("src", "numbers", "dbl", "numbers")
    graph2.add_edge("dbl", "doubled", "sum", "doubled")
    graph2.add_edge("sum", "sum", "avg", "sum")
    graph2.add_edge("sum", "count", "avg", "count")

    print(f"📊 Graph Structure:")
    print(f"   Nodes: {graph2.node_count()}")
    print(f"   Edges: {graph2.edge_count()}")
    print()

    analysis = graph2.analyze()
    print(f"📊 Analysis:")
    print(f"   Depth: {analysis.depth}")
    print(f"   Width: {analysis.width}")
    print(f"   {analysis.summary()}")
    print()

    print("🎨 Mermaid Diagram:")
    print(graph2.to_mermaid())

    # ========================================================================
    # PART 3: FAN-OUT PATTERN (Parallel Execution)
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("PART 3: FAN-OUT PATTERN (Parallel branches)")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph3 = graph_sp.Graph()

    def source3(inputs):
        return {"value": 100}

    def branch_a(inputs):
        return {"result": inputs["input"] * 2}

    def branch_b(inputs):
        return {"result": inputs["input"] * 3}

    def branch_c(inputs):
        return {"result": inputs["input"] * 4}

    def merger(inputs):
        results = []
        for key in ["a", "b", "c"]:
            if key in inputs:
                results.append(inputs[key])
        return {"merged": results}

    graph3.add("source", "Source", [], [graph_sp.Port("value", "Value")], source3)
    graph3.add("branch_a", "Branch A (×2)", [graph_sp.Port("input", "Input")], [graph_sp.Port("result", "Result")], branch_a)
    graph3.add("branch_b", "Branch B (×3)", [graph_sp.Port("input", "Input")], [graph_sp.Port("result", "Result")], branch_b)
    graph3.add("branch_c", "Branch C (×4)", [graph_sp.Port("input", "Input")], [graph_sp.Port("result", "Result")], branch_c)
    graph3.add("merger", "Merge", [graph_sp.Port("a", "A"), graph_sp.Port("b", "B"), graph_sp.Port("c", "C")], [graph_sp.Port("merged", "Merged")], merger)

    # Fan-out from source
    graph3.add_edge("source", "value", "branch_a", "input")
    graph3.add_edge("source", "value", "branch_b", "input")
    graph3.add_edge("source", "value", "branch_c", "input")

    # Fan-in to merger
    graph3.add_edge("branch_a", "result", "merger", "a")
    graph3.add_edge("branch_b", "result", "merger", "b")
    graph3.add_edge("branch_c", "result", "merger", "c")

    print(f"📊 Fan-out Pattern:")
    print(f"   1 source → 3 parallel branches → 1 merger")
    print(f"   This enables parallel execution!")
    print()

    print("🎨 Mermaid Diagram:")
    print(graph3.to_mermaid())

    # ========================================================================
    # PART 4: BRANCHING (Isolated Subgraphs)
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("PART 4: BRANCHING (Isolated subgraphs)")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph4 = graph_sp.Graph()

    # Create experimental branches
    graph4.create_branch("experiment_a")
    graph4.create_branch("experiment_b")
    graph4.create_branch("experiment_c")

    print(f"📊 Branch Information:")
    print(f"   Branches created: {graph4.branch_names()}")
    print(f"   Total branches: {len(graph4.branch_names())}")
    print()

    for branch in graph4.branch_names():
        print(f"   ✓ Has '{branch}': {graph4.has_branch(branch)}")
    print()

    print("🎨 Mermaid Diagram (showing branches):")
    print(graph4.to_mermaid())

    # ========================================================================
    # PART 5: SIMULATING VARIANTS WITH BRANCHES
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("PART 5: SIMULATING VARIANTS (using naming convention)")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph5 = graph_sp.Graph()

    # Create branches with variant naming pattern
    learning_rates = ["lr_0", "lr_1", "lr_2", "lr_3"]
    for lr_name in learning_rates:
        graph5.create_branch(lr_name)

    print(f"📊 Simulated Variants:")
    print(f"   Created {len(learning_rates)} branches with 'lr_N' naming")
    print(f"   Branches: {graph5.branch_names()}")
    print(f"   (In Rust, use create_variants() for automatic setup)")
    print()

    print("🎨 Mermaid Diagram (should show '4 variants' hexagon):")
    print(graph5.to_mermaid())

    # ========================================================================
    # PART 6: EXECUTION AND RESULTS
    # ========================================================================
    print("\n═══════════════════════════════════════════════════════════════════")
    print("PART 6: EXECUTION (Running the graph)")
    print("═══════════════════════════════════════════════════════════════════\n")

    graph6 = graph_sp.Graph()

    def src(inputs):
        return {"value": 10}

    def triple(inputs):
        return {"result": inputs["input"] * 3}

    def add_five(inputs):
        return {"final": inputs["result"] + 5}

    graph6.add("source", "Source", [], [graph_sp.Port("value", "Value")], src)
    graph6.add("tripler", "Tripler", [graph_sp.Port("input", "Input")], [graph_sp.Port("result", "Result")], triple)
    graph6.add("adder", "Add 5", [graph_sp.Port("result", "Input")], [graph_sp.Port("final", "Final")], add_five)

    graph6.add_edge("source", "value", "tripler", "input")
    graph6.add_edge("tripler", "result", "adder", "result")

    print(f"📊 Executing: 10 → ×3 → +5")
    executor = graph_sp.Executor()
    result = executor.execute(graph6)

    print(f"   Success: {result.is_success()}")
    print(f"   Source: {result.get_output('source', 'value')}")
    print(f"   Tripler: {result.get_output('tripler', 'result')}")
    print(f"   Adder: {result.get_output('adder', 'final')}")
    print(f"   Result: 10 × 3 + 5 = {result.get_output('adder', 'final')}")
    print()

    print("🎨 Mermaid Diagram:")
    print(graph6.to_mermaid())

    # ========================================================================
    # SUMMARY
    # ========================================================================
    print("\n╔══════════════════════════════════════════════════════════════════╗")
    print("║  SUMMARY: Python API Features                                   ║")
    print("╚══════════════════════════════════════════════════════════════════╝\n")

    print("📋 CURRENTLY AVAILABLE IN PYTHON:")
    print("   ✅ add() - Add nodes (simplified API)")
    print("   ✅ add_edge() - Explicit edge specification")
    print("   ✅ Branches - create_branch(), has_branch(), branch_names()")
    print("   ✅ Execution - Executor with get_output()")
    print("   ✅ Analysis - Graph statistics")
    print("   ✅ Mermaid - Full visualization")
    print("   ✅ Visualization - Graph inspection")
    print()

    print("📋 AVAILABLE IN RUST (Coming to Python):")
    print("   🔄 Implicit edge mapping - Automatic connections")
    print("   🔄 Variants - create_variants() for parameter sweeps")
    print("   🔄 Merge - Custom merge functions")
    print("   🔄 Nested variants - Cartesian product")
    print()

    print("📖 LEGEND:")
    print("   🔵 Blue nodes      = Source (no inputs)")
    print("   🟣 Purple nodes    = Sink (no outputs)")
    print("   🟠 Orange nodes    = Processing")
    print("   🟢 Green hexagons  = Variant groups (with lr_N naming)")
    print()

    print("💡 TIP: Check out the Rust examples for full feature demonstrations!")
    print()


if __name__ == "__main__":
    main()
