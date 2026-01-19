#!/usr/bin/env python3
"""
Comprehensive example demonstrating all GraphData types in Python.

This example shows how graph-sp can handle multiple data types:
- Integers and floats
- Strings
- Lists of numbers
- Nested structures (dicts/maps)
- Complex data (tuples for complex numbers)
- Arbitrary Python objects (through Map serialization)

The key insight: GraphData is just a transport container. Nodes can process
any data type they want - GraphData doesn't restrict what you can pass,
it just provides a structured way to move data between nodes.
"""

import graph_sp
import json

def data_generator(inputs, variant_params):
    """Generate various data types to demonstrate GraphData flexibility."""
    print("=" * 70)
    print("DataGenerator: Creating diverse data types")
    print("=" * 70)
    
    return {
        # Basic types
        "integer": 42,
        "float": 3.14159,
        "string": "Hello, GraphData!",
        
        # Collections
        "int_list": [1, 2, 3, 4, 5],
        "float_list": [1.1, 2.2, 3.3, 4.4, 5.5],
        
        # Complex numbers (as tuples: real, imag)
        "complex_single": (3.0, 4.0),
        "complex_list": [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)],
        
        # Nested structures (arbitrary Python objects as dicts)
        "metadata": {
            "timestamp": "2026-01-19T03:21:05",
            "version": "1.0.0",
            "author": "graph-sp",
            "config": {
                "mode": "demo",
                "verbose": True,
                "threshold": 0.5
            }
        },
        
        # You can even pass JSON-serializable arbitrary objects
        "custom_object": {
            "type": "sensor_reading",
            "sensor_id": "SENSOR_001",
            "readings": [23.5, 23.7, 23.9, 24.1],
            "status": "nominal",
            "calibration": {
                "offset": 0.1,
                "scale": 1.02
            }
        }
    }

def type_inspector(inputs, variant_params):
    """Inspect and report on all the data types received."""
    print("\n" + "=" * 70)
    print("TypeInspector: Analyzing received data types")
    print("=" * 70)
    
    for key, value in inputs.items():
        print(f"\n{key}:")
        print(f"  Type: {type(value).__name__}")
        print(f"  Value: {value}")
        
        # Show nested structure for dicts
        if isinstance(value, dict):
            print(f"  Structure:")
            for k, v in value.items():
                print(f"    {k}: {type(v).__name__} = {v}")
    
    # Pass everything through unchanged
    return inputs

def data_processor(inputs, variant_params):
    """Process different data types to show GraphData doesn't restrict operations."""
    print("\n" + "=" * 70)
    print("DataProcessor: Processing multiple data types")
    print("=" * 70)
    
    results = {}
    
    # Process integer
    if "integer" in inputs:
        results["integer_doubled"] = inputs["integer"] * 2
        print(f"Integer: {inputs['integer']} → doubled → {results['integer_doubled']}")
    
    # Process float
    if "float" in inputs:
        results["float_squared"] = inputs["float"] ** 2
        print(f"Float: {inputs['float']:.5f} → squared → {results['float_squared']:.5f}")
    
    # Process string
    if "string" in inputs:
        results["string_upper"] = inputs["string"].upper()
        print(f"String: '{inputs['string']}' → upper → '{results['string_upper']}'")
    
    # Process list
    if "float_list" in inputs:
        float_list = inputs["float_list"]
        results["list_sum"] = sum(float_list)
        results["list_avg"] = sum(float_list) / len(float_list)
        print(f"Float List: {float_list}")
        print(f"  Sum: {results['list_sum']:.2f}, Average: {results['list_avg']:.2f}")
    
    # Process complex numbers
    if "complex_single" in inputs:
        r, i = inputs["complex_single"]
        magnitude = (r**2 + i**2) ** 0.5
        results["complex_magnitude"] = magnitude
        print(f"Complex: ({r}, {i}) → magnitude → {magnitude:.3f}")
    
    # Process nested structure
    if "metadata" in inputs:
        meta = inputs["metadata"]
        results["meta_summary"] = f"Version {meta.get('version', 'unknown')} by {meta.get('author', 'unknown')}"
        print(f"Metadata: {results['meta_summary']}")
    
    # Process custom object
    if "custom_object" in inputs:
        obj = inputs["custom_object"]
        readings = obj.get("readings", [])
        if readings:
            results["sensor_avg"] = sum(readings) / len(readings)
            print(f"Sensor {obj.get('sensor_id', 'unknown')}: Average reading = {results['sensor_avg']:.2f}")
    
    return results

