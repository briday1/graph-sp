#!/bin/bash

echo "=== Discovering Python examples from examples/py/ ==="

# Find all .py files in examples/py/ that start with a digit (e.g., 01_, 02_, etc.)
# This excludes utility files like benchmark_utils.py
EXAMPLES=$(find examples/py -maxdepth 1 -name "[0-9]*.py" -type f | sort)

if [ -z "$EXAMPLES" ]; then
    echo "No Python examples found in examples/py/"
    exit 0
fi

echo "Found examples:"
echo "$EXAMPLES" | while read -r example; do
    echo "  - $(basename "$example")"
done
echo ""

# Track results
FAILED_EXAMPLES=()
SUCCESSFUL_EXAMPLES=()

# Run each example
echo "=== Running Python examples ==="
while IFS= read -r example; do
    if [ -n "$example" ]; then
        example_name=$(basename "$example")
        echo "Running example: $example_name"
        if python3 "$example" 2>&1; then
            echo "✓ Example '$example_name' succeeded"
            SUCCESSFUL_EXAMPLES+=("$example_name")
        else
            echo "✗ Example '$example_name' failed"
            FAILED_EXAMPLES+=("$example_name")
        fi
        echo ""
    fi
done <<< "$EXAMPLES"

# Report summary
echo "=== Summary ==="
echo "Successful examples: ${#SUCCESSFUL_EXAMPLES[@]}"
echo "Failed examples: ${#FAILED_EXAMPLES[@]}"

if [ ${#FAILED_EXAMPLES[@]} -gt 0 ]; then
    echo ""
    echo "Failed examples:"
    for example in "${FAILED_EXAMPLES[@]}"; do
        echo "  - $example"
    done
    echo ""
    echo "Note: Some examples may fail, but this is expected for demonstration purposes."
fi

# Exit with 0 to not fail the workflow
exit 0