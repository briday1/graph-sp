#!/usr/bin/env python3
"""
Demo of Linspace, Logspace, and Geomspace helpers for parameter sweeps.

These helpers generate evenly-spaced values with different spacing patterns:
- Linspace: Linear spacing (arithmetic progression)
- Logspace: Logarithmic spacing (good for exponential ranges)
- Geomspace: Geometric progression (multiply by constant ratio)
"""

print("="*70)
print("PARAMETER SWEEP HELPERS DEMO")
print("="*70)

# =============================================================================
# What Problem Do These Solve?
# =============================================================================

print("\n" + "─"*70)
print("THE PROBLEM: Testing Multiple Parameter Values")
print("─"*70)

print("""
Imagine you're training a machine learning model and want to test different
learning rates to find the best one. You could write:

    learning_rates = [0.001, 0.01, 0.1, 1.0]

But what if you want to test 20 values? Or 100? 
What if you want them evenly spaced on a LOG scale (better for learning rates)?

That's where these helpers come in!
""")

# =============================================================================
# 1. LINSPACE - Linear Spacing
# =============================================================================

print("\n" + "─"*70)
print("1. LINSPACE - Linear (Arithmetic) Spacing")
print("─"*70)

print("""
LINSPACE creates evenly-spaced values on a LINEAR scale.

Example: Linspace(start=0.0, end=1.0, count=5)
Result: [0.0, 0.25, 0.5, 0.75, 1.0]

Formula: value[i] = start + i * (end - start) / (count - 1)
""")

print("Python equivalent:")
print("  import numpy as np")
print("  values = np.linspace(0.0, 1.0, 5)")
print("  # [0.0, 0.25, 0.5, 0.75, 1.0]")

print("\n✅ USE LINSPACE FOR:")
print("  - Temperature ranges: 20°C to 100°C in 10 steps")
print("  - Time intervals: 0s to 10s in 100 steps")
print("  - Uniformly distributed parameters")
print("  - Grid search with equal spacing")

# =============================================================================
# 2. LOGSPACE - Logarithmic Spacing
# =============================================================================

print("\n" + "─"*70)
print("2. LOGSPACE - Logarithmic Spacing")
print("─"*70)

print("""
LOGSPACE creates evenly-spaced values on a LOGARITHMIC scale.
This means equal RATIOS between consecutive values.

Example: Logspace(start=1.0, end=1000.0, count=4)
Result: [1.0, 10.0, 100.0, 1000.0]

Notice: 10/1 = 10, 100/10 = 10, 1000/100 = 10 (equal ratios!)

Formula: value[i] = exp(ln(start) + i * (ln(end) - ln(start)) / (count - 1))
""")

print("Python equivalent:")
print("  import numpy as np")
print("  values = np.logspace(np.log10(1), np.log10(1000), 4)")
print("  # [1.0, 10.0, 100.0, 1000.0]")

print("\n✅ USE LOGSPACE FOR:")
print("  - Learning rates: 0.001 to 1.0 (spans orders of magnitude)")
print("  - Regularization parameters: 1e-6 to 1e-1")
print("  - Frequency ranges in signal processing")
print("  - Any parameter that varies exponentially")

# =============================================================================
# 3. GEOMSPACE - Geometric Progression
# =============================================================================

print("\n" + "─"*70)
print("3. GEOMSPACE - Geometric Progression")
print("─"*70)

print("""
GEOMSPACE creates values in geometric progression (multiply by constant).

Example: Geomspace(start=2.0, ratio=3.0, count=5)
Result: [2.0, 6.0, 18.0, 54.0, 162.0]

Notice: 6/2 = 3, 18/6 = 3, 54/18 = 3, 162/54 = 3 (constant ratio!)

Formula: value[i] = start * ratio^i
""")

print("Python equivalent:")
print("  values = [2.0 * (3.0 ** i) for i in range(5)]")
print("  # [2.0, 6.0, 18.0, 54.0, 162.0]")

print("\n✅ USE GEOMSPACE FOR:")
print("  - Exponential growth/decay: populations, radioactive decay")
print("  - Batch sizes: 32, 64, 128, 256, 512 (ratio=2)")
print("  - Powers of 2 or other bases")
print("  - Compound interest calculations")

# =============================================================================
# Visual Comparison
# =============================================================================

print("\n" + "─"*70)
print("VISUAL COMPARISON: Same Range, Different Spacing")
print("─"*70)

print("\nGenerating 5 values from 1 to 1000:")
print()

