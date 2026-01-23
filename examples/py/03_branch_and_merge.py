"""Example 03: Branch and Merge

Demonstrates fan-out (branching) and fan-in (merging) patterns
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


def merge(inputs):
    """Merge results from both paths."""
    a = inputs.get("from_a", 0)
    b = inputs.get("from_b", 0)
    return {"combined": a + b}


def main():
    print_header("Example 03: Branch and Merge")
    
    print("📖 Story:")
    print("   Fan-out (branch): Create independent subgraphs that run in parallel.")
    print("   Fan-in (merge): Combine branch-specific outputs safely.")
    print("   This pattern is useful for processing data through multiple")
    print("   independent pipelines and then combining the results.\n")
    
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
        outputs=[("result", "result")]
    )
    branch_a_id = graph.branch(branch_a)
    
    # Create branch B
    branch_b = dagex.Graph()
    branch_b.add(
        path_b,
        label="PathB (+20)",
        inputs=[("x", "x")],
        outputs=[("result", "result")]
    )
    branch_b_id = graph.branch(branch_b)
    
    # Merge branches
    graph.merge(
        merge,
        label="Merge",
        inputs=[
            (branch_a_id, "result", "from_a"),
            (branch_b_id, "result", "from_b"),
        ],
        outputs=[("combined", "final")]
    )
    
    dag = graph.build()
    
    print_section("Mermaid Diagram")
    print(dag.to_mermaid())
    
    print_section("ASCII Visualization")
    print("          PathA (+10) ──┐")
    print("         /                \\")
    print("  Source                   Merge")
    print("         \\                /")
    print("          PathB (+20) ──┘")
    
    print_section("Execution")
    
    with Benchmark("Branch and merge execution") as bench:
        context = dag.execute(parallel=True, max_threads=4)
    
    bench.print_result()
    
    print_section("Results")
    
    print("📊 Execution flow:")
    print("   Source: 50")
    print("   PathA: 50 + 10 = 60")
    print("   PathB: 50 + 20 = 70")
    print("   Merge: 60 + 70 = 130")
    
    output = context.get("final")
    if output is not None:
        print(f"\n✅ Final output: {output}")
    
    print()


if __name__ == "__main__":
    main()
