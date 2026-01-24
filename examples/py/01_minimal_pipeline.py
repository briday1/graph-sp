"""Example 01: Minimal Pipeline

Demonstrates the simplest dataflow: generator → transformer → aggregator
"""

import sys
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

from benchmark_utils import Benchmark, print_header, print_section
import dagex
import time


def generate(_inputs):
    """Generate initial data."""
    return {"number": 10}


def double(inputs):
    """Double the input value."""
    value = inputs.get("x", 0)
    
    # Simulate I/O or blocking operation that releases the GIL
    # This allows true parallel execution in Python
    time.sleep(0.15)
    
    return {"result": value * 2}


def add_five(inputs):
    """Add five to the input value."""
    value = inputs.get("y", 0)
    
    # Simulate I/O or blocking operation that releases the GIL
    # This allows true parallel execution in Python
    time.sleep(0.15)
    
    return {"final": value + 5}


def main():
    print_header("Example 01: Minimal Pipeline")
    
    print("📖 Story:")
    print("   This example shows the simplest possible DAG pipeline:")
    print("   A generator creates a number, a transformer doubles it,")
    print("   and a final node adds five to produce the result.\n")
    
    print_section("Building the Graph")
    
    graph = dagex.Graph()
    graph.add(
        generate,
        label="Generator",
        inputs=None,
        outputs=[("number", "x")]
    )
    graph.add(
        double,
        label="Doubler",
        inputs=[("x", "x")],
        outputs=[("result", "y")]
    )
    graph.add(
        add_five,
        label="AddFive",
        inputs=[("y", "y")],
        outputs=[("final", "output")]
    )
    
    dag = graph.build()
    
    print_section("Mermaid Diagram")
    print(dag.to_mermaid())
    
    print_section("ASCII Visualization")
    print("  Generator → Doubler → AddFive")
    print("     (10)       (20)       (25)")
    
    print_section("Sequential Execution (parallel=False)")
    
    with Benchmark("Sequential execution") as bench_seq:
        context_seq = dag.execute(parallel=False)
    
    bench_seq.print_result()
    result_seq = bench_seq.result
    
    print_section("Parallel Execution (parallel=True)")
    
    with Benchmark("Parallel execution") as bench_par:
        context_par = dag.execute(parallel=True, max_threads=4)
    
    bench_par.print_result()
    result_par = bench_par.result
    
    print_section("Results")
    
    print("Sequential execution:")
    output_seq = context_seq.get("output")
    if output_seq is not None:
        print(f"  Final output: {output_seq}")
        print(f"  Time: {result_seq.duration_ms:.3f}ms")
    
    print("\nParallel execution:")
    output_par = context_par.get("output")
    if output_par is not None:
        print(f"  Final output: {output_par}")
        print(f"  Time: {result_par.duration_ms:.3f}ms")
    
    print("\n✅ Pipeline completed successfully!")
    print("   (Started with 10, doubled to 20, added 5 = 25)")
    
    print()


if __name__ == "__main__":
    main()
