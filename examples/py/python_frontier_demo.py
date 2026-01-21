#!/usr/bin/env python3
"""
Python Demo: Frontier-Based Variant Dispatch

This demo shows how variants() creates multiple execution paths
and automatically replicates downstream operations for each variant.
"""

import sys
import os

# Add the Python package to path
sys.path.insert(0, '/workspaces/graph-sp/target/release')

try:
    import dagex
except ImportError:
    print("Error: dagex Python module not found. Run 'maturin develop --release' first.")
    sys.exit(1)

def main():
    print("🚀 Python Demo: Frontier-Based Variant Dispatch")
    print("=" * 60)
    print()
    
    # Create graph
    graph = dagex.Graph()
    
    # Add source node
    print("1. Adding source node...")
    def source_fn(inputs):
        return {"x": 42}
    
    graph.add(
        source_fn,
        label="Source",
        outputs=[("x", "data")]
    )
    
    # Add variants - this should create multiple paths
    print("2. Adding variants (2x, 3x, 5x scaling)...")
    
    def scale_2x(inputs):
        val = inputs["input"]
        result = val * 2
        print(f"   Variant 2x: {val} -> {result}")
        return {"scaled": result}
    
    def scale_3x(inputs):
        val = inputs["input"] 
        result = val * 3
        print(f"   Variant 3x: {val} -> {result}")
        return {"scaled": result}
        
    def scale_5x(inputs):
        val = inputs["input"]
        result = val * 5  
        print(f"   Variant 5x: {val} -> {result}")
        return {"scaled": result}
    
    graph.variants(
        [scale_2x, scale_3x, scale_5x],
        label="Scale",
        inputs=[("data", "input")],
        outputs=[("scaled", "result")]
    )
    
    # Add downstream node AFTER variants - should connect to ALL variants
    print("3. Adding downstream node (should replicate for each variant)...")
    
    def downstream_fn(inputs):
        val = inputs["r"]
        result = val + 1000
        print(f"   Downstream: {val} + 1000 = {result}")
        return {"final": result}
    
    graph.add(
        downstream_fn,
        label="Downstream", 
        inputs=[("result", "r")],
        outputs=[("final", "output")]
    )
    
    # Build and execute
    print("\n4. Building DAG...")
    dag = graph.build()
    
    print("\n5. Mermaid Diagram:")
    print(dag.to_mermaid())
    
    print("\n6. Executing...")
    results = dag.execute(parallel=True)
    
    print("\n7. Results:")
    for key, value in results.items():
        print(f"   {key}: {value}")
    
    # Analysis
    print("\n8. Node Analysis:")
    node_count = dag.node_count()
    print(f"   Total nodes: {node_count}")
    
    # Count specific node types by looking at the mermaid output
    mermaid = dag.to_mermaid()
    scale_nodes = mermaid.count('"Scale (v')
    downstream_nodes = mermaid.count('"Downstream"')
    
    print(f"   Scale variant nodes: {scale_nodes}")
    print(f"   Downstream nodes: {downstream_nodes}")
    
    print("\n9. Verification:")
    print("   Expected: 1 Source + 3 Scale + 3 Downstream = 7 nodes")
    if node_count == 7:
        print("   ✅ Frontier-based replication working correctly!")
        print("   ✅ Each variant has its own downstream node!")
        print("   ✅ All downstream operations replicated per variant!")
    else:
        print(f"   ❌ Expected 7 nodes, got {node_count}")
    
    print("\n" + "=" * 60)
    print("🎯 Key Insights:")
    print("• variants() creates multiple execution paths from frontier")
    print("• Downstream operations automatically replicate for each variant")  
    print("• This enables cartesian product parameter sweeps")
    print("• Each variant gets its own isolated downstream processing")
    print("=" * 60)

if __name__ == "__main__":
    main()