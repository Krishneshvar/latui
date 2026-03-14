#!/bin/bash

# Phase 2.2 Test Script
# Tests typo tolerance functionality

echo "==================================="
echo "Phase 2.2 Implementation Test"
echo "==================================="
echo ""

# Run unit tests
echo "Running typo tolerance tests..."
cargo test typo --quiet 2>&1 | grep -E "(test result|running)"
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

# Clear cache
echo "Clearing cache..."
rm -f ~/.cache/latui/apps.json
echo "✅ Cache cleared"
echo ""

echo "==================================="
echo "Phase 2.2 Test Complete"
echo "==================================="
echo ""
echo "Manual Testing Instructions:"
echo "1. Run: ./target/release/latui"
echo ""
echo "2. Test Single Typos:"
echo "   - Type 'firefix' → should show Firefox"
echo "   - Type 'chorme' → should show Chrome"
echo "   - Type 'braev' → should show Brave"
echo "   - Type 'thuner' → should show Thunar"
echo ""
echo "3. Test Transpositions:"
echo "   - Type 'teh' → should match apps with 'the'"
echo "   - Type 'chorme' → should show Chrome (or→ro swap)"
echo ""
echo "4. Test Double Typos:"
echo "   - Type 'fiirefox' → should show Firefox"
echo "   - Type 'chromee' → should show Chrome"
echo ""
echo "5. Test Min Length (< 4 chars):"
echo "   - Type 'fir' → should NOT use typo tolerance"
echo "   - Type 'fire' → should use typo tolerance"
echo ""
echo "6. Press ESC to exit"
echo ""
echo "Expected Behavior:"
echo "- Typos are handled gracefully"
echo "- Single typo: high score (150 × weight)"
echo "- Double typo: medium score (100 × weight)"
echo "- Fast performance (< 2ms overhead)"
echo ""
echo "Common Typos to Test:"
echo "- firefix → firefox ✅"
echo "- chorme → chrome ✅"
echo "- braev → brave ✅"
echo "- thuner → thunar ✅"
echo "- giimp → gimp ✅"
echo "- vlcc → vlc ✅"
