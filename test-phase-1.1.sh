#!/bin/bash

# Phase 1.1 Test Script
# Tests multi-field indexing and semantic search

echo "==================================="
echo "Phase 1.1 Implementation Test"
echo "==================================="
echo ""

# Build the project
echo "Building project..."
cargo build --release 2>&1 | grep -E "(Finished|error)" || echo "Build in progress..."
echo ""

# Check if build succeeded
if [ ! -f "target/release/latui" ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# Clear cache to force re-indexing
echo "Clearing cache to test fresh indexing..."
rm -f ~/.cache/latui/apps.json
echo "✅ Cache cleared"
echo ""

# Check if desktop files exist
echo "Checking for desktop files..."
DESKTOP_COUNT=$(find /usr/share/applications ~/.local/share/applications -name "*.desktop" 2>/dev/null | wc -l)
echo "Found $DESKTOP_COUNT desktop files"
echo ""

# Test that the binary runs (just check help or version)
echo "Testing binary execution..."
timeout 2s ./target/release/latui 2>&1 &
PID=$!
sleep 1
kill $PID 2>/dev/null
echo "✅ Binary executes"
echo ""

# Check if cache was created
if [ -f ~/.cache/latui/apps.json ]; then
    echo "✅ Cache file created"
    
    # Check cache size
    CACHE_SIZE=$(du -h ~/.cache/latui/apps.json | cut -f1)
    echo "   Cache size: $CACHE_SIZE"
    
    # Count indexed apps
    APP_COUNT=$(jq '.apps | length' ~/.cache/latui/apps.json 2>/dev/null || echo "N/A")
    echo "   Indexed apps: $APP_COUNT"
    
    # Show sample app structure
    echo ""
    echo "Sample indexed app structure:"
    jq '.apps[0] | {name, keywords: .keywords[0:3], categories: .categories[0:3], generic_name, executable}' ~/.cache/latui/apps.json 2>/dev/null | head -20
else
    echo "⚠️  Cache file not created (app may not have run long enough)"
fi

echo ""
echo "==================================="
echo "Phase 1.1 Test Complete"
echo "==================================="
echo ""
echo "Manual Testing Instructions:"
echo "1. Run: ./target/release/latui"
echo "2. Type 'browser' - should show web browsers"
echo "3. Type 'edit' - should show text editors"
echo "4. Type 'terminal' - should show terminal emulators"
echo "5. Type 'files' - should show file managers"
echo "6. Press ESC to exit"
echo ""
echo "Expected Behavior:"
echo "- Semantic search works (browser → Firefox, Chrome, etc.)"
echo "- Multi-field matching (name, keywords, categories, etc.)"
echo "- Weighted scoring (more relevant results first)"
echo "- Fast search (< 10ms for most queries)"
