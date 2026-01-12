#!/bin/bash

echo "Testing playground mode..."
rm -f /tmp/chromacat_debug.log

# Run in background
cargo run --features playground-ui -- --playground --art cityscape &
PID=$!

echo "Process started with PID $PID"
echo "Waiting 3 seconds..."
sleep 3

# Check if it's still running
if ps -p $PID > /dev/null; then
    echo "Process is still running (might be working or hung)"
    echo ""
    echo "Debug log contents:"
    cat /tmp/chromacat_debug.log 2>/dev/null || echo "No debug log found"
    echo ""
    echo "Killing process..."
    kill $PID 2>/dev/null
else
    echo "Process exited"
fi