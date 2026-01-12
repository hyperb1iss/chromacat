#!/bin/bash

echo "🎨 Testing Unified Ratatui Renderer"
echo "===================================="
echo ""
echo "This should now:"
echo "1. Show the pattern immediately (not blank)"
echo "2. Display overlay on TOP of the pattern"
echo "3. Both rendered through ratatui in one pass"
echo ""
echo "Starting playground with cityscape art..."
echo ""

# Clear debug log
rm -f /tmp/chromacat_debug.log

# Run playground
cargo run --features playground-ui -- --playground --art cityscape