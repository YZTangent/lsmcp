# LSMCP - Language Server Manager for MCP

**Bring LSP superpowers to your CLI LLM tools!**

LSMCP is a bridge between the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) and [Language Server Protocol (LSP)](https://microsoft.github.io/language-server-protocol/), enabling CLI-based LLM clients like Claude Code and Gemini CLI to access rich code intelligence without grep/cat operations.

## Features

- 🚀 **Zero-config for popular languages**: TypeScript, Python, Rust, Go work out-of-the-box
- 📦 **24 LSP servers supported**: 4 built-in defaults + 20 from Mason registry
- 🎯 **6 core MCP tools**: goto_definition, find_references, hover, document_symbols, diagnostics, workspace_symbols
- ⚡ **Lazy initialization**: LSP servers start on-demand
- 🔧 **Highly configurable**: 3-tier config system (user → registry → defaults)
- 🦀 **Written in Rust**: Fast, safe, single binary

## Quick Start

### Prerequisites

Install the LSP servers you need:

```bash
# TypeScript/JavaScript
npm install -g typescript-language-server typescript

# Python
npm install -g pyright

# Rust (already have it if you're using rustup!)
rustup component add rust-analyzer

# Go
go install golang.org/x/tools/gopls@latest
```

### Build & Install

```bash
git clone https://github.com/YZTangent/lsmcp
cd lsmcp
cargo build --release
cargo install --path .
```

### Configure for Claude Code

Add to your MCP configuration file:

```json
{
  "mcpServers": {
    "lsmcp": {
      "command": "lsmcp",
      "args": ["--workspace", "/path/to/your/project"]
    }
  }
}
```

If you don't specify `--workspace`, LSMCP will auto-detect your git root or use the current directory.

## Available MCP Tools

### `lsp_goto_definition`

Navigate to where a symbol is defined.

**Parameters:**
- `file` (string): Absolute path to the file
- `line` (integer): Line number (0-indexed)
- `character` (integer): Character offset (0-indexed)

**Returns:** File path and location of the definition(s).

---

### `lsp_find_references`

Find all usages of a symbol.

**Parameters:**
- `file` (string): Absolute path to the file
- `line` (integer): Line number (0-indexed)
- `character` (integer): Character offset (0-indexed)
- `includeDeclaration` (boolean, optional): Include the declaration (default: true)

**Returns:** List of all locations where the symbol is referenced.

---

### `lsp_hover`

Get hover information (documentation, type info, signatures).

**Parameters:**
- `file` (string): Absolute path to the file
- `line` (integer): Line number (0-indexed)
- `character` (integer): Character offset (0-indexed)

**Returns:** Documentation, type information, and function signatures.

---

### `lsp_document_symbols`

Get the symbol outline for a file.

**Parameters:**
- `file` (string): Absolute path to the file

**Returns:** Hierarchical structure of all symbols (classes, functions, variables, etc.).

---

### `lsp_diagnostics`

Get diagnostics (errors, warnings, hints) for a file.

**Parameters:**
- `file` (string): Absolute path to the file

**Returns:** List of diagnostics with severity, location, and message. Shows compiler errors, linting issues, type errors, and other problems detected by the LSP server.

---

### `lsp_workspace_symbols`

Search for symbols across the entire workspace by name or pattern.

**Parameters:**
- `query` (string): Search query (symbol name or pattern)
- `language` (string): Language to search in (e.g., 'rust', 'typescript', 'python', 'go')

**Returns:** List of symbols matching the query with their locations and types. Useful for finding functions, classes, variables, etc. across multiple files.

## Supported Languages

### Built-in (Zero Config)

| Language | LSP Server | Extensions |
|----------|-----------|------------|
| TypeScript/JavaScript | typescript-language-server | `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs` |
| Python | pyright | `.py`, `.pyi` |
| Rust | rust-analyzer | `.rs` |
| Go | gopls | `.go` |

### From Mason Registry (20 LSPs)

- **Systems:** C/C++ (clangd), Zig (zls)
- **Scripting:** Lua, Ruby (solargraph), Bash
- **Functional:** Haskell, Elixir
- **JVM:** Java (jdtls), Scala (metals)
- **Web:** JSON, CSS, HTML, Svelte, Vue
- **Markup/Config:** YAML, TOML (taplo), LaTeX (texlab), Markdown (marksman), Dockerfile

See [`registry/`](registry/) for complete list and installation instructions.

## Configuration

### User Configuration

Create `.lsmcp.toml` in your project root or `~/.config/lsmcp/config.toml`:

```toml
[settings]
log_level = "info"

# Override default LSP for Python
[language_overrides]
python = "pylsp"  # Use pylsp instead of pyright

# Custom LSP configuration
[lsp.my-lsp]
languages = ["mylang"]
file_extensions = ["ml"]
command = "my-lsp-server"
args = ["--stdio"]

# Override LSP command path
[lsp.rust-analyzer]
command = "/custom/path/to/rust-analyzer"
```

### Configuration Precedence

LSMCP uses a 3-tier system:

1. **User config** - Highest priority
2. **Mason registry** (embedded in binary) - Medium priority
3. **Built-in defaults** - Lowest priority

## CLI Options

```bash
lsmcp [OPTIONS]

Options:
  -w, --workspace <WORKSPACE>
          Workspace root directory (auto-detects git root if not specified)

  -l, --log-level <LOG_LEVEL>
          Log level: trace, debug, info, warn, error [default: info]

      --log-file <LOG_FILE>
          Write logs to file instead of stderr

  -h, --help
          Print help

  -V, --version
          Print version
```

## Architecture

```
┌─────────────────┐
│  Claude Code /  │
│   Gemini CLI    │
└────────┬────────┘
         │ MCP Protocol (stdio)
         │
┌────────▼────────────────────────────┐
│       LSMCP (Rust Binary)           │
│  ┌──────────────────────────────┐   │
│  │  MCP Server                  │   │
│  │  - JSON-RPC over stdio       │   │
│  │  - Tool handlers             │   │
│  └──────────┬───────────────────┘   │
│             │                        │
│  ┌──────────▼───────────────────┐   │
│  │    LSP Manager               │   │
│  │  - Lazy initialization       │   │
│  │  - Process lifecycle         │   │
│  └──────────┬───────────────────┘   │
│             │                        │
│  ┌──────────▼───────────────────┐   │
│  │  LSP Client Pool             │   │
│  │  - Per-language clients      │   │
│  │  - JSON-RPC over stdin/out   │   │
│  └──────────┬───────────────────┘   │
└─────────────┼────────────────────────┘
              │ LSP Protocol
     ┌────────┴────────┐
     │                 │
┌────▼─────┐    ┌─────▼──────┐
│typescript│    │  rust-     │
│-language-│    │  analyzer  │ ...
│ server   │    │            │
└───────────┘    └────────────┘
```

## How It Works

1. **MCP Client** (Claude Code) sends a tool call request via stdin
2. **MCP Server** parses the JSON-RPC request
3. **LSP Manager** routes to the appropriate LSP client (spawns if needed)
4. **LSP Client** communicates with the language server
5. **Response flows back** through the chain, formatted for the MCP client

## Development

### Project Structure

```
lsmcp/
├── src/
│   ├── config/       # Configuration system (3-tier)
│   ├── lsp/          # LSP client & manager
│   ├── mcp/          # MCP server & tools
│   ├── types/        # Error types
│   └── utils/        # Utilities
├── registry/         # LSP package definitions (20 LSPs)
├── scripts/          # Registry sync scripts
└── docs/             # Architecture & planning docs
```

### Running Tests

```bash
cargo test
```

### Building for Release

```bash
cargo build --release
```

Binary will be in `target/release/lsmcp`.

## Contributing

Contributions welcome! Areas for improvement:

- Add more MCP tools (workspace symbols, call hierarchy, diagnostics)
- Support more languages
- Improve error messages
- Add integration tests
- Performance optimizations

## License

Dual-licensed under MIT or Apache-2.0.

## Credits

- Built with the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- Inspired by [Mason](https://github.com/mason-org/mason-registry) registry
- Implements [Model Context Protocol](https://modelcontextprotocol.io/)

---

**Made with 🦀**
