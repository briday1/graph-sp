#!/usr/bin/env python3
"""
Parallel Execution Demo with Runtime Statistics

This demonstrates:
1. Sequential vs parallel execution timing
2. True parallelization verification
3. Detailed runtime statistics
4. Comparison with Rust version
"""

import graph_sp
import time
import threading

print("=" * 70)
print("Python Parallel Execution Demo - graph-sp")
print("Runtime Statistics and Parallelization Verification")
print("=" * 70)

# Track execution details
execution_log = []
log_lock = threading.Lock()

def log_execution(node_name, start, end):
    with log_lock:
        execution_log.append({
            'node': node_name,
            'start': start,
            'end': end,
            'duration': end - start
        })

# Demo 1: Sequential vs Parallel with Timing
print("\n" + "─" * 70)
print("Demo 1: Sequential vs Parallel Execution")
print("─" * 70)

def source_node(inputs, variant_params):
    """Source node"""
    return {"data": "initial"}

def worker_a(inputs, variant_params):
    """Worker A - 100ms work"""
    start = time.time()
    time.sleep(0.1)
    end = time.time()
    log_execution("WorkerA", start, end)
    return {"result": "A_done"}

def worker_b(inputs, variant_params):
    """Worker B - 100ms work"""
    start = time.time()
    time.sleep(0.1)
    end = time.time()
    log_execution("WorkerB", start, end)
    return {"result": "B_done"}

def worker_c(inputs, variant_params):
    """Worker C - 100ms work"""
    start = time.time()
    time.sleep(0.1)
    end = time.time()
    log_execution("WorkerC", start, end)
    return {"result": "C_done"}

# Create parallel graph
print("\nCreating graph with 3 parallel branches (100ms each)...")
graph = graph_sp.PyGraph()
graph.add(source_node, "Source", None, [("data", "data")])

branch_a = graph_sp.PyGraph()
branch_a.add(worker_a, "WorkerA[100ms]", [("data", "x")], [("result", "result_a")])

branch_b = graph_sp.PyGraph()
branch_b.add(worker_b, "WorkerB[100ms]", [("data", "x")], [("result", "result_b")])

branch_c = graph_sp.PyGraph()
branch_c.add(worker_c, "WorkerC[100ms]", [("data", "x")], [("result", "result_c")])

graph.branch(branch_a)
graph.branch(branch_b)
graph.branch(branch_c)

dag = graph.build()

print("\nMermaid Diagram:")
mermaid = dag.to_mermaid()
print(mermaid)

# Execute with timing
print("\nExecuting with execute_parallel()...")
execution_log.clear()
overall_start = time.time()
result = dag.execute(parallel=True)
overall_end = time.time()
overall_time = overall_end - overall_start

print(f"\nRuntime Statistics:")
print(f"   Total execution time: {overall_time*1000:.2f}ms")
print(f"   Expected (if parallel): ~100ms")
print(f"   Expected (if sequential): ~300ms")

# Analyze parallelization
if overall_time < 0.25:
    print(f"   PARALLEL execution detected!")
    speedup = 0.3 / overall_time
    print(f"   Speedup: ~{speedup:.1f}x")
else:
    print(f"   SEQUENTIAL execution (current Rust implementation)")
    print(f"   Note: Rust DAG currently executes nodes sequentially")

# Show execution log
if execution_log:
    print(f"\nExecution Log:")
    for entry in execution_log:
        print(f"   {entry['node']:15s} took {entry['duration']*1000:.2f}ms")

print(f"\nResults:")
print(f"   result_a: {result.get('result_a')}")
print(f"   result_b: {result.get('result_b')}")
print(f"   result_c: {result.get('result_c')}")

# Demo 2: Diamond Pattern with Dependencies
print("\n" + "─" * 70)
print("Demo 2: Diamond Pattern (Dependencies)")
print("─" * 70)

def diamond_source(inputs, variant_params):
    """Source for diamond pattern"""
    return {"value": "100"}

def diamond_left(inputs, variant_params):
    """Left path"""
    val = int(inputs.get("x", "0"))
    time.sleep(0.05)
    return {"result": str(val * 2)}

