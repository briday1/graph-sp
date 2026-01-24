"""Example 02: Parallel vs Sequential Execution

Demonstrates how independent nodes execute in parallel
"""

import sys
import time
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

from benchmark_utils import Benchmark, print_header, print_section
import dagex


def source(_inputs):
    """Generate initial data."""
    return {"value": 100}


def task_a(inputs):
    """Task A: add 10 with simulated work."""
    value = inputs.get("input", 0)
    # Simulate some work
    time.sleep(0.15)
    return {"result_a": value + 10}


def task_b(inputs):
    """Task B: add 20 with simulated work."""
    value = inputs.get("input", 0)
    # Simulate some work
    time.sleep(0.15)
    return {"result_b": value + 20}


def task_c(inputs):
    """Task C: add 30 with simulated work."""
    value = inputs.get("input", 0)
    # Simulate some work
    time.sleep(0.15)
    return {"result_c": value + 30}


def main():
    print_header("Example 02: Parallel vs Sequential Execution")
    
    print("📖 Story:")
    print("   When nodes at the same level have no dependencies between them,")
    print("   they can execute in parallel. This example shows three independent")
    print("   tasks (A, B, C) that each take ~50ms. Sequential execution takes")
    print("   ~150ms total, while parallel execution takes only ~50ms.\n")
    
    print_section("Building the Graph")
    
    graph = dagex.Graph()
    graph.add(
        source,
        label="Source",
        inputs=None,
        outputs=[("value", "input")]
    )
    graph.add(
        task_a,
        label="TaskA",
        inputs=[("input", "input")],
        outputs=[("result_a", "a")]
    )
    graph.add(
        task_b,
        label="TaskB",
        inputs=[("input", "input")],
        outputs=[("result_b", "b")]
    )
    graph.add(
        task_c,
        label="TaskC",
        inputs=[("input", "input")],
        outputs=[("result_c", "c")]
    )
    
    dag = graph.build()
    
    print_section("Mermaid Diagram")
    print(dag.to_mermaid())
    
    print_section("ASCII Visualization")
    print("        TaskA (+10)")
    print("       /")
    print("  Source -- TaskB (+20)")
    print("       \\")
    print("        TaskC (+30)")
    
    print_section("Sequential Execution")
    
    with Benchmark("Sequential execution") as bench_seq:
        context_seq = dag.execute(parallel=False)
    
    bench_seq.print_result()
    result_seq = bench_seq.result
    
    print_section("Parallel Execution")
    
    with Benchmark("Parallel execution") as bench_par:
        context_par = dag.execute(parallel=True, max_threads=4)
    
    bench_par.print_result()
    result_par = bench_par.result
    
    print_section("Results")
    
    print("Sequential results:")
    print(f"  TaskA: {context_seq.get('a')}")
    print(f"  TaskB: {context_seq.get('b')}")
    print(f"  TaskC: {context_seq.get('c')}")
    print(f"  Time: {result_seq.duration_ms:.3f}ms")
    
    print("\nParallel results:")
    print(f"  TaskA: {context_par.get('a')}")
    print(f"  TaskB: {context_par.get('b')}")
    print(f"  TaskC: {context_par.get('c')}")
    print(f"  Time: {result_par.duration_ms:.3f}ms")
    
    speedup = result_seq.duration_ms / result_par.duration_ms
    print(f"\n⚡ Speedup: {speedup:.2f}x faster with parallel execution!")
    
    print()


if __name__ == "__main__":
    main()
