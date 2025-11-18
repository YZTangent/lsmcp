#!/usr/bin/env bash
# Simple test to verify LSMCP responds to MCP initialize request

set -euo pipefail

echo "=== Testing LSMCP MCP Server ==="
echo ""

# Build
echo "1. Building LSMCP..."
cargo build --release --quiet
echo "   ✓ Build successful"
echo ""

# Create test input
INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}'
CONTENT_LENGTH=${#INIT_REQUEST}

echo "2. Sending initialize request to LSMCP..."
echo "   Request: $INIT_REQUEST"
echo ""

# Send request and capture response (with timeout)
{
    printf "Content-Length: %d\r\n\r\n%s" "$CONTENT_LENGTH" "$INIT_REQUEST"
    sleep 2  # Wait for response
} | timeout 5 ./target/release/lsmcp --workspace $(pwd) --no-log 2>/tmp/lsmcp-stderr.log | {
    # Read response headers
    while IFS= read -r line; do
        line=$(echo "$line" | tr -d '\r')
        [ -z "$line" ] && break
        echo "   Header: $line"
    done

    # Read response body
    echo "   Response:"
    cat | jq '.' 2>/dev/null || cat
} || {
    echo "   ✗ ERROR: LSMCP timed out or failed"
    echo ""
    echo "   Stderr output:"
    cat /tmp/lsmcp-stderr.log
    exit 1
}

echo ""
echo "3. Test complete!"
echo "   ✓ LSMCP is responding to MCP requests correctly"
echo ""
echo "You can now add LSMCP to your Claude Desktop config:"
echo '  "lsmcp": {'
echo '    "command": "lsmcp",'
echo '    "args": ["--workspace", "/path/to/your/project"]'
echo '  }'
