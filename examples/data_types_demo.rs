//! Comprehensive example demonstrating all GraphData types in Rust.
//!
//! This example shows how graph-sp can handle multiple data types:
//! - Integers and floats
//! - Strings
//! - Vectors of numbers
//! - Nested structures (Maps)
//! - Custom structs (via Map serialization)
//!
//! Key insight: GraphData is just a transport container. Your node functions
//! can process any data type - GraphData doesn't restrict what you can pass.

use graph_sp::{Graph, GraphData};
use std::collections::HashMap;

fn data_generator(
    _inputs: &HashMap<String, GraphData>,
    _params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    println!("{}", "=".repeat(70));
    println!("DataGenerator: Creating diverse data types");
    println!("{}", "=".repeat(70));

    let mut outputs = HashMap::new();

    // Basic types
    outputs.insert("integer".to_string(), GraphData::int(42));
    outputs.insert("float".to_string(), GraphData::float(3.14159));
    outputs.insert("string".to_string(), GraphData::string("Hello, GraphData!"));

    // Collections
    outputs.insert(
        "int_list".to_string(),
        GraphData::int_vec(vec![1, 2, 3, 4, 5]),
    );
    outputs.insert(
        "float_list".to_string(),
        GraphData::float_vec(vec![1.1, 2.2, 3.3, 4.4, 5.5]),
    );

    // Nested structures (simulating complex objects)
    let mut metadata = HashMap::new();
    metadata.insert(
        "timestamp".to_string(),
        GraphData::string("2026-01-19T03:21:05"),
    );
    metadata.insert("version".to_string(), GraphData::string("1.0.0"));
    metadata.insert("author".to_string(), GraphData::string("graph-sp"));

    let mut config = HashMap::new();
    config.insert("mode".to_string(), GraphData::string("demo"));
    config.insert("verbose".to_string(), GraphData::int(1)); // bool as int
    config.insert("threshold".to_string(), GraphData::float(0.5));
    metadata.insert("config".to_string(), GraphData::map(config));

    outputs.insert("metadata".to_string(), GraphData::map(metadata));

    // Custom object (sensor reading)
    let mut sensor = HashMap::new();
    sensor.insert("type".to_string(), GraphData::string("sensor_reading"));
    sensor.insert("sensor_id".to_string(), GraphData::string("SENSOR_001"));
    sensor.insert(
        "readings".to_string(),
        GraphData::float_vec(vec![23.5, 23.7, 23.9, 24.1]),
    );
    sensor.insert("status".to_string(), GraphData::string("nominal"));

    let mut calibration = HashMap::new();
    calibration.insert("offset".to_string(), GraphData::float(0.1));
    calibration.insert("scale".to_string(), GraphData::float(1.02));
    sensor.insert("calibration".to_string(), GraphData::map(calibration));

    outputs.insert("custom_object".to_string(), GraphData::map(sensor));

    outputs
}

fn type_inspector(
    inputs: &HashMap<String, GraphData>,
    _params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    println!("\n{}", "=".repeat(70));
    println!("TypeInspector: Analyzing received data types");
    println!("{}", "=".repeat(70));

    for (key, value) in inputs.iter() {
        println!("\n{}:", key);
        match value {
            GraphData::Int(v) => println!("  Type: Int, Value: {}", v),
            GraphData::Float(v) => println!("  Type: Float, Value: {}", v),
            GraphData::String(v) => println!("  Type: String, Value: {}", v),
            GraphData::IntVec(v) => {
                println!("  Type: IntVec, Length: {}, Values: {:?}", v.len(), v)
            }
            GraphData::FloatVec(v) => {
                println!("  Type: FloatVec, Length: {}, Values: {:?}", v.len(), v)
            }
            GraphData::Map(m) => {
                println!("  Type: Map, Keys: {}", m.len());
                for (k, _) in m.iter() {
                    println!("    - {}", k);
                }
            }
            GraphData::None => println!("  Type: None"),
            #[cfg(feature = "radar_examples")]
            _ => println!("  Type: Other (radar type)"),
        }
    }

    // Pass everything through unchanged
    inputs.clone()
}

