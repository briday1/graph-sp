"""
Variants Demo
=============

This example demonstrates how to create and use variants (parameter sweeps) in graph-sp.
Variants allow you to:
1. Run the same graph multiple times with different parameter values
2. Automatically create branches for each variant
3. Merge results from all variants
4. Enable parallelization for faster execution
"""

import pygraph_sp as gs


def train_model(inputs):
    """Simulate model training with a learning rate parameter."""
    learning_rate = inputs.get("learning_rate", 0.01)
    data = inputs.get("data", 100)
    
    # Simulate training - higher LR = faster but less accurate
    accuracy = min(0.95, 0.5 + (learning_rate * 10) - (learning_rate ** 2) * 20)
    loss = 1.0 - accuracy
    
    print(f"Training with LR={learning_rate:.4f}: accuracy={accuracy:.3f}, loss={loss:.3f}")
    
    return {
        "accuracy": accuracy,
        "loss": loss,
        "learning_rate": learning_rate
    }


def prepare_data(inputs):
    """Prepare dataset."""
    print("Preparing dataset...")
    return {"data": 100}


def select_best(inputs):
    """Select the best result from merged variants."""
    accuracies = inputs.get("accuracies", [])
    losses = inputs.get("losses", [])
    lrs = inputs.get("learning_rates", [])
    
    if not accuracies:
        return {"best_lr": 0.0, "best_accuracy": 0.0}
    
    # Find the index of maximum accuracy
    best_idx = accuracies.index(max(accuracies))
    best_lr = lrs[best_idx]
    best_accuracy = accuracies[best_idx]
    
    print(f"\nBest configuration: LR={best_lr:.4f} with accuracy={best_accuracy:.3f}")
    
    return {
        "best_lr": best_lr,
        "best_accuracy": best_accuracy
    }


def main():
    print("=== Variants Demo: Hyperparameter Sweep ===\n")
    
    # Create graph
    graph = gs.Graph()
    
    # Add data preparation node
    graph.add(
        prepare_data,
        label="Data Preparation",
        outputs=["data"]
    )
    
    # Add model training node
    graph.add(
        train_model,
        label="Train Model",
        inputs=["data", "learning_rate"],
        outputs=["accuracy", "loss", "learning_rate"]
    )
    
    # Connect data prep to training
    graph.add_edge("prepare_data", "data", "train_model", "data")
    
    # Create variants with different learning rates
    print("Creating variants with different learning rates...")
    
    def learning_rate_generator(i):
        """Generate learning rate values: 0.001, 0.005, 0.01, 0.05, 0.1"""
        rates = [0.001, 0.005, 0.01, 0.05, 0.1]
        return gs.PortData(rates[i])
    
    # This will create 5 branches: lr_sweep_0, lr_sweep_1, ..., lr_sweep_4
    # Each branch will have the "learning_rate" input set to a different value
    variant_branches = graph.create_variants(
        name_prefix="lr_sweep",
        count=5,
        param_name="learning_rate",
        variant_function=learning_rate_generator,
        parallel=True  # Enable parallel execution
    )
    
    print(f"Created {len(variant_branches)} variant branches")
    for branch in variant_branches:
        print(f"  - {branch}")
    
    # Add result selection node
    graph.add(
        select_best,
        label="Select Best",
        inputs=["accuracies", "losses", "learning_rates"],
        outputs=["best_lr", "best_accuracy"]
    )
    
    # Merge the accuracy results from all variants
    # The default merge function collects all values into a list
    graph.merge_branches(
        node_id="select_best",
        branches=variant_branches,
        port="accuracy"  # This will merge to "accuracies" (pluralized)
    )
    
    graph.merge_branches(
        node_id="select_best",
        branches=variant_branches,
        port="loss"  # This will merge to "losses"
    )
    
    graph.merge_branches(
        node_id="select_best",
        branches=variant_branches,
        port="learning_rate"  # This will merge to "learning_rates"
    )
    
    # Visualize
    print("\n=== Graph Structure ===")
    print(gs.Inspector.visualize(graph))
    
    # Execute
    print("\n=== Executing Graph ===\n")
    executor = gs.Executor()
    result = executor.execute(graph)
    
    if result.is_success():
        print("\n✓ Execution successful!")
        
        # Get the best result
        best_lr = result.get_output("select_best", "best_lr")
        best_acc = result.get_output("select_best", "best_accuracy")
        print(f"\nFinal Result: Best LR = {best_lr}, Best Accuracy = {best_acc}")
    else:
        print("\n✗ Execution failed")
        for node_id, error in result.errors.items():
            print(f"  - {node_id}: {error}")


