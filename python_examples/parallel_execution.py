"""
Parallel execution example using graph-sp Python bindings.

This example demonstrates how independent branches of the DAG can execute
in parallel, showing the parallelization capabilities of the engine.
"""

import graph_sp

import time


def main():
    print("=== graph-sp Python Example: Parallel Execution ===\n")
    
    graph = graph_sp.Graph()
    
    # Source node that outputs a value
    def source_fn(inputs):
        """Generate initial data"""
        print("[source] Generating data...")
        return {"value": 100}
    
    # Branch A - Slow operation
    def branch_a_fn(inputs):
        """Slow processing branch"""
        start = time.time()
        print("[branch_a] Starting slow operation...")
        
        # Simulate heavy computation
        time.sleep(0.5)
        
        result = inputs["input"] * 2
        elapsed = time.time() - start
        print(f"[branch_a] Completed in {elapsed:.3f}s")
        
        return {"output": result}
    
    # Branch B - Fast operation
    def branch_b_fn(inputs):
        """Fast processing branch"""
        start = time.time()
        print("[branch_b] Starting fast operation...")
        
        # Simulate quick computation
        time.sleep(0.1)
        
        result = inputs["input"] + 50
        elapsed = time.time() - start
        print(f"[branch_b] Completed in {elapsed:.3f}s")
        
        return {"output": result}
    
    # Branch C - Medium operation
    def branch_c_fn(inputs):
        """Medium processing branch"""
        start = time.time()
        print("[branch_c] Starting medium operation...")
        
        # Simulate medium computation
        time.sleep(0.3)
        
        result = inputs["input"] // 2
        elapsed = time.time() - start
        print(f"[branch_c] Completed in {elapsed:.3f}s")
        
        return {"output": result}
    
    # Merge node
    def merger_fn(inputs):
        """Merge results from all branches"""
        start = time.time()
        print("[merger] Merging results...")
        
        a = inputs.get("a", 0)
        b = inputs.get("b", 0)
        c = inputs.get("c", 0)
        
        combined = a + b + c
        elapsed = time.time() - start
        print(f"[merger] Completed in {elapsed:.3f}s")
        
        return {"result": combined}
    
    # Build the graph
    print("Building parallel graph...")
    
    graph.add_node(
        "source",
        "Data Source",
        [],
        [graph_sp.Port("value", "Value")],
        source_fn
    )
    
    graph.add_node(
        "branch_a",
        "Branch A (Slow)",
        [graph_sp.Port("input", "Input")],
        [graph_sp.Port("output", "Output")],
        branch_a_fn
    )
    
    graph.add_node(
        "branch_b",
        "Branch B (Fast)",
        [graph_sp.Port("input", "Input")],
        [graph_sp.Port("output", "Output")],
        branch_b_fn
    )
    
    graph.add_node(
        "branch_c",
        "Branch C (Medium)",
        [graph_sp.Port("input", "Input")],
        [graph_sp.Port("output", "Output")],
        branch_c_fn
    )
    
    graph.add_node(
        "merger",
        "Result Merger",
        [
            graph_sp.Port("a", "Branch A Result"),
            graph_sp.Port("b", "Branch B Result"),
            graph_sp.Port("c", "Branch C Result")
        ],
        [graph_sp.Port("result", "Final Result")],
        merger_fn
    )
    
    # Connect source to all branches (fan-out)
    graph.add_edge("source", "value", "branch_a", "input")
    graph.add_edge("source", "value", "branch_b", "input")
    graph.add_edge("source", "value", "branch_c", "input")
    
    # Connect all branches to merger (fan-in)
    graph.add_edge("branch_a", "output", "merger", "a")
    graph.add_edge("branch_b", "output", "merger", "b")
    graph.add_edge("branch_c", "output", "merger", "c")
    
    print("✓ Graph built successfully!\n")
    
    # Validate
    print("Validating graph...")
    graph.validate()
    print("✓ Graph is valid (no cycles detected)\n")
    
    # Analyze
    print("=== Graph Analysis ===")
    analysis = graph.analyze()
    print(f"Node count: {analysis.node_count}")
    print(f"Edge count: {analysis.edge_count}")
    print(f"Depth: {analysis.depth}")
    print(f"Width: {analysis.width}")
    print(f"Summary: {analysis.summary()}\n")
    print("This graph has 3 independent branches that can execute in parallel!\n")
    
    # Visualize
    print("=== Graph Structure ===")
    visualization = graph.visualize()
    print(visualization)
    
    # Generate Mermaid diagram
    print("=== Mermaid Diagram ===")
    mermaid = graph.to_mermaid()
    print(mermaid)
    
    # Execute with timing
    print("=== Executing Graph ===")
    print("Note: Branches A, B, and C will execute in parallel after the source completes.\n")
    
    overall_start = time.time()
    executor = graph_sp.Executor()
    result = executor.execute(graph)
    total_time = time.time() - overall_start
    
    print(f"\n✓ Execution completed!\n")
    
    # Display results
    print("=== Results ===")
    
    source_val = result.get_output("source", "value")
    branch_a_val = result.get_output("branch_a", "output")
    branch_b_val = result.get_output("branch_b", "output")
    branch_c_val = result.get_output("branch_c", "output")
    final_val = result.get_output("merger", "result")
    
    print(f"Source value: {source_val}")
    print(f"Branch A result (×2): {branch_a_val}")
    print(f"Branch B result (+50): {branch_b_val}")
    print(f"Branch C result (÷2): {branch_c_val}")
    print(f"Final merged result: {final_val}")
    
    print(f"\n=== Performance Analysis ===")
    print(f"Total execution time: {total_time:.3f}s")
    print("\nExpected times:")
    print("  - Sequential execution: ~0.9s (0.5 + 0.1 + 0.3)")
    print("  - Parallel execution: ~0.5s (max of branch times)")
    print("\nNote: Current implementation executes sequentially following topological order.")
    print("The architecture supports parallel execution - branches are independent!")
    
    print("\n=== Example Complete ===")


if __name__ == "__main__":
    main()