def diamond_right(inputs, variant_params):
    """Right path"""
    val = int(inputs.get("x", "0"))
    time.sleep(0.05)
    return {"result": str(val + 50)}

def diamond_merge(inputs, variant_params):
    """Merge node"""
    left = int(inputs.get("left", "0"))
    right = int(inputs.get("right", "0"))
    return {"final": str(left + right)}

# Create diamond graph
print("\nCreating diamond pattern graph...")
graph2 = graph_sp.PyGraph()
graph2.add(diamond_source, "Source", None, [("value", "data")])

branch_left = graph_sp.PyGraph()
branch_left.add(diamond_left, "Left[*2]", [("data", "x")], [("result", "left_result")])

branch_right = graph_sp.PyGraph()
branch_right.add(diamond_right, "Right[+50]", [("data", "x")], [("result", "right_result")])

graph2.branch(branch_left)
graph2.branch(branch_right)

graph2.add(diamond_merge, "Merge", 
          [("left_result", "left"), ("right_result", "right")],
          [("final", "final")])

dag2 = graph2.build()

print("\nMermaid Diagram:")
print(dag2.to_mermaid())

print("\nExecuting diamond pattern...")
start = time.time()
result2 = dag2.execute(parallel=True)
elapsed = time.time() - start

print(f"\nRuntime: {elapsed*1000:.2f}ms")
print(f"Left path (100*2): {result2.get('left_result')}")
print(f"Right path (100+50): {result2.get('right_result')}")
print(f"Merged (200+150): {result2.get('final')}")

expected = 350
actual = int(result2.get('final', '0'))
if actual == expected:
    print(f"Result verification PASSED")
else:
    print(f"Result verification FAILED: expected {expected}, got {actual}")

# Demo 3: Deep Pipeline
print("\n" + "─" * 70)
print("Demo 3: Deep Sequential Pipeline")
print("─" * 70)

def step1(inputs, variant_params):
    val = int(inputs.get("x", "1"))
    return {"out": str(val * 2)}

def step2(inputs, variant_params):
    val = int(inputs.get("x", "1"))
    return {"out": str(val + 10)}

def step3(inputs, variant_params):
    val = int(inputs.get("x", "1"))
    return {"out": str(val * 3)}

def step4(inputs, variant_params):
    val = int(inputs.get("x", "1"))
    return {"out": str(val - 5)}

print("\nCreating 4-step pipeline: init -> *2 -> +10 -> *3 -> -5")
graph3 = graph_sp.PyGraph()
graph3.add(lambda i, v: {"value": "10"}, "Init", None, [("value", "v")])
graph3.add(step1, "Step1[*2]", [("v", "x")], [("out", "v")])
graph3.add(step2, "Step2[+10]", [("v", "x")], [("out", "v")])
graph3.add(step3, "Step3[*3]", [("v", "x")], [("out", "v")])
graph3.add(step4, "Step4[-5]", [("v", "x")], [("out", "final")])

dag3 = graph3.build()

print("\nMermaid Diagram:")
print(dag3.to_mermaid())

print("\nExecuting pipeline...")
start = time.time()
result3 = dag3.execute()
elapsed = time.time() - start

print(f"\nRuntime: {elapsed*1000:.2f}ms")

# Trace values: 10 -> 20 -> 30 -> 90 -> 85
print(f"Trace:")
print(f"   10 (init) -> *2 = 20")
print(f"   20 -> +10 = 30")
print(f"   30 -> *3 = 90")
print(f"   90 -> -5 = 85")
print(f"   Final result: {result3.get('final')}")

expected = 85
actual = int(result3.get('final', '0'))
if actual == expected:
    print(f"Pipeline verification PASSED")
else:
    print(f"Pipeline verification FAILED: expected {expected}, got {actual}")

print("\n" + "=" * 70)
print("Python Parallel Demo Complete!")
print("\nKey Observations:")
print("- GIL handling works correctly (released during Rust execution)")
print("- Python callables acquire GIL only during their execution")
print("- Current implementation executes nodes level-by-level")
print("- Compare with: cargo run --example parallel_execution_demo")
print("=" * 70)