def demo_custom_merge():
    """Demo showing custom merge function."""
    print("\n\n=== Custom Merge Demo ===\n")
    
    def process_data(inputs):
        """Process with a batch size parameter."""
        batch_size = inputs.get("batch_size", 32)
        throughput = batch_size * 100  # Simulate throughput
        memory = batch_size * 2  # Simulate memory usage
        
        print(f"Batch size {batch_size}: throughput={throughput}, memory={memory}MB")
        
        return {
            "throughput": throughput,
            "memory": memory,
            "batch_size": batch_size
        }
    
    def compute_average(values):
        """Custom merge function that computes the average."""
        if not values:
            return gs.PortData(0.0)
        total = sum(v for v in values if isinstance(v, (int, float)))
        return gs.PortData(total / len(values))
    
    graph = gs.Graph()
    
    # Add processing node
    graph.add(
        process_data,
        label="Process Data",
        inputs=["batch_size"],
        outputs=["throughput", "memory", "batch_size"]
    )
    
    # Create variants with different batch sizes
    def batch_size_generator(i):
        """Generate batch sizes: 16, 32, 64, 128, 256"""
        sizes = [16, 32, 64, 128, 256]
        return gs.PortData(sizes[i])
    
    variant_branches = graph.create_variants(
        name_prefix="batch_sweep",
        count=5,
        param_name="batch_size",
        variant_function=batch_size_generator
    )
    
    # Add a summary node
    def summarize(inputs):
        avg_throughput = inputs.get("avg_throughput", 0)
        avg_memory = inputs.get("avg_memory", 0)
        print(f"\nAverage across all batch sizes:")
        print(f"  Throughput: {avg_throughput:.0f}")
        print(f"  Memory: {avg_memory:.0f} MB")
        return {}
    
    graph.add(
        summarize,
        label="Summarize",
        inputs=["avg_throughput", "avg_memory"]
    )
    
    # Merge with custom average function
    # Note: The merge will pluralize "throughput" → "avg_throughput"
    graph.merge_branches(
        node_id="summarize",
        branches=variant_branches,
        port="throughput",
        merge_function=compute_average
    )
    
    graph.merge_branches(
        node_id="summarize",
        branches=variant_branches,
        port="memory",
        merge_function=compute_average
    )
    
    # Execute
    print("Executing custom merge demo...\n")
    executor = gs.Executor()
    result = executor.execute(graph)
    
    if result.is_success():
        print("\n✓ Custom merge demo successful!")
    else:
        print("\n✗ Custom merge demo failed")


def demo_nested_variants():
    """Demo showing nested variants (cartesian product)."""
    print("\n\n=== Nested Variants Demo ===\n")
    
    def experiment(inputs):
        lr = inputs.get("learning_rate", 0.01)
        batch = inputs.get("batch_size", 32)
        
        # Simulate experiment result
        score = lr * 100 + batch * 0.1
        print(f"LR={lr:.4f}, Batch={batch}: score={score:.2f}")
        
        return {"score": score}
    
    graph = gs.Graph()
    
    # Add experiment node
    graph.add(
        experiment,
        label="Run Experiment",
        inputs=["learning_rate", "batch_size"],
        outputs=["score"]
    )
    
    # Create first dimension: learning rates
    def lr_gen(i):
        rates = [0.001, 0.01, 0.1]
        return gs.PortData(rates[i])
    
    lr_branches = graph.create_variants(
        name_prefix="lr",
        count=3,
        param_name="learning_rate",
        variant_function=lr_gen
    )
    
    print(f"Created {len(lr_branches)} learning rate variants")
    
    # Create second dimension: batch sizes (nested within each LR branch)
    # This will create a cartesian product: 3 LRs × 4 batch sizes = 12 total branches
    for branch_name in lr_branches:
        branch = graph.get_branch(branch_name)
        
        def batch_gen(i):
            sizes = [16, 32, 64, 128]
            return gs.PortData(sizes[i])
        
        batch_branches = branch.create_variants(
            name_prefix="batch",
            count=4,
            param_name="batch_size",
            variant_function=batch_gen
        )
        
        print(f"  - {branch_name}: created {len(batch_branches)} batch size variants")
    
    # Count total leaf branches
    all_branches = graph.branch_names()
    print(f"\nTotal branches: {len(all_branches)}")
    print("This creates a 3×4 grid of experiments!")
    
    print("\n✓ Nested variants demo complete")
    print("Note: Execute would run all 12 combinations in parallel")


if __name__ == "__main__":
    main()
    demo_custom_merge()
    # Uncomment to see nested variants:
    # demo_nested_variants()
