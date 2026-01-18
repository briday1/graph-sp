#!/usr/bin/env python3
"""
Example demonstrating Python bindings for graph-sp

This example shows how to use the graph-sp library from Python,
including creating graphs, adding nodes with Python functions,
and executing the DAG.
"""

import graph_sp

def data_source(inputs, variant_params):
    """
    Simple data source that produces initial values.
    
    Args:
        inputs: Dictionary of input variables (empty for source nodes)
        variant_params: Dictionary of variant parameters
    
    Returns:
        Dictionary with output variables
    """
    print("🔹 Data source executing...")
    return {"output": "100"}

def processor(inputs, variant_params):
    """
    Process the input data.
    
    Args:
        inputs: Dictionary with 'input' key containing the value to process
        variant_params: Dictionary of variant parameters
    
    Returns:
        Dictionary with 'result' key containing processed value
    """
    print(f"🔹 Processor executing with inputs: {inputs}")
    input_val = inputs.get("input", "0")
    result = str(int(input_val) * 2)
    return {"result": result}

def formatter(inputs, variant_params):
    """
    Format the result for display.
    
    Args:
        inputs: Dictionary with 'value' key
        variant_params: Dictionary of variant parameters
    
    Returns:
        Dictionary with 'formatted' key
    """
    print(f"🔹 Formatter executing with inputs: {inputs}")
    value = inputs.get("value", "0")
    return {"formatted": f"Result: {value}"}

def main():
    print("=" * 70)
    print("graph-sp Python Bindings Example")
    print("=" * 70)
    
    # Create a new graph
    print("\n📊 Creating graph...")
    graph = graph_sp.PyGraph()
    
    # Add source node
    # outputs: (impl_var, broadcast_var) - function returns "output", stored as "data"
    print("➕ Adding source node...")
    graph.add(
        function=data_source,
        label="Source",
        inputs=None,
        outputs=[("output", "data")]
    )
    
    # Add processor node
    # inputs: (broadcast_var, impl_var) - "data" from context becomes "input" in function
    # outputs: (impl_var, broadcast_var) - "result" from function becomes "final" in context
    print("➕ Adding processor node...")
    graph.add(
        function=processor,
        label="Processor",
        inputs=[("data", "input")],
        outputs=[("result", "final")]
    )
    
    # Add formatter node
    print("➕ Adding formatter node...")
    graph.add(
        function=formatter,
        label="Formatter",
        inputs=[("final", "value")],
        outputs=[("formatted", "display")]
    )
    
    # Build the DAG
    print("\n🔧 Building DAG...")
    dag = graph.build()
    
    # Execute the DAG
    print("\n▶️  Executing DAG sequentially...")
    result = dag.execute()
    
    print("\n✅ Execution complete!")
    print(f"📤 Final context: {result}")
    print(f"📍 Display value: {result.get('display')}")
    
    # Test parallel execution
    print("\n▶️  Executing DAG with parallel execution...")
    result2 = dag.execute_parallel()
    print(f"📤 Parallel result: {result2}")
    
    # Generate Mermaid diagram
    print("\n📈 Generating Mermaid diagram...")
    mermaid = dag.to_mermaid()
    print("\n" + mermaid)
    
    print("\n" + "=" * 70)
    print("✨ Example completed successfully!")
    print("=" * 70)

if __name__ == "__main__":
    main()
