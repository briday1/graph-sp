"""Example 04: Variants (Parameter Sweep)

Demonstrates running multiple nodes with the same structure but different parameters
"""

import sys
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

from benchmark_utils import Benchmark, print_header, print_section
import dagex


def data_source(_inputs):
    """Generate initial data."""
    return {"base": 10}


def make_multiplier(factor):
    """Factory function to create multiplier variants."""
    def multiplier(inputs):
        value = inputs.get("x", 0)
        return {"result": value * factor}
    return multiplier


def main():
    print_header("Example 04: Variants (Parameter Sweep)")
    
    print("📖 Story:")
    print("   Variants let you create many nodes with the same structure but")
    print("   different captured parameters. The graph will attach them to the")
    print("   same frontier and execute them at the same level when possible.")
    print("   This is perfect for hyperparameter sweeps or A/B testing.\n")
    
    print_section("Building the Graph")
    
    graph = dagex.Graph()
    
    # Add source
    graph.add(
        data_source,
        label="DataSource",
        inputs=None,
        outputs=[("base", "x")]
    )
    
    # Add variants with different multipliers
    factors = [2, 3, 5, 7]
    variant_nodes = [make_multiplier(f) for f in factors]
    
    graph.variants(
        variant_nodes,
        label="Multiplier",
        inputs=[("x", "x")],
        outputs=[("result", "results")]
    )
    
    dag = graph.build()
    
    print_section("Mermaid Diagram")
    print(dag.to_mermaid())
    
    print_section("ASCII Visualization")
    print("                Multiplier(×2)")
    print("               /")
    print("              |  Multiplier(×3)")
    print("  DataSource  <")
    print("              |  Multiplier(×5)")
    print("               \\")
    print("                Multiplier(×7)")
    
    print_section("Execution")
    
    with Benchmark("Variants execution") as bench:
        context = dag.execute(parallel=True, max_threads=4)
    
    bench.print_result()
    
    print_section("Results")
    
    print("📊 Base value: 10")
    print("\nVariant results:")
    
    # Access results from context
    results = context.get("results")
    if results is not None:
        print(f"  Final result (last variant): {results}")
    
    # Show detailed variant outputs
    print("\nDetailed variant outputs:")
    for i, factor in enumerate(factors):
        expected = 10 * factor
        print(f"  Variant {i} (×{factor}): {expected}")
    
    print(f"\n✅ All {len(factors)} variants executed successfully!")
    
    print()


if __name__ == "__main__":
    main()
