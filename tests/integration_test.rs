//! Integration tests for graph-sp

use graph_sp::core::{Edge, Graph, Node, NodeConfig, Port, PortData};
use graph_sp::executor::Executor;
use graph_sp::inspector::Inspector;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_full_pipeline() {
    // Create a comprehensive graph that tests multiple features
    let mut graph = Graph::new();

    // Node 1: Source that outputs multiple values
    let source = NodeConfig::new(
        "source",
        "Data Source",
        vec![],
        vec![
            Port::new("value1", "Value 1"),
            Port::new("value2", "Value 2"),
        ],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("value1".to_string(), PortData::Int(10));
            outputs.insert("value2".to_string(), PortData::Int(20));
            Ok(outputs)
        }),
    );

    // Node 2: Adds two inputs
    let adder = NodeConfig::new(
        "adder",
        "Adder",
        vec![Port::new("a", "A"), Port::new("b", "B")],
        vec![Port::new("sum", "Sum")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let (Some(PortData::Int(a)), Some(PortData::Int(b))) =
                (inputs.get("a"), inputs.get("b"))
            {
                outputs.insert("sum".to_string(), PortData::Int(a + b));
            }
            Ok(outputs)
        }),
    );

    // Node 3: Multiplies by 2
    let multiplier = NodeConfig::new(
        "multiplier",
        "Multiplier",
        vec![Port::new("input", "Input")],
        vec![Port::new("output", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("input") {
                outputs.insert("output".to_string(), PortData::Int(val * 2));
            }
            Ok(outputs)
        }),
    );

    // Add nodes
    graph.add_node(Node::new(source)).unwrap();
    graph.add_node(Node::new(adder)).unwrap();
    graph.add_node(Node::new(multiplier)).unwrap();

    // Connect nodes
    graph
        .add_edge(Edge::new("source", "value1", "adder", "a"))
        .unwrap();
    graph
        .add_edge(Edge::new("source", "value2", "adder", "b"))
        .unwrap();
    graph
        .add_edge(Edge::new("adder", "sum", "multiplier", "input"))
        .unwrap();

    // Validate
    assert!(graph.validate().is_ok());

    // Analyze
    let analysis = Inspector::analyze(&graph);
    assert_eq!(analysis.node_count, 3);
    assert_eq!(analysis.edge_count, 3);
    assert_eq!(analysis.depth, 3);
    assert_eq!(analysis.source_nodes.len(), 1);
    assert_eq!(analysis.sink_nodes.len(), 1);

    // Execute
    let executor = Executor::new();
    let result = executor.execute(&mut graph).await.unwrap();

    // Verify results
    assert!(result.is_success());

    // Source outputs 10 and 20
    assert_eq!(
        result.get_output("source", "value1"),
        Some(&PortData::Int(10))
    );
    assert_eq!(
        result.get_output("source", "value2"),
        Some(&PortData::Int(20))
    );

    // Adder outputs 30 (10 + 20)
    assert_eq!(result.get_output("adder", "sum"), Some(&PortData::Int(30)));

    // Multiplier outputs 60 (30 * 2)
    assert_eq!(
        result.get_output("multiplier", "output"),
        Some(&PortData::Int(60))
    );
}

#[tokio::test]
async fn test_graph_with_optional_ports() {
    let mut graph = Graph::new();

    // Node with optional port
    let config = NodeConfig::new(
        "optional_node",
        "Optional Node",
        vec![
            Port::new("required", "Required Input"),
            Port::optional("optional", "Optional Input"),
        ],
        vec![Port::new("output", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let Some(PortData::Int(req)) = inputs.get("required") {
                let opt = inputs
                    .get("optional")
                    .and_then(|d| match d {
                        PortData::Int(v) => Some(*v),
                        _ => None,
                    })
                    .unwrap_or(0);
                outputs.insert("output".to_string(), PortData::Int(req + opt));
            }
            Ok(outputs)
        }),
    );

    let mut node = Node::new(config);
    node.set_input("required", PortData::Int(42));
    // Note: not setting optional input

    graph.add_node(node).unwrap();

    let executor = Executor::new();
    let result = executor.execute(&mut graph).await.unwrap();

    // Should succeed even without optional input
    // Test that the node executes successfully even when the optional input port is not provided
    assert!(result.is_success());
    assert_eq!(
        result.get_output("optional_node", "output"),
        Some(&PortData::Int(42))
    );
}

#[tokio::test]
async fn test_complex_data_types() {
    let mut graph = Graph::new();

    // Node that works with different data types
    let config = NodeConfig::new(
        "data_processor",
        "Data Processor",
        vec![],
        vec![
            Port::new("string", "String"),
            Port::new("float", "Float"),
            Port::new("bool", "Bool"),
            Port::new("list", "List"),
        ],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("string".to_string(), PortData::String("Hello".to_string()));
            outputs.insert("float".to_string(), PortData::Float(3.14));
            outputs.insert("bool".to_string(), PortData::Bool(true));
            outputs.insert(
                "list".to_string(),
                PortData::List(vec![PortData::Int(1), PortData::Int(2), PortData::Int(3)]),
            );
            Ok(outputs)
        }),
    );

    graph.add_node(Node::new(config)).unwrap();

    let executor = Executor::new();
    let result = executor.execute(&mut graph).await.unwrap();

    assert!(result.is_success());
    assert_eq!(
        result.get_output("data_processor", "string"),
        Some(&PortData::String("Hello".to_string()))
    );
    assert_eq!(
        result.get_output("data_processor", "float"),
        Some(&PortData::Float(3.14))
    );
    assert_eq!(
        result.get_output("data_processor", "bool"),
        Some(&PortData::Bool(true))
    );
}

#[test]
fn test_graph_validation_rejects_cycles() {
    let mut graph = Graph::new();

    // Create a cycle: A -> B -> C -> A
    for id in ["A", "B", "C"] {
        let config = NodeConfig::new(
            id,
            id,
            vec![Port::new("in", "Input")],
            vec![Port::new("out", "Output")],
            Arc::new(|inputs: &HashMap<String, PortData>| Ok(inputs.clone())),
        );
        graph.add_node(Node::new(config)).unwrap();
    }

    graph.add_edge(Edge::new("A", "out", "B", "in")).unwrap();
    graph.add_edge(Edge::new("B", "out", "C", "in")).unwrap();
    graph.add_edge(Edge::new("C", "out", "A", "in")).unwrap();

    // Should detect cycle
    assert!(graph.validate().is_err());
}

#[test]
fn test_inspector_visualization() {
    let mut graph = Graph::new();

    let config = NodeConfig::new(
        "test_node",
        "Test Node",
        vec![Port::new("input", "Input Port")],
        vec![Port::new("output", "Output Port")],
        Arc::new(|_inputs: &HashMap<String, PortData>| Ok(HashMap::new())),
    );

    graph.add_node(Node::new(config)).unwrap();

    let visualization = Inspector::visualize(&graph).unwrap();

    assert!(visualization.contains("Test Node"));
    assert!(visualization.contains("test_node"));
    assert!(visualization.contains("Input Port"));
    assert!(visualization.contains("Output Port"));
}
