#!/bin/bash

echo "🚀 ChromaCat Unified Renderer Test"
echo "=================================="
echo ""
echo "Features to verify:"
echo "✓ Pattern renders immediately (not blank)"
echo "✓ Overlay appears ON TOP of pattern"
echo "✓ Overlay shows 4 columns: Patterns, Params, Themes, Art"
echo "✓ Tab to switch columns, arrows to navigate"
echo "✓ Press ; to toggle overlay"
echo ""
echo "Starting with cityscape art..."
echo ""

cargo run --features playground-ui -- --playground --art cityscape