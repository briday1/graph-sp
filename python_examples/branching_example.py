"""
Example: Branching and Variants in Python

This example demonstrates the new features:
- Creating named branches (subgraphs)
- Accessing branch information
- Future: Merging and variants (coming soon to Python API)
"""

import pygraph_sp as gs


def main():
    print("=== Graph-SP Python Example: Branching ===\n")

    # Example 1: Simple Branching
    print("Example 1: Creating Branches")
    print("-----------------------------")
    graph = graph_sp.Graph()

    # Create two experimental branches
    graph.create_branch("experiment_a")
    graph.create_branch("experiment_b")

    print(f"✓ Created branches: {graph.branch_names()}")
    print(f"✓ Has 'experiment_a': {graph.has_branch('experiment_a')}")
    print(f"✓ Has 'experiment_b': {graph.has_branch('experiment_b')}")
    print(f"✓ Has 'nonexistent': {graph.has_branch('nonexistent')}")
    print()

    # Example 2: Working with the main graph and branches
    print("Example 2: Main Graph vs Branches")
    print("----------------------------------")
    graph2 = graph_sp.Graph()

    # Add a node to the main graph
    def main_fn(inputs):
        return {"output": "main graph result"}

    graph2.add(
        "main_node",
        "Main Node",
        [],
        [graph_sp.Port("output", "Output")],
        main_fn
    )

    # Create branches
    graph2.create_branch("branch_a")
    graph2.create_branch("branch_b")

    print(f"✓ Main graph has {graph2.node_count()} node")
    print(f"✓ Created {len(graph2.branch_names())} branches")
    print(f"✓ Branches: {graph2.branch_names()}")
    print()

    # Example 3: Use cases
    print("Example 3: Practical Use Cases")
    print("-------------------------------")
    print("Branches can be used for:")
    print("  • A/B testing different algorithms")
    print("  • Parallel experimentation")
    print("  • Organizing complex workflows")
    print("  • Creating variant configurations")
    print()

    print("=== Summary ===")
    print("✓ Branch creation and management demonstrated")
    print("✓ Python bindings support basic branch operations")
    print("\nNote: Full merge and variant features are available in Rust API")
    print("      and will be added to Python API in future updates.")


if __name__ == "__main__":
    main()
