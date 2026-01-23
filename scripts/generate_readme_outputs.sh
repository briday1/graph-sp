#!/usr/bin/env bash
# Script to generate and capture example outputs for README documentation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$REPO_ROOT/scripts/outputs"

mkdir -p "$OUTPUT_DIR"

echo "═══════════════════════════════════════════════════════════"
echo "  Generating Example Outputs"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Rust examples
echo "Running Rust examples..."
echo "────────────────────────────────────────────────────────────"

cd "$REPO_ROOT"

for example in 01_minimal_pipeline 02_parallel_vs_sequential 03_branch_and_merge 04_variants_sweep 05_output_access 06_graphdata_large_payload_arc_or_shared_data; do
    echo "  Running: $example"
    cargo run --example "$example" --release 2>&1 | tee "$OUTPUT_DIR/rs_${example}.txt"
    echo ""
done

echo ""
echo "Running Python examples..."
echo "────────────────────────────────────────────────────────────"

for example in 01_minimal_pipeline 02_parallel_vs_sequential 03_branch_and_merge 04_variants_sweep 05_output_access 06_graphdata_large_payload_arc_or_shared_data; do
    echo "  Running: $example"
    python3 "examples/py/${example}.py" 2>&1 | tee "$OUTPUT_DIR/py_${example}.txt"
    echo ""
done

echo "═══════════════════════════════════════════════════════════"
echo "  All outputs captured in: $OUTPUT_DIR"
echo "═══════════════════════════════════════════════════════════"
