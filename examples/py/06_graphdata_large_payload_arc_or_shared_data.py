"""Example 06: GraphData with Shared Data (Large Payload Sharing)

Demonstrates efficient memory handling for large data in Python
"""

import sys
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

from benchmark_utils import Benchmark, print_header, print_section
import dagex


def create_large_data(_inputs):
    """Create a large dataset."""
    # Create a large list (simulating a large dataset)
    large_list = list(range(1_000_000))
    
    # In Python, objects are reference-counted, so passing the list
    # to multiple consumers doesn't create copies unless modified
    return {"large_data": large_list}


def consumer_a(inputs):
    """Consumer A: process first segment."""
    data = inputs.get("data", [])
    # Access the data by reference - Python won't copy unless we modify
    sum_a = sum(data[:1000])
    return {"sum_a": sum_a}


def consumer_b(inputs):
    """Consumer B: process second segment."""
    data = inputs.get("data", [])
    # Access the data by reference - Python won't copy unless we modify
    sum_b = sum(data[1000:2000])
    return {"sum_b": sum_b}


def consumer_c(inputs):
    """Consumer C: process third segment."""
    data = inputs.get("data", [])
    # Access the data by reference - Python won't copy unless we modify
    sum_c = sum(data[2000:3000])
    return {"sum_c": sum_c}


def main():
    print_header("Example 06: GraphData with Shared Data (Large Payload)")
    
    print("📖 Story:")
    print("   When working with large data, copying it between nodes is expensive.")
    print("   In Python, objects are reference-counted, so passing lists/arrays")
    print("   to multiple consumers shares references rather than copying data.")
    print("   For even better performance, use NumPy arrays which are efficiently")
    print("   passed through the Python/Rust boundary.\n")
    
    print_section("Building the Graph")
    
    graph = dagex.Graph()
    
    # Create large data source
    graph.add(
        create_large_data,
        label="CreateLargeData",
        inputs=None,
        outputs=[("large_data", "data")]
    )
    
    # Add multiple consumers that share the large data
    graph.add(
        consumer_a,
        label="ConsumerA",
        inputs=[("data", "data")],
        outputs=[("sum_a", "sum_a")]
    )
    
    graph.add(
        consumer_b,
        label="ConsumerB",
        inputs=[("data", "data")],
        outputs=[("sum_b", "sum_b")]
    )
    
    graph.add(
        consumer_c,
        label="ConsumerC",
        inputs=[("data", "data")],
        outputs=[("sum_c", "sum_c")]
    )
    
    dag = graph.build()
    
    print_section("Mermaid Diagram")
    print(dag.to_mermaid())
    
    print_section("ASCII Visualization")
    print("                      ConsumerA")
    print("                     /")
    print("  CreateLargeData  -- ConsumerB")
    print("   (1M integers)    \\")
    print("                      ConsumerC")
    
    print("💡 Key insight: The large data (1M integers) is created once")
    print("   and shared by reference. Python's reference counting ensures")
    print("   efficient memory usage.\n")
    
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
    
    print("📊 Consumer outputs (each processes different segments):")
    
    sum_a = context_par.get("sum_a")
    if sum_a is not None:
        print(f"   ConsumerA (first 1000):  sum = {sum_a}")
    
    sum_b = context_par.get("sum_b")
    if sum_b is not None:
        print(f"   ConsumerB (next 1000):   sum = {sum_b}")
    
    sum_c = context_par.get("sum_c")
    if sum_c is not None:
        print(f"   ConsumerC (next 1000):   sum = {sum_c}")
    
    print("\nSequential execution:")
    print(f"  Time: {result_seq.duration_ms:.3f}ms")
    
    print("\nParallel execution:")
    print(f"  Time: {result_par.duration_ms:.3f}ms")
    
    print("\n✅ Reference-based data sharing successful!")
    print("   Memory benefit: Data shared by reference, not copied")
    
    print()


if __name__ == "__main__":
    main()