# Linspace
print("LINSPACE(1, 1000, 5):")
linspace_vals = [1.0 + i * (1000.0 - 1.0) / 4 for i in range(5)]
for i, v in enumerate(linspace_vals):
    print(f"  [{i}] {v:8.1f}   Step size: {v - linspace_vals[i-1] if i > 0 else 0:.1f}")

print()

# Logspace
import math
print("LOGSPACE(1, 1000, 5):")
log_start = math.log(1.0)
log_end = math.log(1000.0)
logspace_vals = [math.exp(log_start + i * (log_end - log_start) / 4) for i in range(5)]
for i, v in enumerate(logspace_vals):
    ratio = v / logspace_vals[i-1] if i > 0 else 1.0
    print(f"  [{i}] {v:8.1f}   Ratio: {ratio:.1f}x")

print()

# Geomspace
print("GEOMSPACE(1, ratio=5.62, 5):")  # ratio chosen to reach ~1000
geomspace_vals = [1.0 * (5.62 ** i) for i in range(5)]
for i, v in enumerate(geomspace_vals):
    ratio = v / geomspace_vals[i-1] if i > 0 else 1.0
    print(f"  [{i}] {v:8.1f}   Ratio: {ratio:.1f}x")

# =============================================================================
# Real-World Example: Hyperparameter Tuning
# =============================================================================

print("\n" + "─"*70)
print("REAL-WORLD EXAMPLE: Hyperparameter Tuning")
print("─"*70)

print("""
You're tuning a neural network and want to test:
- Learning rates from 0.0001 to 0.1 (4 orders of magnitude!)
- Batch sizes that are powers of 2
- Dropout rates uniformly distributed

Here's how you'd use each helper:
""")

print("1. Learning Rates (LOGSPACE - spans orders of magnitude):")
print("   Rust: Logspace::new(0.0001, 0.1, 5)")
print("   Values: [0.0001, 0.001, 0.01, 0.1]")
print()

print("2. Batch Sizes (GEOMSPACE - powers of 2):")
print("   Rust: Geomspace::new(16.0, 2.0, 6)")
print("   Values: [16, 32, 64, 128, 256, 512]")
print()

print("3. Dropout Rates (LINSPACE - uniform distribution):")
print("   Rust: Linspace::new(0.0, 0.5, 6)")
print("   Values: [0.0, 0.1, 0.2, 0.3, 0.4, 0.5]")

# =============================================================================
# How They Work in dagex
# =============================================================================

print("\n" + "─"*70)
print("HOW THEY WORK IN DAGEX")
print("─"*70)

print("""
In dagex, these helpers are used with the .variant() API to create
multiple copies of a node with different parameter values.

Example (Rust):
    let mut graph = Graph::new();
    
    graph.add(load_data, Some("LoadData"), None, 
              Some(vec![("data", "dataset")]));
    
    // Create 10 training variants with different learning rates
    graph.variant("learning_rate", Logspace::new(0.001, 0.1, 10));
    
    graph.add(train_model, Some("Train"), 
              Some(vec![("dataset", "data")]),
              Some(vec![("model", "trained_model")]));
    
    let dag = graph.build();

This creates 10 parallel copies of the Train node, each with a different
learning_rate parameter. All 10 train in PARALLEL!

The Mermaid diagram will show:
    LoadData → [Train(v0), Train(v1), ..., Train(v9)]
    
Each variant is color-coded and executes independently.
""")

# =============================================================================
# Summary
# =============================================================================

print("\n" + "="*70)
print("SUMMARY")
print("="*70)

print("""
┌─────────────┬──────────────────────┬─────────────────────────────┐
│ Helper      │ Pattern              │ Best For                    │
├─────────────┼──────────────────────┼─────────────────────────────┤
│ Linspace    │ Equal steps          │ Uniform distributions       │
│             │ (arithmetic)         │ Time series, temperatures   │
├─────────────┼──────────────────────┼─────────────────────────────┤
│ Logspace    │ Equal ratios         │ Exponential ranges          │
│             │ (logarithmic)        │ Learning rates, frequencies │
├─────────────┼──────────────────────┼─────────────────────────────┤
│ Geomspace   │ Constant multiplier  │ Powers of N                 │
│             │ (geometric)          │ Batch sizes, growth rates   │
└─────────────┴──────────────────────┴─────────────────────────────┘

💡 KEY INSIGHT:
These helpers make it EASY to create parameter sweeps for hyperparameter
tuning, sensitivity analysis, and grid search WITHOUT writing loops or
manually calculating values.

They integrate seamlessly with dagex's variant system to create parallel
execution of multiple parameter configurations!
""")

print("\n" + "="*70)
print("Run this in a Rust project or check the integration tests to see")
print("these helpers in action with actual dagex code!")
print("="*70 + "\n")
