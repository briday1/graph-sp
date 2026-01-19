//! GraphData container for passing typed data between nodes
//!
//! This module provides a generic container that can hold various data types
//! (numbers, arrays, complex arrays, strings, etc.) and be passed through graph nodes.

use std::collections::HashMap;

#[cfg(feature = "radar_examples")]
use ndarray::Array1;
#[cfg(feature = "radar_examples")]
use num_complex::Complex;

#[cfg(feature = "python")]
use pyo3::PyObject;

/// GraphData enum supporting multiple data types
#[derive(Clone, Debug)]
pub enum GraphData {
    /// 64-bit integer
    Int(i64),
    /// 64-bit floating point
    Float(f64),
    /// UTF-8 string
    String(String),
    /// Vector of floats
    FloatVec(Vec<f64>),
    /// Vector of integers
    IntVec(Vec<i64>),
    /// Complex number
    #[cfg(feature = "radar_examples")]
    Complex(Complex<f64>),
    /// 1D array of floats (ndarray)
    #[cfg(feature = "radar_examples")]
    FloatArray(Array1<f64>),
    /// 1D array of complex numbers (ndarray)
    #[cfg(feature = "radar_examples")]
    ComplexArray(Array1<Complex<f64>>),
    /// Nested map of GraphData (for structured data)
    Map(HashMap<String, GraphData>),
    /// Python object (opaque, no conversion)
    #[cfg(feature = "python")]
    PyObject(PyObject),
    /// Empty/null value
    None,
}

impl GraphData {
    /// Create an Int variant
    pub fn int(value: i64) -> Self {
        GraphData::Int(value)
    }

    /// Create a Float variant
    pub fn float(value: f64) -> Self {
        GraphData::Float(value)
    }

    /// Create a String variant
    pub fn string(value: impl Into<String>) -> Self {
        GraphData::String(value.into())
    }

    /// Create a FloatVec variant
    pub fn float_vec(value: Vec<f64>) -> Self {
        GraphData::FloatVec(value)
    }

    /// Create an IntVec variant
    pub fn int_vec(value: Vec<i64>) -> Self {
        GraphData::IntVec(value)
    }

    /// Create a Map variant
    pub fn map(value: HashMap<String, GraphData>) -> Self {
        GraphData::Map(value)
    }

    /// Create a None variant
    pub fn none() -> Self {
        GraphData::None
    }

    #[cfg(feature = "radar_examples")]
    /// Create a Complex variant
    pub fn complex(value: Complex<f64>) -> Self {
        GraphData::Complex(value)
    }

    #[cfg(feature = "radar_examples")]
    /// Create a FloatArray variant
    pub fn float_array(value: Array1<f64>) -> Self {
        GraphData::FloatArray(value)
    }

    #[cfg(feature = "radar_examples")]
    /// Create a ComplexArray variant
    pub fn complex_array(value: Array1<Complex<f64>>) -> Self {
        GraphData::ComplexArray(value)
    }

    #[cfg(feature = "python")]
    /// Create a PyObject variant (stores Python object without conversion)
    pub fn py_object(value: PyObject) -> Self {
        GraphData::PyObject(value)
    }

