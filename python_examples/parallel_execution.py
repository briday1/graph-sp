"""
Parallel execution example using graph-sp Python bindings.

This example demonstrates how independent branches of the DAG can execute
in parallel, showing the parallelization capabilities of the engine.
"""

import pygraphsp as gs
import time


def main():
    print("=== graph-sp Python Example: Parallel Execution ===\n")
    
    graph = gs.Graph()
    # Disable implicit "previous node" auto-wiring so branches remain independent
    graph.set_strict_edge_mapping(True)
    
    # Source node that outputs a value
    def source_fn(inputs):
        """Generate initial data"""
        print("[source] Generating data...")
        return {"input": 100}

    # Branch durations (seconds) — keep these as constants so reporting matches behavior
    BRANCH_A_SLEEP = 0.5
    BRANCH_B_SLEEP = 0.1
    BRANCH_C_SLEEP = 0.3

    # Branch A - Slow operation
    def branch_a_fn(inputs):
        """Slow processing branch"""
        start = time.time()
        print("[branch_a] Starting slow operation...")

        # Simulate heavy computation (longer so parallelism is obvious)
        time.sleep(BRANCH_A_SLEEP)

        result = inputs["input"] * 2
        elapsed = time.time() - start
        print(f"[branch_a] Completed in {elapsed:.3f}s")

        return {"a": result}

    # Branch B - Fast operation
    def branch_b_fn(inputs):
        """Fast processing branch"""
        start = time.time()
        print("[branch_b] Starting fast operation...")

        # Simulate quick computation
        time.sleep(BRANCH_B_SLEEP)

        result = inputs["input"] + 50
        elapsed = time.time() - start
        print(f"[branch_b] Completed in {elapsed:.3f}s")

        return {"b": result}

    # Branch C - Medium operation
    def branch_c_fn(inputs):
        """Medium processing branch"""
        start = time.time()
        print("[branch_c] Starting medium operation...")

        # Simulate medium computation
        time.sleep(BRANCH_C_SLEEP)

        result = inputs["input"] // 2
        elapsed = time.time() - start
        print(f"[branch_c] Completed in {elapsed:.3f}s")

        return {"c": result}
    
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

    # Source now emits to "input" so auto_connect can wire fan-out
    graph.add(
        source_fn,
        label="Data Source",
        outputs=["input"]
    )

    # Branches output to a/b/c respectively so auto_connect will wire fan-in
    graph.add(
        branch_a_fn,
        label="Branch A (Slow)",
        inputs=["input"],
        outputs=["a"]
    )

    graph.add(
        branch_b_fn,
        label="Branch B (Fast)",
        inputs=["input"],
        outputs=["b"]
    )

    graph.add(
        branch_c_fn,
        label="Branch C (Medium)",
        inputs=["input"],
        outputs=["c"]
    )

    graph.add(
        merger_fn,
        label="Result Merger",
        inputs=["a", "b", "c"],
        outputs=["result"]
    )

    # Auto-connect everything based on port name matching
    edges_created = graph.auto_connect()
    print(f"✓ Graph built! Auto-connected {edges_created} edges\n")

    # Validate
    print("Validating graph...")
    graph.validate()
    print("✓ Graph is valid (no cycles detected)\n")

    # Lightweight analysis (Inspector not exposed to Python)
    print("=== Graph Analysis ===")
    print(f"Node count: {graph.node_count()}")
    print(f"Edge count: {graph.edge_count()}\n")
    print("This graph has 3 independent branches that can execute in parallel!\n")

    # Execute with timing
    print("=== Executing Graph ===")
    print("Note: Branches A, B, and C will execute in parallel after the source completes.\n")

    overall_start = time.time()
    # Request an executor configured for multiple workers.
    # Note: Python-level parallelism is limited by the GIL for CPU-bound Python code.
    # This demo uses time.sleep (which releases the GIL) to show concurrent scheduling.
    executor = gs.Executor(4)
    result = executor.execute(graph)
    total_time = time.time() - overall_start

    print(f"\n✓ Execution completed!\n")

    # Display results
    print("=== Results ===")

    source_val = result.get_output("source_fn", "input")
    branch_a_val = result.get_output("branch_a_fn", "a")
    branch_b_val = result.get_output("branch_b_fn", "b")
    branch_c_val = result.get_output("branch_c_fn", "c")
    final_val = result.get_output("merger_fn", "result")

    print(f"Source value: {source_val}")
    print(f"Branch A result (×2): {branch_a_val}")
    print(f"Branch B result (+50): {branch_b_val}")
    print(f"Branch C result (÷2): {branch_c_val}")
    print(f"Final merged result: {final_val}")

    print(f"\n=== Performance Analysis ===")
    print(f"Total execution time: {total_time:.3f}s")

    sequential_expected = BRANCH_A_SLEEP + BRANCH_B_SLEEP + BRANCH_C_SLEEP
    parallel_expected = max(BRANCH_A_SLEEP, BRANCH_B_SLEEP, BRANCH_C_SLEEP)

    print("\nExpected times:")
    print(f"  - Sequential execution: ~{sequential_expected:.3f}s (sum of branch times)")
    print(f"  - Parallel execution: ~{parallel_expected:.3f}s (max of branch times)")

    # Allow a small tolerance for scheduling overhead
    tolerance = 0.2
    if total_time <= parallel_expected + tolerance:
        print("\n✓ Parallel execution confirmed! Branches executed concurrently.")
        print("The executor identified 3 independent branches and ran them in parallel.")
    else:
        print("\n⚠ Sequential execution detected. Execution time matches sequential.")

    print("\n=== Example Complete ===")


if __name__ == "__main__":
    main()
