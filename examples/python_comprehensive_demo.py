#!/usr/bin/env python3
"""
Comprehensive Python demo matching the Rust comprehensive_demo.rs

This demonstrates:
1. Basic sequential pipeline
2. Parallel branching
3. Mermaid visualization
4. Runtime comparison

Run this alongside the Rust version to verify identical behavior.
"""

import graph_sp
import time

print("=" * 70)
print("Python Comprehensive Demo - graph-sp")
print("=" * 70)

# Demo 1: Basic Sequential Pipeline
print("\n" + "─" * 70)
print("Demo 1: Basic Sequential Pipeline")
print("─" * 70)

def data_source(inputs, variant_params):
    """Source node that produces initial data"""
    return {"raw_data": "100"}

def double_value(inputs, variant_params):
    """Process node that doubles the input"""
    val = int(inputs.get("input", "0"))
    return {"doubled": str(val * 2)}

def add_ten(inputs, variant_params):
    """Process node that adds 10"""
    val = int(inputs.get("input", "0"))
    return {"result": str(val + 10)}

# Create sequential pipeline
graph = graph_sp.PyGraph()
graph.add(data_source, "Source", None, [("raw_data", "data")])
graph.add(double_value, "Double", [("data", "input")], [("doubled", "doubled_data")])
graph.add(add_ten, "AddTen", [("doubled_data", "input")], [("result", "final")])

dag = graph.build()

print("\nExecuting sequential pipeline...")
start_time = time.time()
result = dag.execute(parallel=False)
elapsed = time.time() - start_time

print(f"Execution completed in {elapsed*1000:.2f}ms")
print(f"Final result: {result.get('final')}")
print(f"   Expected: 210 (100 * 2 + 10)")

print("\nMermaid Diagram:")
print(dag.to_mermaid())

# Demo 2: Parallel Branching
print("\n" + "─" * 70)
print("Demo 2: Parallel Branching (Fan-Out)")
print("─" * 70)

def source(inputs, variant_params):
    return {"value": "50"}

def branch_a(inputs, variant_params):
    """Branch A: multiply by 2"""
    val = int(inputs.get("x", "0"))
    time.sleep(0.1)  # Simulate work
    return {"result": str(val * 2)}

def branch_b(inputs, variant_params):
    """Branch B: multiply by 3"""
    val = int(inputs.get("x", "0"))
    time.sleep(0.1)  # Simulate work
    return {"result": str(val * 3)}

def branch_c(inputs, variant_params):
    """Branch C: add 100"""
    val = int(inputs.get("x", "0"))
    time.sleep(0.1)  # Simulate work
    return {"result": str(val + 100)}

# Create graph with branches
graph2 = graph_sp.PyGraph()
graph2.add(source, "Source", None, [("value", "data")])

# Create branches
branch_graph_a = graph_sp.PyGraph()
branch_graph_a.add(branch_a, "BranchA[*2]", [("data", "x")], [("result", "result_a")])

branch_graph_b = graph_sp.PyGraph()
branch_graph_b.add(branch_b, "BranchB[*3]", [("data", "x")], [("result", "result_b")])

branch_graph_c = graph_sp.PyGraph()
branch_graph_c.add(branch_c, "BranchC[+100]", [("data", "x")], [("result", "result_c")])

graph2.branch(branch_graph_a)
graph2.branch(branch_graph_b)
graph2.branch(branch_graph_c)

dag2 = graph2.build()

print("\nExecuting parallel branches...")
print("   Each branch simulates 100ms of work")

start_time = time.time()
result2 = dag2.execute(parallel=True)
elapsed = time.time() - start_time

print(f"\nExecution completed in {elapsed*1000:.2f}ms")
print(f"Branch A result (50*2): {result2.get('result_a')}")
print(f"Branch B result (50*3): {result2.get('result_b')}")
print(f"Branch C result (50+100): {result2.get('result_c')}")

if elapsed < 0.25:
    print(f"Appears parallel! (< 250ms with 3x 100ms branches)")
else:
    print(f"Sequential execution detected (took {elapsed*1000:.2f}ms)")

print("\nMermaid Diagram:")
print(dag2.to_mermaid())

# Demo 3: Complex Pipeline with Statistics
print("\n" + "─" * 70)
print("Demo 3: Statistics and Verification")
print("─" * 70)

def source_large(inputs, variant_params):
    return {"data": "1000"}

def compute_1(inputs, variant_params):
    val = int(inputs.get("x", "0"))
    return {"out": str(val // 2)}

def compute_2(inputs, variant_params):
    val = int(inputs.get("x", "0"))
    return {"out": str(val * 3)}

# Create simpler graph without merge for now
# (merge requires special handling in the API that may not be fully exposed in Python)
graph3 = graph_sp.PyGraph()
graph3.add(source_large, "Source", None, [("data", "data")])
graph3.add(compute_1, "Compute1[/2]", [("data", "x")], [("out", "path1")])
graph3.add(compute_2, "Compute2[*3]", [("path1", "x")], [("out", "path2")])

dag3 = graph3.build()

print("\nExecuting sequential pipeline...")
start_time = time.time()
result3 = dag3.execute(parallel=False)
elapsed = time.time() - start_time

print(f"Execution completed in {elapsed*1000:.2f}ms")
print(f"Path 1 (1000/2): {result3.get('path1')}")
print(f"Path 2 (500*3): {result3.get('path2')}")
print(f"   Expected path2: 1500")

# Verify correctness
expected = 1500
actual = int(result3.get('path2', '0'))
if actual == expected:
    print("Verification PASSED: Results match expected values")
else:
    print(f"Verification FAILED: Expected {expected}, got {actual}")

print("\nMermaid Diagram:")
print(dag3.to_mermaid())

print("\n" + "=" * 70)
print("Python Demo Complete!")
print("Compare this output with: cargo run --example comprehensive_demo")
print("=" * 70)
