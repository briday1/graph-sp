"""Example 03: Branch and Merge

Demonstrates fan-out (branching) patterns. Note: Python bindings don't currently
expose the merge API, so this example shows branching without explicit merging.
"""

import sys
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

from benchmark_utils import Benchmark, print_header, print_section
import dagex


def source(_inputs):
    """Generate initial data."""
    return {"data": 50}


def path_a(inputs):
    """Path A: add 10."""
    value = inputs.get("x", 0)
    return {"result": value + 10}


def path_b(inputs):
    """Path B: add 20."""
    value = inputs.get("x", 0)
    return {"result": value + 20}


def combine(inputs):
    """Combine results from branches."""
    # In absence of merge API, we combine sequentially
    a = inputs.get("a", 0)
    b = inputs.get("b", 0)
    return {"combined": a + b}


def main():
    print_header("Example 03: Branch and Merge")
    
    print("📖 Story:")
    print("   Fan-out (branch): Create independent subgraphs that run in parallel.")
    print("   This example demonstrates branching. Note: Python bindings don't")
    print("   currently expose a direct merge API, so we combine results manually.\n")
    
    print_section("Building the Graph")
    
    graph = dagex.Graph()
    
    # Add source
    graph.add(
        source,
        label="Source",
        inputs=None,
        outputs=[("data", "x")]
    )
    
    # Create branch A
    branch_a = dagex.Graph()
    branch_a.add(
        path_a,
        label="PathA (+10)",
        inputs=[("x", "x")],
        outputs=[("result", "a")]
    )
    branch_a_id = graph.branch(branch_a)
    
    # Create branch B
    branch_b = dagex.Graph()
    branch_b.add(
        path_b,
        label="PathB (+20)",
        inputs=[("x", "x")],
        outputs=[("result", "b")]
    )
    branch_b_id = graph.branch(branch_b)
    
    # Add combine node
    graph.add(
        combine,
        label="Combine",
        inputs=[("a", "a"), ("b", "b")],
        outputs=[("combined", "final")]
    )
    
    dag = graph.build()
    
    print_section("Mermaid Diagram")
    print(dag.to_mermaid())
    
    print_section("ASCII Visualization")
    print("          PathA (+10) ──┐")
    print("         /                \\")
    print("  Source                   Combine")
    print("         \\                /")
    print("          PathB (+20) ──┘")
    
    print_section("Execution")
    
    with Benchmark("Branch execution") as bench:
        context = dag.execute(parallel=True, max_threads=4)
    
    bench.print_result()
    
    print_section("Results")
    
    print("📊 Execution flow:")
    print("   Source: 50")
    print("   PathA: 50 + 10 = 60")
    print("   PathB: 50 + 20 = 70")
    print("   Combine: 60 + 70 = 130")
    
    output = context.get("final")
    if output is not None:
        print(f"\n✅ Final output: {output}")
    
    print()


if __name__ == "__main__":
    main()
