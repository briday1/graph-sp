#!/usr/bin/env python3
"""
Demonstration of how Arc wrapping in Rust affects Python users.

Key insight: Python objects are ALREADY stored as PyObject pointers,
so Arc wrapping in Rust is completely transparent to Python users!
"""

import sys
sys.path.insert(0, '../target/wheels')

try:
    from dagex import Graph
except ImportError:
    print("ERROR: Could not import dagex. Build with: maturin develop --release")
    sys.exit(1)

def create_data(inputs):
    """Create a large Python list"""
    print("Creating large Python list...")
    data = list(range(1_000_000))
    print(f"  Python list id: {id(data)}")
    return {'data': data}

def node1(inputs):
    """First node accessing the data"""
    data = inputs.get('data')
    print(f"\nNode1:")
    print(f"  Received data id: {id(data)}")
    print(f"  Type: {type(data)}")
    print(f"  Length: {len(data)}")
    return {'result1': sum(data[:100])}

def node2(inputs):
    """Second node accessing the data"""
    data = inputs.get('data')
    print(f"\nNode2:")
    print(f"  Received data id: {id(data)}")
    print(f"  Type: {type(data)}")
    print(f"  Length: {len(data)}")
    return {'result2': sum(data[100:200])}

def node3(inputs):
    """Third node accessing the data"""
    data = inputs.get('data')
    print(f"\nNode3:")
    print(f"  Received data id: {id(data)}")
    print(f"  Type: {type(data)}")
    print(f"  Length: {len(data)}")
    return {'result3': sum(data[200:300])}

def main():
    print("=== Python Arc Wrapping Demo ===\n")
    print("This shows how Arc wrapping in Rust affects Python users.\n")
    
    graph = Graph()
    
    graph.add(
        create_data,
        label="Source",
        inputs=None,
        outputs=[('data', 'data')]
    )
    
    graph.add(
        node1,
        label="Node1",
        inputs=[('data', 'data')],
        outputs=[('result1', 'result1')]
    )
    
    graph.add(
        node2,
        label="Node2",
        inputs=[('data', 'data')],
        outputs=[('result2', 'result2')]
    )
    
    graph.add(
        node3,
        label="Node3",
        inputs=[('data', 'data')],
        outputs=[('result3', 'result3')]
    )
    
    dag = graph.build()
    result = dag.execute(parallel=False)
    
    print("\n=== Key Insights ===")
    print("\n1. Python Object Storage:")
    print("   - Python objects are stored in Rust as 'PyObject' (already a pointer)")
    print("   - They're NOT converted to Rust Vec/Array at the boundary")
    print("   - Rust stores: GraphData::PyObject(ptr)")
    
    print("\n2. What Happens When Rust Uses Arc:")
    print("   Current: GraphData::FloatVec(Vec<f64>)")
    print("            ↓ .clone() copies entire Vec")
    print("   With Arc: GraphData::FloatVec(Arc<Vec<f64>>)")
    print("            ↓ .clone() copies only pointer")
    
    print("\n3. Impact on Python Users:")
    print("   ✓ NO API changes - same function signatures")
    print("   ✓ NO behavior changes - data still flows the same way")
    print("   ✓ Transparent performance improvement for Rust nodes")
    print("   ✓ Python nodes already benefit from PyObject reference counting")
    
    print("\n4. When It Matters:")
    print("   - Rust nodes creating large Vec/Array data")
    print("   - Multiple Rust nodes consuming that data")
    print("   - Python→Python: already efficient (PyObject refs)")
    print("   - Rust→Rust: Arc makes it efficient too!")
    
    print("\n5. Python Object IDs:")
    if id(result.get('data')) == id(result.get('data')):
        print("   Note: IDs might differ because Python creates new wrapper")
        print("   objects on each access, but underlying data is shared!")

if __name__ == '__main__':
    main()
