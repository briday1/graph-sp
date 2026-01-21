#!/usr/bin/env python3
"""
Pure Python Parallel Execution Demo - GIL Limitations

This demo demonstrates that pure Python computation (without NumPy or
other C extensions) is limited by the Global Interpreter Lock (GIL).

Key insight: Pure Python code holds the GIL, preventing true parallelism.
Multiple threads will mostly run sequentially, with minimal speedup.

This contrasts with:
- time.sleep() which releases the GIL (good parallelism)
- NumPy which releases the GIL (excellent parallelism)
- Pure Python which holds the GIL (poor parallelism)
"""

import dagex
import time
import math

print("=" * 70)
print("  PURE PYTHON PARALLEL EXECUTION - GIL LIMITATIONS")
print("  Pure Python holds GIL → Poor parallelization")
print("=" * 70)

def demo_pure_python_computation():
    """Pure Python computation shows GIL limitations"""
    print("\n" + "─" * 70)
    print("Demo 1: Pure Python Math (4 parallel nodes)")
    print("─" * 70)
    print("🐍 Each node does: loops, conditionals, Python object operations")
    print("   (No NumPy, no C extensions - pure Python only)")

    def source(inputs, params):
        return {"n": 100000}

    def pure_python_worker(inputs, params):
        """CPU-bound pure Python code - GIL is held throughout"""
        n = inputs.get("input", 100000)
        
        # Lots of Python operations that keep the GIL
        result = 0
        for i in range(n):
            # Pure Python math operations
            x = math.sin(i) * math.cos(i)
            y = math.sqrt(abs(x) + 1)
            z = math.log(y + 1)
            result += z
            
            # Python object operations
            temp_list = [x, y, z]
            temp_dict = {"x": x, "y": y, "z": z}
            _ = sum(temp_list) + sum(temp_dict.values())
        
        return {"result": result}

    # Build graph with 4 parallel branches
    graph = dagex.Graph()
    graph.add(source, "Source", None, [("n", "data")])

    for i in range(4):
        branch = dagex.Graph()
        branch.add(
            pure_python_worker,
            f"PythonWorker{i}",
            [("data", "input")],
            [("result", f"result_{i}")]
        )
        graph.branch(branch)

    dag = graph.build()

    print(f"\n📈 Expected behavior:")
    print(f"   Sequential: 4x one computation time")
    print(f"   Parallel:   ~4x (GIL prevents true parallelism)")
    print(f"   Speedup:    ~1.0-1.2x (minimal improvement)")

    # Sequential execution
    print("\n🐌 Sequential Execution (parallel=False):")
    start = time.time()
    dag.execute(parallel=False)
    sequential_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {sequential_time:.1f}ms")

    # Parallel execution
    print("\n⚡ Parallel Execution (parallel=True):")
    start = time.time()
    dag.execute(parallel=True)
    parallel_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {parallel_time:.1f}ms")

    # Analysis
    speedup = sequential_time / parallel_time
    efficiency = (speedup / 4.0) * 100  # 4 parallel nodes
    
    print(f"\n📊 Results:")
    print(f"   Sequential: {sequential_time:.1f}ms")
    print(f"   Parallel:   {parallel_time:.1f}ms")
    print(f"   Speedup:    {speedup:.2f}x")
    print(f"   Efficiency: {efficiency:.1f}% (4 cores)")

    if speedup < 1.3:
        print("   ⚠️  EXPECTED: Minimal speedup due to GIL")
        print("       Pure Python computation holds the GIL")
    elif speedup < 2.0:
        print("   ⚠️  Limited speedup - GIL is the bottleneck")
    else:
        print("   ❓ Unexpected speedup (are you using PyPy or free-threaded Python?)")

