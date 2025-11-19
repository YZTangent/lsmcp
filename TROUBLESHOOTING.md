# Troubleshooting LSMCP with Claude Desktop

If Claude Desktop times out when trying to connect to LSMCP, follow these steps:

## Step 1: Verify LSMCP is Installed

```bash
# Check if lsmcp is in PATH
which lsmcp

# Or find it manually
find ~/.cargo/bin -name lsmcp 2>/dev/null
```

**Expected**: Should show path like `/home/username/.cargo/bin/lsmcp`

If not found:
```bash
cd /path/to/lsmcp
cargo install --path .
```

## Step 2: Test LSMCP Manually

```bash
# Test that LSMCP starts and responds
cd /path/to/lsmcp
./test-mcp-init.sh
```

**Expected**: Should show "✓ LSMCP is responding to MCP requests correctly"

## Step 3: Check Claude Desktop Config

Your config file should be at:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

### Correct Configuration

```json
{
  "mcpServers": {
    "lsmcp": {
      "command": "/full/path/to/lsmcp",
      "args": ["--workspace", "/absolute/path/to/your/project"]
    }
  }
}
```

**Important**:
- Use **full absolute paths** (run `which lsmcp` to get the command path)
- Use **absolute workspace path** (not `~` or relative paths)
- Ensure workspace directory exists

### Example with Debug Logging

```json
{
  "mcpServers": {
    "lsmcp": {
      "command": "/home/username/.cargo/bin/lsmcp",
      "args": [
        "--workspace", "/home/username/projects/myproject",
        "--log-file", "/tmp/lsmcp-debug.log",
        "--log-level", "debug"
      ]
    }
  }
}
```

Then check `/tmp/lsmcp-debug.log` for errors.

## Step 4: Test with Explicit Paths

Create a test config with explicit paths:

```bash
# Get the full lsmcp path
LSMCP_PATH=$(which lsmcp)
echo "LSMCP is at: $LSMCP_PATH"

# Get your project path
PROJECT_PATH=$(pwd)
echo "Project is at: $PROJECT_PATH"
```

Then use these exact paths in your config.

## Step 5: Check Permissions

```bash
# Check if lsmcp is executable
ls -la $(which lsmcp)

# Should show -rwxr-xr-x (executable)

# Check if workspace is readable
ls -la /path/to/your/project

# Check if LSMCP can create its data directory
mkdir -p ~/.local/share/lsmcp
ls -la ~/.local/share/lsmcp
```

## Step 6: Test with Minimal Config

Try the simplest possible config first:

```json
{
  "mcpServers": {
    "lsmcp": {
      "command": "lsmcp",
      "args": ["--no-log"]
    }
  }
}
```

This uses:
- Default PATH lookup for `lsmcp`
- Current directory as workspace (not ideal but tests connectivity)
- No logging to avoid any stderr issues

## Step 7: Check Claude Desktop Logs

Claude Desktop has its own logs. Look for them at:
- **macOS**: `~/Library/Logs/Claude/`
- **Linux**: `~/.config/Claude/logs/`

Look for errors like:
- "Command not found"
- "Permission denied"
- "Timeout"
- JSON parsing errors

## Step 8: Verify MCP Protocol

Test the full MCP flow:

```bash
cd /path/to/lsmcp
./test-full-mcp.sh
```

This tests both `initialize` and `tools/list` requests.

## Common Issues

### Issue: "Command not found"
**Solution**: Use full path to lsmcp in config
```json
"command": "/home/username/.cargo/bin/lsmcp"
```

### Issue: "Timeout"
**Solutions**:
1. Add `--no-log` flag to disable all logging
2. Use `--log-file` to redirect logs away from stderr
3. Check that workspace path is correct and accessible

### Issue: "Permission denied"
**Solution**: Make sure lsmcp is executable
```bash
chmod +x ~/.cargo/bin/lsmcp
```

### Issue: "Failed to create LSP manager"
**Solution**: Check that `~/.local/share/lsmcp` can be created
```bash
mkdir -p ~/.local/share/lsmcp
```

## Still Not Working?

1. **Restart Claude Desktop** after changing config
2. **Check you're editing the right config file** (Claude Desktop vs Claude Code)
3. **Try a different workspace** to rule out project-specific issues
4. **Run LSMCP manually** and paste the initialize request:

```bash
lsmcp --workspace /your/project --log-file /tmp/lsmcp.log

# Then paste this and press Enter:
Content-Length: 142

{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}
```

5. **Check the debug log** at `/tmp/lsmcp.log`

## Need Help?

If none of these work, please open an issue with:
1. Your OS and Claude Desktop version
2. Output of `lsmcp --version`
3. Your Claude Desktop config (sanitized)
4. Output of `./test-mcp-init.sh`
5. Claude Desktop logs
6. LSMCP debug log (`/tmp/lsmcp.log`)
