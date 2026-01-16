//! Example: Passing complex objects through ports
//!
//! This example demonstrates how to pass complex data structures through ports
//! using the Map and JSON data types.

use graph_sp::core::{Graph, Node, NodeConfig, Port, PortData, Edge};
use graph_sp::executor::Executor;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Complex Object Passing Example ===\n");

    let mut graph = Graph::new();

    // Example 1: Using Map to pass structured data
    println!("Example 1: Using PortData::Map for structured objects\n");
    
    let user_creator = NodeConfig::new(
        "user_creator",
        "User Creator",
        vec![],
        vec![Port::new("user", "User Object")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            
            // Create a user object with different attributes
            let mut user = HashMap::new();
            user.insert("name".to_string(), PortData::String("Alice".to_string()));
            user.insert("age".to_string(), PortData::Int(30));
            user.insert("email".to_string(), PortData::String("alice@example.com".to_string()));
            user.insert("active".to_string(), PortData::Bool(true));
            user.insert("score".to_string(), PortData::Float(95.5));
            
            // Nested object - address
            let mut address = HashMap::new();
            address.insert("street".to_string(), PortData::String("123 Main St".to_string()));
            address.insert("city".to_string(), PortData::String("Springfield".to_string()));
            address.insert("zip".to_string(), PortData::String("12345".to_string()));
            user.insert("address".to_string(), PortData::Map(address));
            
            // Array of hobbies
            let hobbies = vec![
                PortData::String("reading".to_string()),
                PortData::String("coding".to_string()),
                PortData::String("hiking".to_string()),
            ];
            user.insert("hobbies".to_string(), PortData::List(hobbies));
            
            outputs.insert("user".to_string(), PortData::Map(user));
            Ok(outputs)
        }),
    );

    let user_processor = NodeConfig::new(
        "user_processor",
        "User Processor",
        vec![Port::new("user", "User Object")],
        vec![Port::new("summary", "Summary")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            
            if let Some(PortData::Map(user)) = inputs.get("user") {
                // Extract fields from the user object
                let name = match user.get("name") {
                    Some(PortData::String(s)) => s.clone(),
                    _ => "Unknown".to_string(),
                };
                
                let age = match user.get("age") {
                    Some(PortData::Int(n)) => *n,
                    _ => 0,
                };
                
                let hobby_count = match user.get("hobbies") {
                    Some(PortData::List(hobbies)) => hobbies.len(),
                    _ => 0,
                };
                
                let city = match user.get("address") {
                    Some(PortData::Map(addr)) => match addr.get("city") {
                        Some(PortData::String(s)) => s.clone(),
                        _ => "Unknown".to_string(),
                    },
                    _ => "Unknown".to_string(),
                };
                
                let summary = format!(
                    "{} is {} years old, lives in {}, and has {} hobbies",
                    name, age, city, hobby_count
                );
                
                outputs.insert("summary".to_string(), PortData::String(summary));
            }
            
            Ok(outputs)
        }),
    );

    graph.add_node(Node::new(user_creator)).unwrap();
    graph.add_node(Node::new(user_processor)).unwrap();
    graph.add_edge(Edge::new("user_creator", "user", "user_processor", "user")).unwrap();

    let executor = Executor::new();
    let result = executor.execute(&mut graph).await?;

    if let Some(PortData::String(summary)) = result.get_output("user_processor", "summary") {
        println!("Summary: {}\n", summary);
    }

    // Example 2: Using JSON for arbitrary structures
    println!("Example 2: Using PortData::Json for arbitrary JSON data\n");
    
    let mut graph2 = Graph::new();

    let json_producer = NodeConfig::new(
        "json_producer",
        "JSON Producer",
        vec![],
        vec![Port::new("data", "JSON Data")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            
            // Create complex JSON object
            let data = json!({
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
                    "available": true
                }
            });
            
            outputs.insert("data".to_string(), PortData::Json(data));
            Ok(outputs)
        }),
    );

    let json_consumer = NodeConfig::new(
        "json_consumer",
        "JSON Consumer",
        vec![Port::new("data", "JSON Data")],
        vec![Port::new("description", "Description")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            
            if let Some(PortData::Json(data)) = inputs.get("data") {
                // Extract values from JSON
                let name = data["product"]["name"].as_str().unwrap_or("Unknown");
                let price = data["product"]["price"].as_f64().unwrap_or(0.0);
                let cpu = data["product"]["specs"]["cpu"].as_str().unwrap_or("N/A");
                let available = data["product"]["available"].as_bool().unwrap_or(false);
                
                let description = format!(
                    "{} - ${:.2} (CPU: {}, Available: {})",
                    name, price, cpu, available
                );
                
                outputs.insert("description".to_string(), PortData::String(description));
            }
            
            Ok(outputs)
        }),
    );

    graph2.add_node(Node::new(json_producer)).unwrap();
    graph2.add_node(Node::new(json_consumer)).unwrap();
    graph2.add_edge(Edge::new("json_producer", "data", "json_consumer", "data")).unwrap();

    let result2 = executor.execute(&mut graph2).await?;

    if let Some(PortData::String(desc)) = result2.get_output("json_consumer", "description") {
        println!("Description: {}\n", desc);
    }

    // Example 3: Passing binary data
    println!("Example 3: Using PortData::Bytes for binary data\n");
    
    let mut graph3 = Graph::new();

    let binary_producer = NodeConfig::new(
        "binary_producer",
        "Binary Producer",
        vec![],
        vec![Port::new("binary", "Binary Data")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            
            // Create some binary data (e.g., image bytes)
            let binary_data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]; // JPEG header
            
            outputs.insert("binary".to_string(), PortData::Bytes(binary_data));
            Ok(outputs)
        }),
    );

    let binary_analyzer = NodeConfig::new(
        "binary_analyzer",
        "Binary Analyzer",
        vec![Port::new("binary", "Binary Data")],
        vec![Port::new("info", "Info")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            
            if let Some(PortData::Bytes(data)) = inputs.get("binary") {
                let info = format!(
                    "Binary data: {} bytes, first byte: 0x{:02X}",
                    data.len(),
                    data.first().unwrap_or(&0)
                );
                
                outputs.insert("info".to_string(), PortData::String(info));
            }
            
            Ok(outputs)
        }),
    );

    graph3.add_node(Node::new(binary_producer)).unwrap();
    graph3.add_node(Node::new(binary_analyzer)).unwrap();
    graph3.add_edge(Edge::new("binary_producer", "binary", "binary_analyzer", "binary")).unwrap();

    let result3 = executor.execute(&mut graph3).await?;

    if let Some(PortData::String(info)) = result3.get_output("binary_analyzer", "info") {
        println!("Info: {}\n", info);
    }

    println!("=== Summary of Port Data Types ===");
    println!("✓ PortData::Map - For structured objects with named fields");
    println!("✓ PortData::Json - For arbitrary JSON structures");
    println!("✓ PortData::List - For arrays/vectors of data");
    println!("✓ PortData::Bytes - For binary data");
    println!("✓ Primitives - Int, Float, String, Bool, None");
    println!("\nAll types support nesting and composition!");

    Ok(())
}
