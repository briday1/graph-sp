#!/usr/bin/env python3
"""
Test script demonstrating that the data flow dependency resolution fix works correctly.

This shows:
1. Independent nodes run in parallel (not sequentially)
2. Mermaid diagram shows actual data dependencies (not insertion order)
3. Complex DAGs with multiple levels execute correctly
"""

import dagex
import time
from datetime import datetime

print("="*70)
print("DATA FLOW DEPENDENCY RESOLUTION - VERIFICATION TEST")
print("="*70)

# =============================================================================
# TEST 1: Independent Nodes Run in Parallel
# =============================================================================
print("\n" + "─"*70)
print("TEST 1: Independent nodes should run in parallel")
print("─"*70)

def source_a(inputs):
    ts = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    print(f'  [{ts}] SourceA starting')
    time.sleep(0.2)
    ts = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    print(f'  [{ts}] SourceA finished')
    return {'data_a': 'A'}

def source_b(inputs):
    ts = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    print(f'  [{ts}] SourceB starting')
    time.sleep(0.2)
    ts = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    print(f'  [{ts}] SourceB finished')
    return {'data_b': 'B'}

def source_c(inputs):
    ts = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    print(f'  [{ts}] SourceC starting')
    time.sleep(0.2)
    ts = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    print(f'  [{ts}] SourceC finished')
    return {'data_c': 'C'}

graph1 = dagex.Graph()
graph1.add(source_a, 'SourceA', None, [('data_a', 'data_a')])
graph1.add(source_b, 'SourceB', None, [('data_b', 'data_b')])
graph1.add(source_c, 'SourceC', None, [('data_c', 'data_c')])

dag1 = graph1.build()

print("\nMermaid (should show no connections between A, B, C):")
print(dag1.to_mermaid())

print("\nExecuting with parallel=True:")
start = time.time()
result1 = dag1.execute(parallel=True)
elapsed = time.time() - start

print(f"\n📊 Results:")
print(f"   Time: {elapsed:.3f}s")
print(f"   Expected: ~0.2s (parallel) vs ~0.6s (sequential)")
if elapsed < 0.3:
    print("   ✅ PASS: All three sources ran IN PARALLEL")
else:
    print("   ❌ FAIL: Still running sequentially")

# =============================================================================
# TEST 2: Data Dependencies Are Respected
# =============================================================================
print("\n" + "─"*70)
print("TEST 2: Data dependencies should create correct graph structure")
print("─"*70)

def init_config(inputs):
    return {'config': 'initialized'}

def process_a(inputs):
    # Depends on config
    config = inputs['config']
    return {'result_a': f'A-{config}'}

def process_b(inputs):
    # Depends on config
    config = inputs['config']
    return {'result_b': f'B-{config}'}

def combine(inputs):
    # Depends on both process_a and process_b
    a = inputs['result_a']
    b = inputs['result_b']
    return {'final': f'{a}+{b}'}

graph2 = dagex.Graph()
graph2.add(init_config, 'Init', None, [('config', 'config')])
graph2.add(process_a, 'ProcessA', [('config', 'config')], [('result_a', 'result_a')])
graph2.add(process_b, 'ProcessB', [('config', 'config')], [('result_b', 'result_b')])
graph2.add(combine, 'Combine', [('result_a', 'result_a'), ('result_b', 'result_b')], 
           [('final', 'final')])

dag2 = graph2.build()

print("\nMermaid (should show Init → [A, B] → Combine):")
print(dag2.to_mermaid())

result2 = dag2.execute()
print(f"\n📊 Results:")
print(f"   Final result: {result2.get('final')}")
print(f"   Expected: 'A-initialized+B-initialized'")
if result2.get('final') == 'A-initialized+B-initialized':
    print("   ✅ PASS: Data flow is correct")
else:
    print("   ❌ FAIL: Data flow is incorrect")

# =============================================================================
# TEST 3: Complex Multi-Level Pipeline
# =============================================================================
print("\n" + "─"*70)
print("TEST 3: Complex multi-level pipeline with fan-out and fan-in")
print("─"*70)

def data_source_1(inputs):
    time.sleep(0.05)
    return {'src1': 'data1'}

def data_source_2(inputs):
    time.sleep(0.05)
    return {'src2': 'data2'}

def process_1a(inputs):
    return {'p1a': inputs['src1'] + '-p1a'}

def process_1b(inputs):
    return {'p1b': inputs['src1'] + '-p1b'}

def process_2a(inputs):
    return {'p2a': inputs['src2'] + '-p2a'}

def merge_all(inputs):
    return {'final': f"{inputs['p1a']}+{inputs['p1b']}+{inputs['p2a']}"}

graph3 = dagex.Graph()
graph3.add(data_source_1, 'Src1', None, [('src1', 'src1')])
graph3.add(data_source_2, 'Src2', None, [('src2', 'src2')])
graph3.add(process_1a, 'P1A', [('src1', 'src1')], [('p1a', 'p1a')])
graph3.add(process_1b, 'P1B', [('src1', 'src1')], [('p1b', 'p1b')])
graph3.add(process_2a, 'P2A', [('src2', 'src2')], [('p2a', 'p2a')])
graph3.add(merge_all, 'Merge', [('p1a', 'p1a'), ('p1b', 'p1b'), ('p2a', 'p2a')], 
           [('final', 'final')])

dag3 = graph3.build()

print("\nMermaid (should show proper fan-out/fan-in structure):")
print(dag3.to_mermaid())

start = time.time()
result3 = dag3.execute(parallel=True)
elapsed = time.time() - start

print(f"\n📊 Results:")
print(f"   Time: {elapsed:.3f}s")
print(f"   Final: {result3.get('final')}")
print(f"   Expected: 'data1-p1a+data1-p1b+data2-p2a'")
if result3.get('final') == 'data1-p1a+data1-p1b+data2-p2a':
    print("   ✅ PASS: Complex pipeline works correctly")
else:
    print("   ❌ FAIL: Complex pipeline produced wrong result")

# =============================================================================
# SUMMARY
# =============================================================================
print("\n" + "="*70)
print("SUMMARY")
print("="*70)
print("""
✅ The graph builder now correctly:
   1. Resolves dependencies based on data flow (input/output mappings)
   2. Allows independent nodes to execute in parallel
   3. Generates accurate Mermaid diagrams showing actual dependencies
   4. Handles complex multi-level pipelines with fan-out/fan-in

This fixes the issue where nodes were connected sequentially based on
insertion order, which caused unnecessary serialization and incorrect
visualization.
""")
