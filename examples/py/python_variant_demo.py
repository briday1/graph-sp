#!/usr/bin/env python3
"""
Python Variant Demo - Parameter Sweep

Demonstrates the variant() method for creating parameter sweeps
where multiple node variants execute in parallel with different parameters.
"""

import dagex

print("=" * 70)
print("Python Variant Demo - dagex")
print("Parameter Sweep and Variant Execution")
print("=" * 70)

# Demo 1: Simple parameter sweep
print("\n" + "─" * 70)
print("Demo 1: Simple Parameter Sweep")
print("─" * 70)

def data_source(inputs, variant_params):
    """Source node that produces initial data"""
    return {"value": 100}

def make_multiplier(factor):
    """Factory function that creates a multiplier node with given factor"""
    def multiplier(inputs, variant_params):
        val = inputs.get("x", 0)
        # Access the parameter from variant_params
        param_val = variant_params.get("param_value", factor)
        result = val * factor
        print(f"  Variant with factor {factor}: {val} * {factor} = {result}")
        return {"result": result}
    return multiplier

graph = dagex.Graph()

# Add source
graph.add(
    function=data_source,
    label="DataSource",
    inputs=None,
    outputs=[("value", "data")]
)

# Create variants with different multiplier factors
graph.variant(
    factory=make_multiplier,
    param_values=[0.5, 1.0, 2.0, 3.0, 5.0],
    label="Multiply",
    inputs=[("data", "x")],
    outputs=[("result", "result")]
)

dag = graph.build()

print("\nMermaid Diagram:")
print(dag.to_mermaid())

print("\nExecuting variants...")
context = dag.execute()

print(f"\nFinal context (last variant overwrites 'result'):")
print(f"  result = {context.get('result', 'N/A')}")

# Demo 2: Scaling with different factors
print("\n" + "─" * 70)
print("Demo 2: Data Processing with Multiple Scalers")
print("─" * 70)

def source_data(inputs, variant_params):
    """Generate initial dataset"""
    return {"dataset": [10, 20, 30, 40, 50]}

def make_scaler(scale_factor):
    """Factory that creates a scaler node"""
    def scaler(inputs, variant_params):
        data = inputs.get("data", [])
        scaled = [x * scale_factor for x in data]
        avg = sum(scaled) / len(scaled) if scaled else 0
        return {
            "scaled_data": scaled,
            "average": avg,
            "scale_factor": scale_factor
        }
    return scaler

graph2 = dagex.Graph()

graph2.add(
    function=source_data,
    label="Source",
    inputs=None,
    outputs=[("dataset", "raw_data")]
)

# Create variants with different scale factors
graph2.variant(
    factory=make_scaler,
    param_values=[0.1, 0.5, 1.0, 2.0, 10.0],
    label="Scale",
    inputs=[("raw_data", "data")],
    outputs=[
        ("scaled_data", "scaled"),
        ("average", "avg"),
        ("scale_factor", "factor")
    ]
)

dag2 = graph2.build()

print("\nMermaid Diagram:")
print(dag2.to_mermaid())

print("\nExecuting scalers...")
context2 = dag2.execute()

print(f"\nResults (last variant):")
print(f"  scaled = {context2.get('scaled', 'N/A')}")
print(f"  average = {context2.get('avg', 'N/A')}")
print(f"  factor = {context2.get('factor', 'N/A')}")

# Demo 3: String parameter variants
print("\n" + "─" * 70)
print("Demo 3: String Parameter Variants")
print("─" * 70)

def text_source(inputs, variant_params):
    """Source text"""
    return {"text": "hello world"}

def make_transformer(operation):
    """Factory that creates text transformation nodes"""
    def transformer(inputs, variant_params):
        text = inputs.get("input", "")
        
        if operation == "upper":
            result = text.upper()
        elif operation == "lower":
            result = text.lower()
        elif operation == "title":
            result = text.title()
        elif operation == "reverse":
            result = text[::-1]
        elif operation == "capitalize":
            result = text.capitalize()
        else:
            result = text
            
        print(f"  {operation}: '{text}' -> '{result}'")
        return {"transformed": result}
    return transformer

graph3 = dagex.Graph()

graph3.add(
    function=text_source,
    label="TextSource",
    inputs=None,
    outputs=[("text", "original")]
)

# Create variants with different string operations
graph3.variant(
    factory=make_transformer,
    param_values=["upper", "lower", "title", "reverse", "capitalize"],
    label="Transform",
    inputs=[("original", "input")],
    outputs=[("transformed", "output")]
)

dag3 = graph3.build()

print("\nMermaid Diagram:")
print(dag3.to_mermaid())

print("\nExecuting transformers...")
context3 = dag3.execute()

print(f"\nFinal output: '{context3.get('output', 'N/A')}'")

print("\n" + "=" * 70)
print("Python Variant Demo Complete!")
print("=" * 70)