def demo_list_operations():
    """Pure Python list operations and comprehensions"""
    print("\n" + "─" * 70)
    print("Demo 2: Pure Python List Operations (4 parallel nodes)")
    print("─" * 70)

    def source(inputs, params):
        return {"size": 50000}

    def list_worker(inputs, params):
        """List operations, comprehensions, filtering"""
        size = inputs.get("input", 50000)
        
        # Create lists with comprehensions
        nums = [i for i in range(size)]
        
        # Filter and map operations
        evens = [x for x in nums if x % 2 == 0]
        squares = [x * x for x in evens]
        
        # Sorting (pure Python comparison operations)
        sorted_data = sorted(squares, reverse=True)
        
        # More list operations
        result = sum(sorted_data[:1000])
        
        return {"result": result}

    # Build graph
    graph = dagex.Graph()
    graph.add(source, "Source", None, [("size", "size")])

    for i in range(4):
        branch = dagex.Graph()
        branch.add(
            list_worker,
            f"ListWorker{i}",
            [("size", "input")],
            [("result", f"result_{i}")]
        )
        graph.branch(branch)

    dag = graph.build()

    # Sequential
    print("\n🐌 Sequential Execution:")
    start = time.time()
    dag.execute(parallel=False)
    sequential_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {sequential_time:.1f}ms")

    # Parallel
    print("\n⚡ Parallel Execution:")
    start = time.time()
    dag.execute(parallel=True)
    parallel_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {parallel_time:.1f}ms")

    # Analysis
    speedup = sequential_time / parallel_time
    print(f"\n📊 Results:")
    print(f"   Sequential: {sequential_time:.1f}ms")
    print(f"   Parallel:   {parallel_time:.1f}ms")
    print(f"   Speedup:    {speedup:.2f}x")
    
    if speedup < 1.5:
        print("   ⚠️  GIL limits parallelization of pure Python code")

def demo_string_processing():
    """Pure Python string operations"""
    print("\n" + "─" * 70)
    print("Demo 3: Pure Python String Processing (3 parallel nodes)")
    print("─" * 70)

    def source(inputs, params):
        # Generate text data
        text = "The quick brown fox jumps over the lazy dog. " * 1000
        return {"text": text}

    def string_worker(inputs, params):
        """String manipulation, parsing, formatting"""
        text = inputs.get("input", "")
        
        # String operations
        words = text.split()
        
        # Process each word
        processed = []
        for word in words:
            # String manipulation
            upper = word.upper()
            lower = word.lower()
            reversed_word = word[::-1]
            
            # String formatting
            formatted = f"{upper}_{lower}_{reversed_word}"
            processed.append(formatted)
        
        # Join and count
        result_text = " ".join(processed)
        char_count = len(result_text)
        word_count = len(processed)
        
        # Dictionary operations
        word_stats = {
            "chars": char_count,
            "words": word_count,
            "avg_length": char_count / word_count if word_count > 0 else 0
        }
        
        return {"result": word_stats["chars"]}

    # Build graph
    graph = dagex.Graph()
    graph.add(source, "Source", None, [("text", "text")])

    for i in range(3):
        branch = dagex.Graph()
        branch.add(
            string_worker,
            f"StringWorker{i}",
            [("text", "input")],
            [("result", f"result_{i}")]
        )
        graph.branch(branch)

    dag = graph.build()

    # Sequential
    print("\n🐌 Sequential Execution:")
    start = time.time()
    dag.execute(parallel=False)
    sequential_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {sequential_time:.1f}ms")

    # Parallel
    print("\n⚡ Parallel Execution:")
    start = time.time()
    dag.execute(parallel=True)
    parallel_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {parallel_time:.1f}ms")

    # Analysis
    speedup = sequential_time / parallel_time
    efficiency = (speedup / 3.0) * 100
    
    print(f"\n📊 Results:")
    print(f"   Sequential: {sequential_time:.1f}ms")
    print(f"   Parallel:   {parallel_time:.1f}ms")
    print(f"   Speedup:    {speedup:.2f}x")
    print(f"   Efficiency: {efficiency:.1f}% (3 cores)")
    
    if speedup < 1.5:
        print("   ⚠️  Limited by GIL as expected for pure Python")

if __name__ == "__main__":
    demo_pure_python_computation()
    demo_list_operations()
    demo_string_processing()
    
    print("\n" + "=" * 70)
    print("  CONCLUSION: Pure Python has limited parallel speedup")
    print("  ")
    print("  Why? The Global Interpreter Lock (GIL) prevents multiple")
    print("  threads from executing Python bytecode simultaneously.")
    print("  ")
    print("  Solutions:")
    print("  • Use NumPy/C extensions (release GIL) ← RECOMMENDED")
    print("  • Use multiprocessing (separate processes)")
    print("  • Use async/await for I/O-bound work")
    print("  • Wait for Python 3.13+ free-threaded mode")
    print("  ")
    print("  For CPU-bound work: NumPy or multiprocessing is the answer!")
    print("=" * 70 + "\n")
