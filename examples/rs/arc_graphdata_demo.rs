/// Example showing a hybrid Arc approach for GraphData
/// 
/// This demonstrates wrapping only large data types (Vecs, Arrays)
/// in Arc while keeping small types (Int, Float, String) as direct values.

use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum GraphDataWithArc {
    // Small types: no Arc needed
    Int(i64),
    Float(f64),
    String(String),
    
    // Large types: wrapped in Arc for efficient cloning
    FloatVec(Arc<Vec<f64>>),
    IntVec(Arc<Vec<i64>>),
    
    // Could also do:
    // FloatVec(Arc<[f64]>),  // Arc<slice> is slightly more efficient
}

impl GraphDataWithArc {
    // Convenience constructors
    pub fn int(value: i64) -> Self {
        Self::Int(value)
    }
    
    pub fn float(value: f64) -> Self {
        Self::Float(value)
    }
    
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }
    
    pub fn float_vec(value: Vec<f64>) -> Self {
        Self::FloatVec(Arc::new(value))
    }
    
    pub fn int_vec(value: Vec<i64>) -> Self {
        Self::IntVec(Arc::new(value))
    }
    
    // Access methods return references (no clone needed!)
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }
    
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Int(v) => Some(*v as f64),
            _ => None,
        }
    }
    
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
    
    // Returns reference to the Arc'd data
    pub fn as_float_vec(&self) -> Option<&[f64]> {
        match self {
            Self::FloatVec(v) => Some(v.as_ref()),
            _ => None,
        }
    }
    
    pub fn as_int_vec(&self) -> Option<&[i64]> {
        match self {
            Self::IntVec(v) => Some(v.as_ref()),
            _ => None,
        }
    }
}

fn main() {
    println!("=== Arc GraphData Demo ===\n");
    
    // Small data - no overhead from Arc
    let small = GraphDataWithArc::int(42);
    let small_clone = small.clone();
    println!("Small data (Int): Original and clone both efficient");
    println!("  Size: {} bytes\n", std::mem::size_of_val(&small));
    
    // Large data - Arc makes cloning cheap
    let large = GraphDataWithArc::float_vec(vec![0.0; 1_000_000]);
    println!("Large data (FloatVec with 1M elements):");
    println!("  Size of GraphDataWithArc enum: {} bytes", std::mem::size_of_val(&large));
    println!("  Actual Vec size: ~8 MB");
    
    let large_clone1 = large.clone();
    let large_clone2 = large.clone();
    let large_clone3 = large.clone();
    
    // All point to same data
    if let Some(v) = large.as_float_vec() {
        println!("  Original pointer: {:p}", v.as_ptr());
    }
    if let Some(v) = large_clone1.as_float_vec() {
        println!("  Clone 1 pointer:  {:p}", v.as_ptr());
    }
    if let Some(v) = large_clone2.as_float_vec() {
        println!("  Clone 2 pointer:  {:p}", v.as_ptr());
    }
    if let Some(v) = large_clone3.as_float_vec() {
        println!("  Clone 3 pointer:  {:p}", v.as_ptr());
    }
    
    println!("\n✓ All clones point to the same underlying data!");
    println!("  Memory used: ~8 MB (not 32 MB)");
    println!("  Clone cost: O(1) pointer increment (not O(n) memory copy)");
    
    // When you need to modify (rare case)
    println!("\n=== Mutation (when needed) ===");
    let mut data = GraphDataWithArc::float_vec(vec![1.0, 2.0, 3.0]);
    
    // To mutate, you'd need to get mutable access:
    // This requires the match pattern since we can't easily get mutable Arc
    // This is the main downside - mutation requires careful handling
    println!("Mutation requires Arc::make_mut or creating new data");
    println!("(Usually not needed in DAG execution - data flows forward)");
}
