# Port Data Types and Restrictions

## Overview

The graph-sp execution engine uses a strongly-typed enum `PortData` to pass data between nodes. This document describes the available types, their restrictions, and best practices.

## Available Data Types

### 1. Primitives

```rust
PortData::None       // Unit type (no data)
PortData::Bool(bool) // Boolean values
PortData::Int(i64)   // 64-bit signed integers
PortData::Float(f64) // 64-bit floating point
PortData::String(String) // UTF-8 strings
```

### 2. Collections

```rust
PortData::List(Vec<PortData>)           // Ordered list of any PortData
PortData::Map(HashMap<String, PortData>) // Key-value pairs with String keys
```

### 3. Special Types

```rust
PortData::Bytes(Vec<u8>)        // Binary data (images, files, etc.)
PortData::Json(serde_json::Value) // Arbitrary JSON structures
```

## Passing Objects with Different Attributes

You can pass objects with different attributes using three approaches:

### Approach 1: Using PortData::Map (Recommended for Rust)

Best for: Type-safe, structured data with known fields

```rust
// Create an object
let mut person = HashMap::new();
person.insert("name".to_string(), PortData::String("Alice".to_string()));
person.insert("age".to_string(), PortData::Int(30));
person.insert("email".to_string(), PortData::String("alice@example.com".to_string()));

// Nested objects
let mut address = HashMap::new();
address.insert("street".to_string(), PortData::String("123 Main St".to_string()));
address.insert("city".to_string(), PortData::String("New York".to_string()));
person.insert("address".to_string(), PortData::Map(address));

// Pass it through a port
outputs.insert("person".to_string(), PortData::Map(person));
```

**Restrictions:**
- Keys must be Strings
- Values must be one of the PortData variants
- No circular references (use Arc/Rc if needed)
- All data is cloned when passing between nodes

### Approach 2: Using PortData::Json (Recommended for Dynamic Data)

Best for: Arbitrary structures, especially when interfacing with JSON APIs

```rust
use serde_json::json;

// Create complex JSON
let product = json!({
    "id": 12345,
    "name": "Laptop",
    "price": 999.99,
    "specs": {
        "cpu": "Intel i7",
        "ram": "16GB"
    },
    "tags": ["electronics", "portable"]
});

outputs.insert("product".to_string(), PortData::Json(product));
```

**Restrictions:**
- Must be valid JSON (serializable via serde_json)
- Supports nested structures of any depth
- Can represent null, bool, number, string, array, object
- Performance overhead due to JSON parsing

### Approach 3: Using Custom Serialization with PortData::Bytes

Best for: Complex Rust types, custom binary protocols

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CustomObject {
    field1: String,
    field2: i32,
    // ... any serializable fields
}

// Serialize to bytes
let obj = CustomObject { /* ... */ };
let bytes = bincode::serialize(&obj)?;
outputs.insert("object".to_string(), PortData::Bytes(bytes));

// Deserialize on receiving end
if let Some(PortData::Bytes(bytes)) = inputs.get("object") {
    let obj: CustomObject = bincode::deserialize(bytes)?;
}
```

**Restrictions:**
- Requires serde-compatible types
- Both sender and receiver must know the type structure
- Binary format is not human-readable

## Nesting and Composition

All types can be nested arbitrarily:

```rust
// List of Maps
PortData::List(vec![
    PortData::Map(user1),
    PortData::Map(user2),
])

// Map containing Lists and other Maps
let mut data = HashMap::new();
data.insert("users".to_string(), PortData::List(users));
data.insert("metadata".to_string(), PortData::Map(metadata));
data.insert("timestamp".to_string(), PortData::Int(1234567890));
```

## Key Restrictions

### 1. **Type Safety**
- Port data is strongly typed at runtime
- Pattern matching is required to extract values
- Type mismatches will cause execution errors

```rust
// ✓ Correct
if let Some(PortData::Int(value)) = inputs.get("number") {
    // Use value
}

// ✗ Wrong - will panic
let value = inputs.get("number").unwrap();
// Need to match the variant first!
```

### 2. **Cloning**
- All data is cloned when passed between nodes
- For large data, consider using:
  - PortData::Bytes with shared references
  - External storage with IDs passed through ports

### 3. **No Circular References**
- PortData structures cannot contain circular references
- All data must be tree-structured

### 4. **Serialization**
- All PortData variants implement `Serialize` and `Deserialize`
- Can be persisted or transmitted over network
- JSON format is used for human-readable serialization

### 5. **Map Keys**
- Map keys must be Strings
- For other key types, use JSON or custom serialization

## Best Practices

### 1. Choose the Right Type

- **Primitives**: For simple values (counters, flags, IDs)
- **Map**: For structured objects with known schema
- **Json**: For dynamic/arbitrary structures or JSON APIs
- **List**: For collections of homogeneous or heterogeneous items
- **Bytes**: For binary data or custom serialization

### 2. Document Port Schemas

```rust
// Good: Document expected structure
/// Input port "user" expects:
/// {
///   "name": String,
///   "age": Int,
///   "email": String (optional)
/// }
Port::new("user", "User Object")
```

### 3. Handle Optional Fields

```rust
// Use unwrap_or for defaults
let age = match data.get("age") {
    Some(PortData::Int(n)) => *n,
    _ => 0, // default value
};
```

### 4. Validate Input Data

```rust
// Validate structure before processing
fn validate_user(user: &HashMap<String, PortData>) -> Result<()> {
    if !user.contains_key("name") {
        return Err(GraphError::PortError("Missing 'name' field".into()));
    }
    // ... more validations
    Ok(())
}
```

### 5. Consider Performance

- **Small objects**: Use Map (type-safe, efficient)
- **Large objects**: Use Bytes with custom serialization
- **External data**: Pass IDs/references, store data externally

## Python Bindings

When using Python bindings (with `--features python`):

```python
import graph_sp

# Python types automatically convert:
# - None → PortData::None
# - bool → PortData::Bool
# - int → PortData::Int
# - float → PortData::Float
# - str → PortData::String
# - list → PortData::List
# - dict → PortData::Map

def process_user(inputs):
    user = inputs["user"]  # Python dict
    return {
        "greeting": f"Hello, {user['name']}!"
    }
```

## Examples

See these example files for complete demonstrations:

- `examples/simple_pipeline.rs` - Basic data flow
- `examples/complex_objects.rs` - Complex object passing
- Tests in `tests/integration_test.rs` - All data type usage

## Summary

✅ **Can Pass:**
- Any combination of primitives, collections, JSON, and bytes
- Nested structures of arbitrary depth
- Objects with different attributes using Map or Json
- Binary data and custom serialized types

❌ **Cannot Pass:**
- Circular references
- Non-String map keys (use Json instead)
- Non-serializable types (use Bytes with custom serialization)
- Direct Rust trait objects (serialize them first)

The system is designed to be flexible while maintaining type safety and performance!
