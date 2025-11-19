#!/bin/bash

# Test script for gemini-bridge.js
# Tests the bridge script with various scenarios

set -e

BRIDGE_SCRIPT="./gemini-bridge.js"
TEST_DIR="$(dirname "$0")"

echo "=== Testing Gemini Bridge Script ==="
echo ""

# Test 1: Invalid JSON
echo "Test 1: Invalid JSON input"
echo "invalid json" | timeout 5 node "$BRIDGE_SCRIPT" 2>&1 | head -1
echo ""

# Test 2: Missing type field
echo "Test 2: Missing 'type' field"
echo '{"content": "test"}' | timeout 5 node "$BRIDGE_SCRIPT" 2>&1 | head -1
echo ""

# Test 3: Unknown request type
echo "Test 3: Unknown request type"
echo '{"type": "unknown"}' | timeout 5 node "$BRIDGE_SCRIPT" 2>&1 | head -1
echo ""

# Test 4: Missing content
echo "Test 4: Missing content field"
echo '{"type": "message"}' | timeout 5 node "$BRIDGE_SCRIPT" 2>&1 | head -1
echo ""

# Test 5: Valid message (will fail if not authenticated, but tests structure)
echo "Test 5: Valid message (may fail without auth, testing structure)"
echo '{"type": "message", "content": "Hello, world!", "model": "gemini-2.5-flash"}' | timeout 30 node "$BRIDGE_SCRIPT" 2>&1 | head -1
echo ""

echo "=== Tests Complete ==="

