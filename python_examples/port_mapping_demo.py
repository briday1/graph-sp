"""
Port Mapping Demo
=================

This example demonstrates the difference between broadcast names and implementation names:
- broadcast_name: External name used for connecting nodes via edges
- impl_name: Internal parameter name used within the function

This allows you to have different internal parameter names than the port names used
for connections, which is useful when:
1. You want to reuse existing functions with specific parameter names
2. You need more descriptive internal names without changing the connection interface
3. You're working with multiple functions that have conflicting parameter names
"""

import pygraph_sp as gs


def data_source():
    """Produces data with a specific output port name."""
    return {"data": 42}


def process_data(input_value):
    """
    Process function that expects 'input_value' as parameter name.
    But we'll connect it via the 'data' broadcast name.
    """
    print(f"Processing: {input_value}")
    return {"output_data": input_value * 2}


def display_result(final_value):
    """
    Display function that expects 'final_value' as parameter name.
    But we'll connect it via the 'output_data' broadcast name.
    """
    print(f"Final result: {final_value}")
    return {}


def main():
    print("=== Port Mapping Demo ===\n")
    
    # Create graph
    graph = gs.Graph()
    
    # Add source node - outputs to "data" port
    graph.add(
        data_source,
        label="Data Source",
        outputs=["data"]
    )
    
    # Add process node with port mapping
    # - Connects via "data" broadcast name (matches source output)
    # - Maps to "input_value" parameter inside the function
    # - Outputs via "output_data" broadcast name
    # - Maps from "output_data" return key in the function
    graph.add(
        process_data,
        label="Processor",
        inputs=[("data", "input_value")],  # (broadcast_name, impl_name)
        outputs=[("output_data", "output_data")]  # Can also use tuple for outputs
    )
    
    # Add display node with port mapping
    # - Connects via "output_data" broadcast name (matches processor output)
    # - Maps to "final_value" parameter inside the function
    graph.add(
        display_result,
        label="Display",
        inputs=[("output_data", "final_value")]  # (broadcast_name, impl_name)
    )
    
    # Add edges using broadcast names
    graph.add_edge("data_source", "data", "process_data", "data")
    graph.add_edge("process_data", "output_data", "display_result", "output_data")
    
    # Visualize the graph
    print("Graph structure:")
    print(gs.Inspector.visualize(graph))
    print()
    
    # Execute
    print("Executing graph...")
    executor = gs.Executor()
    result = executor.execute(graph)
    
    if result.is_success():
        print("\n✓ Execution successful!")
    else:
        print("\n✗ Execution failed")
        for node_id, error in result.errors.items():
            print(f"  - {node_id}: {error}")


def demo_simple_ports():
    """
    Demo showing simple port syntax where broadcast_name == impl_name.
    This is the most common case.
    """
    print("\n=== Simple Port Demo ===\n")
    
    def add_numbers(a, b):
        """Simple function with matching port and parameter names."""
        return {"sum": a + b}
    
    def multiply(sum):
        """Uses the 'sum' parameter."""
        return {"result": sum * 10}
    
    graph = gs.Graph()
    
    # When using simple strings, broadcast_name and impl_name are the same
    graph.add(
        add_numbers,
        label="Adder",
        inputs=["a", "b"],  # Both broadcast and impl names are "a" and "b"
        outputs=["sum"]
    )
    
    graph.add(
        multiply,
        label="Multiplier",
        inputs=["sum"],  # Both broadcast and impl names are "sum"
        outputs=["result"]
    )
    
    # Auto-connect based on matching broadcast names
    graph.auto_connect()
    
    # Set inputs
    executor = gs.Executor()
    graph.get_node("add_numbers").set_input("a", gs.PortData(5))
    graph.get_node("add_numbers").set_input("b", gs.PortData(3))
    
    # Execute
    result = executor.execute(graph)
    
    if result.is_success():
        final_result = result.get_output("multiply", "result")
        print(f"Result: 5 + 3 = 8, then 8 * 10 = {final_result}")
        print("✓ Simple port demo successful!")


def demo_port_objects():
    """
    Demo showing how to use Port objects directly for maximum control.
    """
    print("\n=== Port Objects Demo ===\n")
    
    def legacy_function(old_param_name):
        """A legacy function with an inconvenient parameter name."""
        return {"legacy_output": old_param_name.upper()}
    
    graph = gs.Graph()
    
    # Use Port objects for fine-grained control
    input_port = gs.Port(
        "modern_input",  # broadcast_name - used for connections
        "old_param_name",  # impl_name - actual parameter name
        "Modern Input"  # display_name - shown in visualizations
    )
    input_port.with_description("This port connects modern data to a legacy function")
    
    output_port = gs.Port(
        "modern_output",  # broadcast_name
        "legacy_output",  # impl_name
        "Modern Output"  # display_name
    )
    
    graph.add(
        legacy_function,
        label="Legacy Function",
        inputs=[input_port],
        outputs=[output_port]
    )
    
    print("✓ Port objects demo created")
    print("  This shows how you can wrap legacy functions with modern interfaces")


if __name__ == "__main__":
    main()
    demo_simple_ports()
    demo_port_objects()