    /// Try to extract as i64
    pub fn as_int(&self) -> Option<i64> {
        match self {
            GraphData::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract as f64
    pub fn as_float(&self) -> Option<f64> {
        match self {
            GraphData::Float(v) => Some(*v),
            GraphData::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Try to extract as String reference
    pub fn as_string(&self) -> Option<&str> {
        match self {
            GraphData::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Try to extract as Vec<f64> reference
    pub fn as_float_vec(&self) -> Option<&Vec<f64>> {
        match self {
            GraphData::FloatVec(v) => Some(v),
            _ => None,
        }
    }

    /// Try to extract as Vec<i64> reference
    pub fn as_int_vec(&self) -> Option<&Vec<i64>> {
        match self {
            GraphData::IntVec(v) => Some(v),
            _ => None,
        }
    }

    /// Try to extract as HashMap reference
    pub fn as_map(&self) -> Option<&HashMap<String, GraphData>> {
        match self {
            GraphData::Map(m) => Some(m),
            _ => None,
        }
    }

    #[cfg(feature = "radar_examples")]
    /// Try to extract as Complex<f64>
    pub fn as_complex(&self) -> Option<Complex<f64>> {
        match self {
            GraphData::Complex(c) => Some(*c),
            _ => None,
        }
    }

    #[cfg(feature = "radar_examples")]
    /// Try to extract as Array1<f64> reference
    pub fn as_float_array(&self) -> Option<&Array1<f64>> {
        match self {
            GraphData::FloatArray(a) => Some(a),
            _ => None,
        }
    }

    #[cfg(feature = "radar_examples")]
    /// Try to extract as Array1<Complex<f64>> reference
    pub fn as_complex_array(&self) -> Option<&Array1<Complex<f64>>> {
        match self {
            GraphData::ComplexArray(a) => Some(a),
            _ => None,
        }
    }

    #[cfg(feature = "python")]
    /// Try to extract as PyObject reference
    pub fn as_py_object(&self) -> Option<&PyObject> {
        match self {
            GraphData::PyObject(obj) => Some(obj),
            _ => None,
        }
    }

    /// Check if this is None
    pub fn is_none(&self) -> bool {
        matches!(self, GraphData::None)
    }

    /// Convert GraphData to a string representation (for compatibility)
    pub fn to_string_repr(&self) -> String {
        match self {
            GraphData::Int(v) => v.to_string(),
            GraphData::Float(v) => v.to_string(),
            GraphData::String(s) => s.clone(),
            GraphData::FloatVec(v) => format!("{:?}", v),
            GraphData::IntVec(v) => format!("{:?}", v),
            #[cfg(feature = "radar_examples")]
            GraphData::Complex(c) => format!("{:?}", c),
            #[cfg(feature = "radar_examples")]
            GraphData::FloatArray(a) => format!("{:?}", a),
            #[cfg(feature = "radar_examples")]
            GraphData::ComplexArray(a) => format!("{:?}", a),
            GraphData::Map(m) => format!("{:?}", m),
            #[cfg(feature = "python")]
            GraphData::PyObject(_) => "<PyObject>".to_string(),
            GraphData::None => "None".to_string(),
        }
    }

    /// Try to parse GraphData from a string
    pub fn from_string(s: &str) -> Self {
        // Try to parse as i64
        if let Ok(v) = s.parse::<i64>() {
            return GraphData::Int(v);
        }
        // Try to parse as f64
        if let Ok(v) = s.parse::<f64>() {
            return GraphData::Float(v);
        }
        // Otherwise, store as string
        GraphData::String(s.to_string())
    }
}

impl Default for GraphData {
    fn default() -> Self {
        GraphData::None
    }
}

impl From<i64> for GraphData {
    fn from(v: i64) -> Self {
        GraphData::Int(v)
    }
}

impl From<f64> for GraphData {
    fn from(v: f64) -> Self {
        GraphData::Float(v)
    }
}

impl From<String> for GraphData {
    fn from(v: String) -> Self {
        GraphData::String(v)
    }
}

impl From<&str> for GraphData {
    fn from(v: &str) -> Self {
        GraphData::String(v.to_string())
    }
}

impl From<Vec<f64>> for GraphData {
    fn from(v: Vec<f64>) -> Self {
        GraphData::FloatVec(v)
    }
}

impl From<Vec<i64>> for GraphData {
    fn from(v: Vec<i64>) -> Self {
        GraphData::IntVec(v)
    }
}

#[cfg(feature = "radar_examples")]
impl From<Complex<f64>> for GraphData {
    fn from(v: Complex<f64>) -> Self {
        GraphData::Complex(v)
    }
}

#[cfg(feature = "radar_examples")]
impl From<Array1<f64>> for GraphData {
    fn from(v: Array1<f64>) -> Self {
        GraphData::FloatArray(v)
    }
}

#[cfg(feature = "radar_examples")]
impl From<Array1<Complex<f64>>> for GraphData {
    fn from(v: Array1<Complex<f64>>) -> Self {
        GraphData::ComplexArray(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_construction() {
        let data = GraphData::int(42);
        assert_eq!(data.as_int(), Some(42));
        assert_eq!(data.as_float(), Some(42.0));
    }

    #[test]
    fn test_float_construction() {
        let data = GraphData::float(3.14);
        assert_eq!(data.as_float(), Some(3.14));
        assert!(data.as_int().is_none());
    }

    #[test]
    fn test_string_construction() {
        let data = GraphData::string("hello");
        assert_eq!(data.as_string(), Some("hello"));
    }

    #[test]
    fn test_float_vec_construction() {
        let data = GraphData::float_vec(vec![1.0, 2.0, 3.0]);
        assert_eq!(data.as_float_vec(), Some(&vec![1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_int_vec_construction() {
        let data = GraphData::int_vec(vec![1, 2, 3]);
        assert_eq!(data.as_int_vec(), Some(&vec![1, 2, 3]));
    }

    #[test]
    fn test_map_construction() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), GraphData::int(10));
        map.insert("y".to_string(), GraphData::float(20.5));
        let data = GraphData::map(map);

        let extracted = data.as_map().unwrap();
        assert_eq!(extracted.get("x").and_then(|d| d.as_int()), Some(10));
        assert_eq!(extracted.get("y").and_then(|d| d.as_float()), Some(20.5));
    }

    #[test]
    fn test_none_construction() {
        let data = GraphData::none();
        assert!(data.is_none());
    }

    #[test]
    fn test_from_conversions() {
        let d1: GraphData = 42i64.into();
        assert_eq!(d1.as_int(), Some(42));

        let d2: GraphData = 3.14f64.into();
        assert_eq!(d2.as_float(), Some(3.14));

        let d3: GraphData = "test".into();
        assert_eq!(d3.as_string(), Some("test"));

        let d4: GraphData = vec![1.0, 2.0].into();
        assert_eq!(d4.as_float_vec(), Some(&vec![1.0, 2.0]));
    }

    #[test]
    fn test_to_string_repr() {
        assert_eq!(GraphData::int(42).to_string_repr(), "42");
        assert_eq!(GraphData::float(3.14).to_string_repr(), "3.14");
        assert_eq!(GraphData::string("test").to_string_repr(), "test");
        assert!(GraphData::none().to_string_repr().contains("None"));
    }

    #[test]
    fn test_from_string() {
        let d1 = GraphData::from_string("42");
        assert_eq!(d1.as_int(), Some(42));

        let d2 = GraphData::from_string("3.14");
        assert_eq!(d2.as_float(), Some(3.14));

        let d3 = GraphData::from_string("not a number");
        assert_eq!(d3.as_string(), Some("not a number"));
    }
}
