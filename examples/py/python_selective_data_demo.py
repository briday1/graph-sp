#!/usr/bin/env python3
"""
Test to verify that Python nodes only receive the data they request
via input_mapping, not the entire context.
"""

import sys
sys.path.insert(0, '../target/wheels')  # Adjust if needed

try:
    from dagex import Graph
except ImportError:
    print("ERROR: Could not import dagex. Make sure to build with:")
    print("  maturin develop --release")
    sys.exit(1)

def create_multiple_outputs(inputs):
    """Source node that creates 4 large datasets"""
    # Create 4 large lists (simulating large data)
    data_a = list(range(1_000_000))
    data_b = list(range(1_000_000, 2_000_000))
    data_c = list(range(2_000_000, 3_000_000))
    data_d = list(range(3_000_000, 4_000_000))
    
    size_mb = sys.getsizeof(data_a) / 1_000_000
    print(f"Source: Created 4 datasets, each ~{size_mb:.1f} MB (total ~{size_mb * 4:.1f} MB)")
    
    return {
        'data_a': data_a,
        'data_b': data_b,
        'data_c': data_c,
        'data_d': data_d,
    }

def node_wants_only_a(inputs):
    """Node that only requests data_a"""
    print(f"\nNode_A received {len(inputs)} inputs:")
    for key, value in inputs.items():
        if isinstance(value, list):
            size_mb = sys.getsizeof(value) / 1_000_000
            print(f"  - {key}: ~{size_mb:.1f} MB")
    
    if 'data_a' in inputs:
        return {'result_a': sum(inputs['data_a'])}
    return {}

def node_wants_only_b(inputs):
    """Node that only requests data_b"""
    print(f"\nNode_B received {len(inputs)} inputs:")
    for key, value in inputs.items():
        if isinstance(value, list):
            size_mb = sys.getsizeof(value) / 1_000_000
            print(f"  - {key}: ~{size_mb:.1f} MB")
    
    if 'data_b' in inputs:
        return {'result_b': sum(inputs['data_b'])}
    return {}

def node_wants_c_and_d(inputs):
    """Node that requests both data_c and data_d"""
    print(f"\nNode_CD received {len(inputs)} inputs:")
    for key, value in inputs.items():
        if isinstance(value, list):
            size_mb = sys.getsizeof(value) / 1_000_000
            print(f"  - {key}: ~{size_mb:.1f} MB")
    
    if 'data_c' in inputs and 'data_d' in inputs:
        return {'result_cd': sum(inputs['data_c']) + sum(inputs['data_d'])}
    return {}

def node_wants_nothing(inputs):
    """Node that requests nothing from context"""
    print(f"\nNode_Nothing received {len(inputs)} inputs:")
    for key in inputs.keys():
        print(f"  - {key}")
    
    return {'constant': 42}

def main():
    print("=== Python Selective Data Access Test ===\n")
    print("Testing whether Python nodes only receive data they request via input_mapping\n")
    print("Context has 4 large datasets (data_a, data_b, data_c, data_d)")
    print("Each node requests different subsets via input_mapping\n")
    
    graph = Graph()
    
    # Source creates 4 datasets
    graph.add(
        create_multiple_outputs,
        label="Source",
        inputs=None,
        outputs=[
            ('data_a', 'data_a'),
            ('data_b', 'data_b'),
            ('data_c', 'data_c'),
            ('data_d', 'data_d'),
        ]
    )
    
    # Node only wants data_a
    graph.add(
        node_wants_only_a,
        label="Node_A",
        inputs=[('data_a', 'data_a')],  # ← Only requests data_a!
        outputs=[('result_a', 'result_a')]
    )
    
    # Node only wants data_b
    graph.add(
        node_wants_only_b,
        label="Node_B",
        inputs=[('data_b', 'data_b')],  # ← Only requests data_b!
        outputs=[('result_b', 'result_b')]
    )
    
    # Node wants data_c and data_d
    graph.add(
        node_wants_c_and_d,
        label="Node_CD",
        inputs=[('data_c', 'data_c'), ('data_d', 'data_d')],  # ← Requests both
        outputs=[('result_cd', 'result_cd')]
    )
    
    # Node wants nothing from context
    graph.add(
        node_wants_nothing,
        label="Node_Nothing",
        inputs=None,  # ← Requests nothing!
        outputs=[('constant', 'constant')]
    )
    
    dag = graph.build()
    
    print("\n--- Executing DAG ---")
    result = dag.execute(parallel=False)
    
    print("\n=== Analysis ===")
    print("EXPECTED:")
    print("  - Node_A should receive ONLY data_a (1 input)")
    print("  - Node_B should receive ONLY data_b (1 input)")
    print("  - Node_CD should receive ONLY data_c and data_d (2 inputs)")
    print("  - Node_Nothing should receive NOTHING (0 inputs)")
    print("\nIf nodes receive ALL context data instead:")
    print("  - Each would receive 4+ inputs")
    print("  - Memory usage would be (# nodes) × (total context size)")
    print("\n✓ If you see the expected behavior above, Python has the same")
    print("  memory-efficient selective data access as Rust!")

if __name__ == '__main__':
    main()
