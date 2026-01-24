"""Example 05: Output Access

Demonstrates accessing outputs in Python. Note: Python bindings primarily
expose the context (final outputs). This example shows branching and data flow.
"""

import sys
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

from benchmark_utils import Benchmark, print_header, print_section
import dagex
import time


def source(_inputs):
    """Generate initial data."""
    return {"data": 100}


def processor_a(inputs):
    """Processor A: multiply by 2."""
    value = inputs.get("input", 0)
    
    # Simulate I/O or blocking operation that releases the GIL
    # This allows true parallel execution in Python
    time.sleep(0.02)
    
    return {"processed": value * 2}


def processor_b(inputs):
    """Processor B: add 50."""
    value = inputs.get("input", 0)
    
    # Simulate I/O or blocking operation that releases the GIL
    # This allows true parallel execution in Python
    time.sleep(0.02)
    
    return {"processed": value + 50}


def combine(inputs):
    """Combine outputs from branches."""
    a = inputs.get("a", 0)
    b = inputs.get("b", 0)
    return {"final": a + b + 1}


def main():
    print_header("Example 05: Output Access")
    
    print("📖 Story:")
    print("   Sometimes you need access to intermediate results, not just the")
    print("   final outputs. While Python bindings primarily expose the context")
    print("   (final outputs), this example shows how to work with branching")
    print("   and understand the flow of data through the graph.\n")
    
    print_section("Building the Graph")
    
    graph = dagex.Graph()
    
    # Add source
    graph.add(
        source,
        label="Source",
        inputs=None,
        outputs=[("data", "input")]
    )
    
    # Create branch A
    branch_a = dagex.Graph()
    branch_a.add(
        processor_a,
        label="ProcessorA",
        inputs=[("input", "input")],
        outputs=[("processed", "a")]
    )
    branch_a_id = graph.branch(branch_a)
    
    # Create branch B
    branch_b = dagex.Graph()
    branch_b.add(
        processor_b,
        label="ProcessorB",
        inputs=[("input", "input")],
        outputs=[("processed", "b")]
    )
    branch_b_id = graph.branch(branch_b)
    
    # Add combine node
    graph.add(
        combine,
        label="Combine",
        inputs=[("a", "a"), ("b", "b")],
        outputs=[("final", "output")]
    )
    
    dag = graph.build()
    
    print_section("Mermaid Diagram")
    print(dag.to_mermaid())
    
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
    
    print("📊 Accessing outputs:\n")
    
    print("Sequential execution:")
    print(f"  Time: {result_seq.duration_ms:.3f}ms")
    
    print("\nParallel execution:")
    print(f"  Time: {result_par.duration_ms:.3f}ms")
    
    # Final context outputs
    print("\nFinal context outputs:")
    output = context_par.get("output")
    if output is not None:
        print(f"   output: {output}")
    
    print("\nExecution flow:")
    print("   Source: 100")
    print("   ProcessorA (branch A): 100 × 2 = 200")
    print("   ProcessorB (branch B): 100 + 50 = 150")
    print("   Combine: 200 + 150 + 1 = 351")
    
    print("\n✅ Successfully accessed outputs!")
    
    print()


if __name__ == "__main__":
    main()
