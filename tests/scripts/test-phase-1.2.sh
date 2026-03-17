#!/bin/bash

# Phase 1.2 Test Script
# Tests tokenization, CamelCase splitting, and acronym matching

echo "==================================="
echo "Phase 1.2 Implementation Test"
echo "==================================="
echo ""

# Run unit tests
echo "Running unit tests..."
cargo test tokenizer --quiet 2>&1 | grep -E "(test result|running)"
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

# Clear cache to force re-indexing with tokenization
echo "Clearing cache to test tokenization..."
rm -f ~/.cache/latui/apps.json
echo "✅ Cache cleared"
echo ""

# Check cache after running
echo "Testing cache generation with tokenization..."
timeout 2s ./target/release/latui 2>&1 &
PID=$!
sleep 1
kill $PID 2>/dev/null
echo ""

if [ -f ~/.cache/latui/apps.json ]; then
    echo "✅ Cache file created with tokenization"
    
    # Check if tokens are present
    if jq -e '.apps[0].name_tokens' ~/.cache/latui/apps.json > /dev/null 2>&1; then
        echo "✅ Tokens found in cache"
        
        # Show sample tokens
        echo ""
        echo "Sample tokenized app:"
        jq '.apps[0] | {name, name_tokens, acronyms}' ~/.cache/latui/apps.json 2>/dev/null | head -15
    else
        echo "⚠️  Tokens not found in cache (may need to rebuild)"
    fi
    
    # Check if acronyms are present
    if jq -e '.apps[0].acronyms' ~/.cache/latui/apps.json > /dev/null 2>&1; then
        echo ""
        echo "✅ Acronyms found in cache"
        
        # Count apps with acronyms
        ACRONYM_COUNT=$(jq '[.apps[] | select(.acronyms | length > 0)] | length' ~/.cache/latui/apps.json 2>/dev/null)
        echo "   Apps with acronyms: $ACRONYM_COUNT"
    fi
else
    echo "⚠️  Cache file not created (app may not have run long enough)"
fi

echo ""
echo "==================================="
echo "Phase 1.2 Test Complete"
echo "==================================="
echo ""
echo "Manual Testing Instructions:"
echo "1. Run: ./target/release/latui"
echo ""
echo "2. Test Acronym Matching:"
echo "   - Type 'gc' → should show Google Chrome, GNOME Calculator"
echo "   - Type 'vsc' → should show Visual Studio Code"
echo "   - Type 'lo' → should show LibreOffice apps"
echo ""
echo "3. Test CamelCase Matching:"
echo "   - Type 'libre' → should show LibreOffice apps"
echo "   - Type 'office' → should show LibreOffice apps"
echo ""
echo "4. Test Token Matching:"
echo "   - Type 'visual code' → should show Visual Studio Code"
echo "   - Type 'file manager' → should show file managers"
echo ""
echo "5. Press ESC to exit"
echo ""
echo "Expected Behavior:"
echo "- Acronym matching works (gc → Google Chrome)"
echo "- CamelCase splitting works (libre → LibreOffice)"
echo "- Multi-token matching works (visual code → VS Code)"
echo "- Fast search (< 10ms for most queries)"