fn data_processor(
    inputs: &HashMap<String, GraphData>,
    _params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    println!("\n{}", "=".repeat(70));
    println!("DataProcessor: Processing multiple data types");
    println!("{}", "=".repeat(70));

    let mut results = HashMap::new();

    // Process integer
    if let Some(GraphData::Int(val)) = inputs.get("integer") {
        results.insert("integer_doubled".to_string(), GraphData::int(val * 2));
        println!("Integer: {} → doubled → {}", val, val * 2);
    }

    // Process float
    if let Some(GraphData::Float(val)) = inputs.get("float") {
        results.insert("float_squared".to_string(), GraphData::float(val * val));
        println!("Float: {:.5} → squared → {:.5}", val, val * val);
    }

    // Process string
    if let Some(GraphData::String(val)) = inputs.get("string") {
        results.insert(
            "string_upper".to_string(),
            GraphData::string(val.to_uppercase()),
        );
        println!("String: '{}' → upper → '{}'", val, val.to_uppercase());
    }

    // Process list
    if let Some(GraphData::FloatVec(list)) = inputs.get("float_list") {
        let sum: f64 = list.iter().sum();
        let avg = sum / list.len() as f64;
        results.insert("list_sum".to_string(), GraphData::float(sum));
        results.insert("list_avg".to_string(), GraphData::float(avg));
        println!("Float List: {:?}", list);
        println!("  Sum: {:.2}, Average: {:.2}", sum, avg);
    }

    // Process nested structure
    if let Some(GraphData::Map(meta)) = inputs.get("metadata") {
        if let Some(GraphData::String(version)) = meta.get("version") {
            if let Some(GraphData::String(author)) = meta.get("author") {
                let summary = format!("Version {} by {}", version, author);
                results.insert(
                    "meta_summary".to_string(),
                    GraphData::string(summary.clone()),
                );
                println!("Metadata: {}", summary);
            }
        }
    }

    // Process custom object
    if let Some(GraphData::Map(obj)) = inputs.get("custom_object") {
        if let Some(GraphData::String(sensor_id)) = obj.get("sensor_id") {
            if let Some(GraphData::FloatVec(readings)) = obj.get("readings") {
                let avg = readings.iter().sum::<f64>() / readings.len() as f64;
                results.insert("sensor_avg".to_string(), GraphData::float(avg));
                println!("Sensor {}: Average reading = {:.2}", sensor_id, avg);
            }
        }
    }

    results
}

