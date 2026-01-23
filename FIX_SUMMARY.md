# Data Flow Dependency Resolution Fix

## Summary

Fixed a critical issue in the graph builder where node dependencies were determined by insertion order rather than actual data flow, causing:
1. **Incorrect Mermaid visualizations** - showing sequential chains instead of actual dependencies
2. **Severe performance degradation** - forcing sequential execution of independent nodes

## The Problem

### Before (Broken)
```rust
// In builder.rs add() method
} else if let Some(pid) = parent {
    node.dependencies.push(pid);  // ❌ Always depends on previous node
}
```

This created a sequential dependency chain regardless of actual data needs:
```
UserData → TransactionData → BehaviorData → AgeGrouping → ...
```

### Impact
- **Visualization**: Mermaid showed `0 → 1 → 2 → 3 → ...` instead of actual data flow
- **Performance**: 3 independent 0.3s tasks took 0.9s instead of 0.3s (no parallelism!)

## The Solution

### After (Fixed)
```rust
// In builder.rs:

// 1. Don't auto-create frontier dependencies in add()
if !self.merge_targets.is_empty() {
    node.dependencies.extend(self.merge_targets.iter().copied());
    self.merge_targets.clear();
}
// Note: We no longer automatically add frontier dependencies here
// Dependencies will be resolved based on data flow in build()

// 2. Resolve dependencies based on input/output mappings in build()
fn resolve_data_dependencies(&mut self) {
    // Build a map of which nodes produce which broadcast variables
    let mut producers: HashMap<String, Vec<NodeId>> = HashMap::new();
    
    for node in &self.nodes {
        for broadcast_var in node.output_mapping.values() {
            producers.entry(broadcast_var.clone())
                .or_insert_with(Vec::new)
                .push(node.id);
        }
    }

    // For each node, find dependencies based on required inputs
    for i in 0..self.nodes.len() {
        let node = &self.nodes[i];
        let required_inputs: Vec<String> = node.input_mapping.keys().cloned().collect();
        let node_id = node.id;
        
        let mut dependencies: HashSet<NodeId> = HashSet::new();
        
        // Keep any existing dependencies (from merge_targets or branches)
        dependencies.extend(node.dependencies.iter().copied());
        
        // Add dependencies based on data flow
        for broadcast_var in &required_inputs {
            if let Some(producer_ids) = producers.get(broadcast_var) {
                for &producer_id in producer_ids {
                    if producer_id != node_id {
                        dependencies.insert(producer_id);
                    }
                }
            }
        }
        
        self.nodes[i].dependencies = dependencies.into_iter().collect();
    }
}
```

## Results

### ✅ Correct Visualization
Mermaid now shows actual data dependencies:
```
0 -->|age → age| 3
1 -->|transactions → transactions| 4
2 -->|page_views → page_views<br/>clicks → clicks...| 5
```

### ✅ True Parallelism
Independent nodes now execute concurrently:
- **Before**: 3 × 0.3s tasks = 0.9s (sequential)
- **After**: 3 × 0.3s tasks = 0.3s (parallel) ✨

### ✅ Complex DAGs Work
Multi-level pipelines with fan-out/fan-in execute correctly with proper dependency resolution.

## Test Results

All verification tests pass:
- ✅ Independent nodes run in parallel
- ✅ Data dependencies create correct graph structure  
- ✅ Complex multi-level pipelines work correctly
- ✅ Mermaid diagrams show actual data flow

## Files Modified

- `src/builder.rs`: 
  - Modified `add()` to not auto-create frontier dependencies
  - Added `resolve_data_dependencies()` method
  - Updated `build()` to call dependency resolution

## Impact

This fix fundamentally changes how the graph builder works:
- **Before**: Imperative sequential chaining based on insertion order
- **After**: Declarative data flow based on input/output mappings

The API remains the same, but now works as users would naturally expect!