def result_aggregator(inputs, variant_params):
    """Aggregate all results and create a summary."""
    print("\n" + "=" * 70)
    print("ResultAggregator: Creating final summary")
    print("=" * 70)
    
    # Count different types of results
    counts = {
        "numeric": 0,
        "string": 0,
        "other": 0
    }
    
    for key, value in inputs.items():
        if isinstance(value, (int, float)):
            counts["numeric"] += 1
        elif isinstance(value, str):
            counts["string"] += 1
        else:
            counts["other"] += 1
    
    summary = {
        "total_outputs": len(inputs),
        "type_counts": counts,
        "all_keys": list(inputs.keys())
    }
    
    print(f"\nSummary:")
    print(f"  Total outputs: {summary['total_outputs']}")
    print(f"  Numeric results: {counts['numeric']}")
    print(f"  String results: {counts['string']}")
    print(f"  Other results: {counts['other']}")
    print(f"  Keys: {', '.join(summary['all_keys'])}")
    
    return {
        "summary": summary,
        "success": True
    }

def main():
    print("\n" + "=" * 70)
    print("GraphData Multiple Data Types Demo")
    print("=" * 70)
    print("\nThis demo shows that GraphData supports ANY data type you want!")
    print("The graph executor doesn't care about the data - it just passes it.")
    print("Your node functions can work with any Python objects.")
    print("=" * 70)
    
    # Build the graph
    graph = graph_sp.PyGraph()
    
    # Node 1: Generate diverse data types
    graph.add(
        function=data_generator,
        label="DataGenerator",
        inputs=None,
        outputs=[
            ("integer", "int_val"),
            ("float", "float_val"),
            ("string", "str_val"),
            ("int_list", "int_list"),
            ("float_list", "float_list"),
            ("complex_single", "complex_val"),
            ("complex_list", "complex_list"),
            ("metadata", "meta"),
            ("custom_object", "custom")
        ]
    )
    
    # Node 2: Inspect types (receives all outputs from Node 1)
    graph.add(
        function=type_inspector,
        label="TypeInspector",
        inputs=[
            ("int_val", "integer"),
            ("float_val", "float"),
            ("str_val", "string"),
            ("int_list", "int_list"),
            ("float_list", "float_list"),
            ("complex_val", "complex_single"),
            ("complex_list", "complex_list"),
            ("meta", "metadata"),
            ("custom", "custom_object")
        ],
        outputs=[
            ("integer", "inspected_int"),
            ("float", "inspected_float"),
            ("string", "inspected_str"),
            ("float_list", "inspected_list"),
            ("complex_single", "inspected_complex"),
            ("metadata", "inspected_meta"),
            ("custom_object", "inspected_custom")
        ]
    )
    
    # Node 3: Process the data
    graph.add(
        function=data_processor,
        label="DataProcessor",
        inputs=[
            ("inspected_int", "integer"),
            ("inspected_float", "float"),
            ("inspected_str", "string"),
            ("inspected_list", "float_list"),
            ("inspected_complex", "complex_single"),
            ("inspected_meta", "metadata"),
            ("inspected_custom", "custom_object")
        ],
        outputs=[
            ("integer_doubled", "result_int"),
            ("float_squared", "result_float"),
            ("string_upper", "result_str"),
            ("list_sum", "result_sum"),
            ("list_avg", "result_avg"),
            ("complex_magnitude", "result_mag"),
            ("meta_summary", "result_meta"),
            ("sensor_avg", "result_sensor")
        ]
    )
    
    # Node 4: Aggregate results
    graph.add(
        function=result_aggregator,
        label="ResultAggregator",
        inputs=[
            ("result_int", "integer_doubled"),
            ("result_float", "float_squared"),
            ("result_str", "string_upper"),
            ("result_sum", "list_sum"),
            ("result_avg", "list_avg"),
            ("result_mag", "complex_magnitude"),
            ("result_meta", "meta_summary"),
            ("result_sensor", "sensor_avg")
        ],
        outputs=[
            ("summary", "final_summary"),
            ("success", "status")
        ]
    )
    
    # Build and execute
    print("\nBuilding DAG...")
    dag = graph.build()
    
    print("\n" + "=" * 70)
    print("Mermaid Diagram")
    print("=" * 70)
    print(dag.to_mermaid())
    
    print("\n" + "=" * 70)
    print("Executing Graph")
    print("=" * 70)
    result = dag.execute(parallel=False)
    
    print("\n" + "=" * 70)
    print("Final Results")
    print("=" * 70)
    if "final_summary" in result:
        print(f"\nExecution successful! Summary: {result['final_summary']}")
    print(f"\nStatus: {result.get('status', 'unknown')}")
    
    print("\n" + "=" * 70)
    print("Key Takeaway")
    print("=" * 70)
    print("""
GraphData is a TRANSPORT container, not a type restriction!

You can pass ANY Python object through the graph:
- Built-in types (int, float, str, list, dict)
- Complex numbers (as tuples)
- Nested structures (dicts of dicts)
- Custom objects (as dicts with your data)
- Even numpy arrays, pandas DataFrames, or ANY serializable object!

The graph executor doesn't care what's in the container.
Your node functions decide what to do with the data.
This gives you complete flexibility!
    """)

if __name__ == "__main__":
    main()
