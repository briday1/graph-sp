"""Example 05: Output Access

Demonstrates accessing individual node and branch outputs
"""

import sys
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

from benchmark_utils import Benchmark, print_header, print_section
import dagex


def source(_inputs):
    """Generate initial data."""
    return {"data": 100}


def processor_a(inputs):
    """Processor A: multiply by 2."""
    value = inputs.get("input", 0)
    return {"processed": value * 2}


def processor_b(inputs):
    """Processor B: add 50."""
    value = inputs.get("input", 0)
    return {"processed": value + 50}


def final_node(inputs):
    """Final processing node."""
    value = inputs.get("x", 0)
    return {"final": value + 1}


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
        outputs=[("processed", "result")]
    )
    branch_a_id = graph.branch(branch_a)
    
    # Create branch B
    branch_b = dagex.Graph()
    branch_b.add(
        processor_b,
        label="ProcessorB",
        inputs=[("input", "input")],
        outputs=[("processed", "result")]
    )
    branch_b_id = graph.branch(branch_b)
    
    # Add final node that consumes from branch A
    graph.add(
        final_node,
        label="FinalNode",
        inputs=[(branch_a_id, "result", "x")],
        outputs=[("final", "output")]
    )
    
    dag = graph.build()
    
    print_section("Mermaid Diagram")
    print(dag.to_mermaid())
    
    print_section("Execution")
    
    with Benchmark("Execution with detailed output") as bench:
        context = dag.execute(parallel=True, max_threads=4)
    
    bench.print_result()
    
    print_section("Results")
    
    print("📊 Accessing outputs:\n")
    
    # Final context outputs
    print("Final context outputs:")
    output = context.get("output")
    if output is not None:
        print(f"   output: {output}")
    
    print("\nExecution flow:")
    print("   Source: 100")
    print("   ProcessorA (branch A): 100 × 2 = 200")
    print("   ProcessorB (branch B): 100 + 50 = 150")
    print("   FinalNode: 200 + 1 = 201")
    
    print("\n✅ Successfully accessed outputs!")
    
    print()


if __name__ == "__main__":
    main()
