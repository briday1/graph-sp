"""
Complex objects example using graph-sp Python bindings.

This example demonstrates passing complex data structures through ports
using Maps, JSON, Lists, and binary data.
"""

import pygraph_sp as gs


def main():
    print("=== Complex Object Passing Example ===\n")
    
    # Example 1: Using Maps for structured objects
    print("Example 1: Using Maps for structured objects\n")
    
    graph1 = graph_sp.Graph()
    
    def user_creator(inputs):
        """Create a user object with nested structure"""
        user = {
            "name": "Alice",
            "age": 30,
            "email": "alice@example.com",
            "active": True,
            "score": 95.5,
            "address": {
                "street": "123 Main St",
                "city": "Springfield",
                "zip": "12345"
            },
            "hobbies": ["reading", "coding", "hiking"]
        }
        return {"user": user}
    
    def user_processor(inputs):
        """Process user object and extract information"""
        user = inputs["user"]
        name = user.get("name", "Unknown")
        age = user.get("age", 0)
        hobby_count = len(user.get("hobbies", []))
        city = user.get("address", {}).get("city", "Unknown")
        
        summary = f"{name} is {age} years old, lives in {city}, and has {hobby_count} hobbies"
        return {"summary": summary}
    
    graph1.add(
        "user_creator",
        "User Creator",
        [],
        [graph_sp.Port("user", "User Object")],
        user_creator
    )
    
    graph1.add(
        "user_processor",
        "User Processor",
        [graph_sp.Port("user", "User Object")],
        [graph_sp.Port("summary", "Summary")],
        user_processor
    )
    
    graph1.add_edge("user_creator", "user", "user_processor", "user")
    
    executor = graph_sp.Executor()
    result1 = executor.execute(graph1)
    
    summary = result1.get_output("user_processor", "summary")
    print(f"Summary: {summary}\n")
    
    # Example 2: Using JSON for arbitrary structures
    print("Example 2: Using JSON for arbitrary structures\n")
    
    graph2 = graph_sp.Graph()
    
    def json_producer(inputs):
        """Create complex JSON object"""
        data = {
            "product": {
                "id": 12345,
                "name": "Laptop",
                "price": 999.99,
                "specs": {
                    "cpu": "Intel i7",
                    "ram": "16GB",
                    "storage": "512GB SSD"
                },
                "tags": ["electronics", "computers", "portable"],
                "available": True
            }
        }
        return {"data": data}
    
    def json_consumer(inputs):
        """Extract values from JSON"""
        data = inputs["data"]
        product = data.get("product", {})
        name = product.get("name", "Unknown")
        price = product.get("price", 0.0)
        cpu = product.get("specs", {}).get("cpu", "N/A")
        available = product.get("available", False)
        
        description = f"{name} - ${price:.2f} (CPU: {cpu}, Available: {available})"
        return {"description": description}
    
    graph2.add(
        "json_producer",
        "JSON Producer",
        [],
        [graph_sp.Port("data", "JSON Data")],
        json_producer
    )
    
    graph2.add(
        "json_consumer",
        "JSON Consumer",
        [graph_sp.Port("data", "JSON Data")],
        [graph_sp.Port("description", "Description")],
        json_consumer
    )
    
    graph2.add_edge("json_producer", "data", "json_consumer", "data")
    
    result2 = executor.execute(graph2)
    
    description = result2.get_output("json_consumer", "description")
    print(f"Description: {description}\n")
    
    # Example 3: Using Lists
    print("Example 3: Using Lists for collections\n")
    
    graph3 = graph_sp.Graph()
    
    def list_creator(inputs):
        """Create a list of numbers"""
        return {"numbers": [1, 2, 3, 4, 5]}
    
    def list_processor(inputs):
        """Process list to compute statistics"""
        numbers = inputs["numbers"]
        total = sum(numbers)
        count = len(numbers)
        average = total / count if count > 0 else 0
        
        return {
            "sum": total,
            "count": count,
            "average": average
        }
    
    graph3.add(
        "list_creator",
        "List Creator",
        [],
        [graph_sp.Port("numbers", "Numbers")],
        list_creator
    )
    
    graph3.add(
        "list_processor",
        "List Processor",
        [graph_sp.Port("numbers", "Numbers")],
        [
            graph_sp.Port("sum", "Sum"),
            graph_sp.Port("count", "Count"),
            graph_sp.Port("average", "Average")
        ],
        list_processor
    )
    
    graph3.add_edge("list_creator", "numbers", "list_processor", "numbers")
    
    result3 = executor.execute(graph3)
    
    total = result3.get_output("list_processor", "sum")
    count = result3.get_output("list_processor", "count")
    average = result3.get_output("list_processor", "average")
    
    print(f"Numbers: [1, 2, 3, 4, 5]")
    print(f"Sum: {total}")
    print(f"Count: {count}")
    print(f"Average: {average:.2f}\n")
    
    print("=== Summary of Port Data Types ===")
    print("✓ Maps - For structured objects with named fields")
    print("✓ JSON - For arbitrary JSON structures")
    print("✓ Lists - For arrays/vectors of data")
    print("✓ Primitives - Int, Float, String, Bool, None")
    print("\nAll types support nesting and composition!")
    print("\n=== Example Complete ===")


if __name__ == "__main__":
    main()
