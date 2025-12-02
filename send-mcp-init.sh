#!/usr/bin/env bash
# Send a mock MCP initialization request to stdio
# Usage: ./send-mcp-init.sh | lsmcp --workspace /path/to/project --log-file /tmp/lsmcp.log

set -euo pipefail

# MCP initialize request (JSON-RPC 2.0)
INIT_REQUEST='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'

# Write request to stdout
printf "$INIT_REQUEST"

# Optional: send a tools/list request after initialization
sleep 0.5
LIST_TOOLS='{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
printf "$LIST_TOOLS"
