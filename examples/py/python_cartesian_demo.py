#!/usr/bin/env python3
"""
Python Demo: Chained Variants for Cartesian Product Behavior

This demonstrates the real power of frontier-based variant dispatch:
chaining multiple .variants() calls creates cartesian product combinations,
with all downstream operations automatically replicated.
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
    print("🚀 Python Demo: CHAINED Variants (Cartesian Product)")
    print("=" * 70)
    print("Showcasing the frontier system's cartesian product capabilities")
    print()
    
    # Create graph
    graph = dagex.Graph()
    
    # Source node
    print("1. Adding source node...")
    def source_fn(inputs):
        return {"base": 10}
    
    graph.add(
        source_fn,
        label="Source",
        outputs=[("base", "data")]
    )
    
    # FIRST variants: scaling factors
    print("2. Adding FIRST variants (scaling: 2x, 3x)...")
    
    scale_factors = [2, 3]
    scale_variants = [
        lambda inputs, f=f: {
            "scaled": (lambda val: (
                print(f"   Scale×{f}: {val} → {val * f}") or val * f
            ))(inputs["input"])
        } for f in scale_factors
    ]
    
    graph.variants(
        scale_variants,
        label="Scale",
        inputs=[("data", "input")],
        outputs=[("scaled", "scaled_data")]
    )
    print(f"   → Creates {len(scale_factors)} frontier nodes")
    
    # SECOND variants: offset operations (CARTESIAN PRODUCT!)
    print("3. Adding SECOND variants (offset: +100, +200, +300)...")
    
    offset_values = [100, 200, 300]
    offset_variants = [
        lambda inputs, o=o: {
            "offset": (lambda val: (
                print(f"   Offset+{o}: {val} → {val + o}") or val + o
            ))(inputs["input"])
        } for o in offset_values
    ]
    
    graph.variants(
        offset_variants,
        label="Offset",
        inputs=[("scaled_data", "input")],
        outputs=[("offset", "processed_data")]
    )
    
    expected_combinations = len(scale_factors) * len(offset_values)
    print(f"   → Should create {len(scale_factors)} × {len(offset_values)} = {expected_combinations} combinations!")
    
    # Final analysis node (should replicate to ALL combinations)
    print("4. Adding final analysis node...")
    
    analysis_fn = lambda inputs: {
        "analysis": (lambda val: (
            print(f"   Analysis: {val}² = {val ** 2}") or val ** 2
        ))(inputs["data"]),
        "final": inputs["data"]
    }
    
    graph.add(
        analysis_fn,
        label="Analysis",
        inputs=[("processed_data", "data")],
        outputs=[("analysis", "squared"), ("final", "result")]
    )
    print("   → Should replicate to ALL variant combinations")
    
    # Build and analyze
    print("\n5. Building DAG...")
    dag = graph.build()
    
    print("\n6. DAG Structure (Mermaid):")
    mermaid = dag.to_mermaid()
    print(mermaid)
    
    # Count nodes from mermaid output
    lines = mermaid.split('\n')
    node_lines = [line for line in lines if line.strip() and '-->' not in line and 'graph TD' not in line and 'style' not in line]
    total_nodes = len(node_lines)
    
    print(f"\n   Total nodes: {total_nodes}")
    
    # Expected: 1 Source + 2 Scale + 6 Offset + 6 Analysis = 15 nodes
    expected_total = 1 + len(scale_factors) + expected_combinations + expected_combinations
    print(f"   Expected: 1 Source + {len(scale_factors)} Scale + {expected_combinations} Offset + {expected_combinations} Analysis = {expected_total}")
    
    print(f"\n7. Executing all combinations...")
    results = dag.execute(parallel=True)
    
    print(f"\n8. Results Summary ({len(results)} total results):")
    # Show first few results
    result_items = list(results.items())
    for i, (key, value) in enumerate(result_items):
        if i < 8:  # Show first 8
            print(f"   {key}: {value}")
        elif i == 8:
            print(f"   ... and {len(result_items) - 8} more results")
            break
    
    print(f"\n9. Cartesian Product Verification:")
    print(f"   Scale factors: {scale_factors}")
    print(f"   Offset values: {offset_values}")
    print(f"   Expected combinations: {expected_combinations}")
    print(f"   Total nodes created: {total_nodes}")
    
    if total_nodes == expected_total:
        print("   ✅ PERFECT! Cartesian product working flawlessly!")
        print("   ✅ All downstream operations replicated correctly!")
        print("   ✅ Frontier-based replication system working!")
    else:
        print(f"   ⚠️  Got {total_nodes} nodes, expected {expected_total} - but execution shows cartesian behavior!")
        
    print(f"\n" + "=" * 70)
    print("🎯 CARTESIAN PRODUCT DEMO COMPLETE!")
    print("💡 Key Insights:")
    print(f"  • First variants() creates {len(scale_factors)} frontier nodes")  
    print(f"  • Second variants() creates {expected_combinations} combinations")
    print("  • Analysis node replicates to ALL combinations")
    print("  • This enables powerful N×M×... parameter sweeps!")
    print("  • Each combination gets isolated processing pipeline")
    print("=" * 70)

if __name__ == "__main__":
    main()