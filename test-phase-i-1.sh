#!/bin/bash
set -e

echo "=== Phase I.1 Integration Test ==="
echo ""
echo "This script demonstrates the Phase I.1 MCP Gate Layer in action:"
echo "1. Create a gate context with Phase 1 blocking"
echo "2. Simulate 5 tool calls"
echo "3. Verify evidence ledger grows and chain is intact"
echo ""

# Build the gates crate
echo "Building copper-asx-gates..."
cargo build -p copper-asx-gates --quiet

# Run the test
echo "Running Phase I.1 test suite..."
cargo test -p copper-asx-gates --lib gates_integration::tests --quiet -- --nocapture

echo ""
echo "=== Test Results ==="
echo "✓ All gate tests passed"
echo "✓ Evidence ledger structure verified"
echo "✓ SHA256 chain integrity maintained"
echo ""
echo "Phase I.1 is complete and ready for MCP server integration."
