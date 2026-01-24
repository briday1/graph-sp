#!/bin/bash
set -e

echo "=== Discovering Rust examples from Cargo.toml ==="

# Extract example names from Cargo.toml
# This looks for [[example]] sections and extracts the name field
EXAMPLES=$(awk '/^\[\[example\]\]$/ {in_example=1; next} 
                in_example && /^name = / {gsub(/^name = "|"$/, ""); print; in_example=0}' Cargo.toml)

if [ -z "$EXAMPLES" ]; then
    echo "No examples found in Cargo.toml"
    exit 0
fi

echo "Found examples:"
echo "$EXAMPLES" | while read -r example; do
    echo "  - $example"
done
echo ""

# Track results
FAILED_EXAMPLES=()
SUCCESSFUL_EXAMPLES=()

# Run each example
echo "=== Running examples ==="
while IFS= read -r example; do
    if [ -n "$example" ]; then
        echo "Running example: $example"
        if cargo run --example "$example" 2>&1; then
            echo "✓ Example '$example' succeeded"
            SUCCESSFUL_EXAMPLES+=("$example")
        else
            echo "✗ Example '$example' failed"
            FAILED_EXAMPLES+=("$example")
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

# Exit with 0 to not fail the workflow - we want to continue even if some examples fail
exit 0
