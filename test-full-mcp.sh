#!/usr/bin/env bash
# Full MCP protocol test - initialize + tools/list

set -euo pipefail

echo "=== Full MCP Protocol Test ==="
echo ""

# Build
cargo build --release --quiet 2>/dev/null
echo "✓ Build successful"
echo ""

# Test workspace
WORKSPACE=$(pwd)

# Start LSMCP and communicate with it
{
    # 1. Send initialize
    INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}'
    printf "Content-Length: %d\r\n\r\n%s" ${#INIT} "$INIT"

    sleep 1

    # 2. Send tools/list
    TOOLS='{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
    printf "Content-Length: %d\r\n\r\n%s" ${#TOOLS} "$TOOLS"

    sleep 1
} | ./target/release/lsmcp --workspace "$WORKSPACE" --no-log 2>&1 | {
    echo "=== Server Responses ==="
    echo ""

    # Read first response (initialize)
    echo "1. Initialize response:"
    while IFS= read -r line; do
        line=$(echo "$line" | tr -d '\r')
        [ -z "$line" ] && break
    done

    # Read initialize body
    head -c 200 | jq '.' 2>/dev/null || cat
    echo ""
    echo ""

    # Read second response (tools/list)
    echo "2. Tools/list response:"
    while IFS= read -r line; do
        line=$(echo "$line" | tr -d '\r')
        [ -z "$line" ] && break
    done

    # Read tools/list body
    cat | jq '.result.tools[].name' 2>/dev/null || cat
}

echo ""
echo "=== Test Complete ==="
