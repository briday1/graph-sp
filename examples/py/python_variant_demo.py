#!/usr/bin/env python3
"""
Python Variant Demo - Parameter Sweeps with List Comprehensions

Demonstrates using graph.variant() with list comprehensions for clean parameter sweeps.
"""

import dagex

print("=" * 70)
print("Python Variant Demo - dagex")
print("=" * 70)

# Demo 1: Simple parameter sweep with list comprehension
print("\n" + "─" * 70)
print("Demo 1: List Comprehension Variant Sweep")
print("─" * 70)

graph = dagex.Graph()

# Source node
graph.add(
    lambda inputs, params: {"value": 10},
    "Source",
    None,
    [("value", "data")]
)

# Variant sweep: multiply by different factors
factors = [2, 3, 5, 10]
graph.variant(
    [lambda inputs, params, f=f: {"result": inputs["x"] * f} for f in factors],
    "Scale",
    [("data", "x")],
    [("result", "scaled")]
)

dag = graph.build()
context = dag.execute()

print(f"\nResults:")
print(f"  Input: {context['data']}")
print(f"  Factors: {factors}")
print(f"  Output (last variant): {context['scaled']}")
print(f"\nMermaid:\n{dag.to_mermaid()}")

# Demo 2: Numpy linspace parameter sweep
print("\n" + "─" * 70)
print("Demo 2: Numpy Linspace Parameter Sweep")
print("─" * 70)

try:
    import numpy as np
    
    graph2 = dagex.Graph()
    
    graph2.add(
        lambda inputs, params: {"base": 100},
        "Source",
        None,
        [("base", "value")]
    )
    
    # Sweep over a range of scaling factors
    factors = np.linspace(0.5, 2.0, 5)
    graph2.variant(
        [lambda inputs, params, f=f: {"scaled": int(inputs["v"] * f)} for f in factors],
        "LinearScale",
        [("value", "v")],
        [("scaled", "result")]
    )
    
    dag2 = graph2.build()
    context2 = dag2.execute()
    
    print(f"\nUsed {len(factors)} factors from {factors[0]:.2f} to {factors[-1]:.2f}")
    print(f"Base value: {context2['value']}")
    print(f"Final result: {context2['result']}")
    
except ImportError:
    print("NumPy not available, skipping this demo")

# Demo 3: Power function variants
print("\n" + "─" * 70)
print("Demo 3: Power Function Variants")
print("─" * 70)

graph3 = dagex.Graph()

graph3.add(
    lambda inputs, params: {"x": 2},
    "Source",
    None,
    [("x", "number")]
)

# Variant: different power functions
exponents = [2, 3, 4, 5]
graph3.variant(
    [lambda inputs, params, exp=exp: {"powered": inputs["n"] ** exp} for exp in exponents],
    "Power",
    [("number", "n")],
    [("powered", "result")]
)

dag3 = graph3.build()
context3 = dag3.execute()

print(f"\nBase: {context3['number']}")
print(f"Exponents: {exponents}")
print(f"Result (last variant, 2^5): {context3['result']}")

print("\n" + "=" * 70)
print("Demo Complete!")
print("=" * 70)
