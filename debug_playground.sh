#!/bin/bash

echo "🔍 ChromaCat Playground Debug Test"
echo "=================================="
echo ""
echo "Debug log will be written to: /tmp/chromacat_debug.log"
echo "You can watch it in another terminal with: tail -f /tmp/chromacat_debug.log"
echo ""
echo "Starting playground with --art cityscape..."
echo ""

# Clear previous debug log
rm -f /tmp/chromacat_debug.log

# Run with cityscape art
cargo run --features playground-ui -- --playground --art cityscape