fn result_aggregator(
    inputs: &HashMap<String, GraphData>,
    _params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    println!("\n{}", "=".repeat(70));
    println!("ResultAggregator: Creating final summary");
    println!("{}", "=".repeat(70));

    // Count different types of results
    let mut numeric = 0;
    let mut string = 0;
    let mut other = 0;

    for (_, value) in inputs.iter() {
        match value {
            GraphData::Int(_) | GraphData::Float(_) => numeric += 1,
            GraphData::String(_) => string += 1,
            _ => other += 1,
        }
    }

    println!("\nSummary:");
    println!("  Total outputs: {}", inputs.len());
    println!("  Numeric results: {}", numeric);
    println!("  String results: {}", string);
    println!("  Other results: {}", other);
    println!(
        "  Keys: {}",
        inputs
            .keys()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut summary = HashMap::new();
    summary.insert("total".to_string(), GraphData::int(inputs.len() as i64));
    summary.insert("numeric".to_string(), GraphData::int(numeric));
    summary.insert("string".to_string(), GraphData::int(string));
    summary.insert("other".to_string(), GraphData::int(other));

    let mut outputs = HashMap::new();
    outputs.insert("summary".to_string(), GraphData::map(summary));
    outputs.insert("success".to_string(), GraphData::int(1)); // true as 1

    outputs
}

fn main() {
    println!("\n{}", "=".repeat(70));
    println!("GraphData Multiple Data Types Demo (Rust)");
    println!("{}", "=".repeat(70));
    println!("\nThis demo shows that GraphData supports ANY data type you want!");
    println!("The graph executor doesn't care about the data - it just passes it.");
    println!("Your node functions can work with any Rust types.");
    println!("{}", "=".repeat(70));

    // Build the graph
    let mut graph = Graph::new();

    // Node 1: Generate diverse data types
    graph.add(
        data_generator,
        Some("DataGenerator"),
        None,
        Some(vec![
            ("integer", "int_val"),
            ("float", "float_val"),
            ("string", "str_val"),
            ("int_list", "int_list"),
            ("float_list", "float_list"),
            ("metadata", "meta"),
            ("custom_object", "custom"),
        ]),
    );

    // Node 2: Inspect types
    graph.add(
        type_inspector,
        Some("TypeInspector"),
        Some(vec![
            ("int_val", "integer"),
            ("float_val", "float"),
            ("str_val", "string"),
            ("int_list", "int_list"),
            ("float_list", "float_list"),
            ("meta", "metadata"),
            ("custom", "custom_object"),
        ]),
        Some(vec![
            ("integer", "inspected_int"),
            ("float", "inspected_float"),
            ("string", "inspected_str"),
            ("float_list", "inspected_list"),
            ("metadata", "inspected_meta"),
            ("custom_object", "inspected_custom"),
        ]),
    );

    // Node 3: Process the data
    graph.add(
        data_processor,
        Some("DataProcessor"),
        Some(vec![
            ("inspected_int", "integer"),
            ("inspected_float", "float"),
            ("inspected_str", "string"),
            ("inspected_list", "float_list"),
            ("inspected_meta", "metadata"),
            ("inspected_custom", "custom_object"),
        ]),
        Some(vec![
            ("integer_doubled", "result_int"),
            ("float_squared", "result_float"),
            ("string_upper", "result_str"),
            ("list_sum", "result_sum"),
            ("list_avg", "result_avg"),
            ("meta_summary", "result_meta"),
            ("sensor_avg", "result_sensor"),
        ]),
    );

    // Node 4: Aggregate results
    graph.add(
        result_aggregator,
        Some("ResultAggregator"),
        Some(vec![
            ("result_int", "integer_doubled"),
            ("result_float", "float_squared"),
            ("result_str", "string_upper"),
            ("result_sum", "list_sum"),
            ("result_avg", "list_avg"),
            ("result_meta", "meta_summary"),
            ("result_sensor", "sensor_avg"),
        ]),
        Some(vec![("summary", "final_summary"), ("success", "status")]),
    );

    // Build and execute
    println!("\nBuilding DAG...");
    let dag = graph.build();

    println!("\n{}", "=".repeat(70));
    println!("Mermaid Diagram");
    println!("{}", "=".repeat(70));
    println!("{}", dag.to_mermaid());

    println!("\n{}", "=".repeat(70));
    println!("Executing Graph");
    println!("{}", "=".repeat(70));
    let result = dag.execute(false, None);

    println!("\n{}", "=".repeat(70));
    println!("Final Results");
    println!("{}", "=".repeat(70));
    if let Some(GraphData::Map(summary)) = result.get("final_summary") {
        println!("\nExecution successful! Summary:");
        for (k, v) in summary.iter() {
            println!("  {}: {}", k, v.to_string_repr());
        }
    }
    if let Some(status) = result.get("status") {
        println!("\nStatus: {}", status.to_string_repr());
    }

    println!("\n{}", "=".repeat(70));
    println!("Key Takeaway");
    println!("{}", "=".repeat(70));
    println!(
        r#"
GraphData is a TRANSPORT container, not a type restriction!

You can pass ANY Rust type through the graph:
- Built-in types (i64, f64, String, Vec<T>)
- Nested structures (HashMap/Map)
- Custom structs (serialize to Map)
- Even arbitrary data via Map!

The graph executor doesn't care what's in the container.
Your node functions decide what to do with the data.
This gives you complete flexibility!
    "#
    );
}
