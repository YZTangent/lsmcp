# LSP Registry

This directory contains LSP package definitions in TOML format. These configurations are embedded into the LSMCP binary at compile time.

## Registry Structure

Each `.toml` file defines an LSP server configuration with the following format:

```toml
name = "lsp-name"
description = "Brief description"
homepage = "https://..."
licenses = ["MIT"]
languages = ["lang1", "lang2"]
file_extensions = ["ext1", "ext2"]

[source]
type = "External"  # or "Npm", "Cargo", "Pip", "GithubRelease"
command = "lsp-command"

[bin]
primary = "lsp-binary"
additional = []
lsp_args = ["--stdio"]
```

## Available LSP Servers

Currently, the registry includes 20 LSP servers:

### Systems Languages
- **rust-analyzer** - Rust
- **clangd** - C/C++/Objective-C
- **zls** - Zig

### Scripting Languages
- **lua-language-server** - Lua
- **solargraph** - Ruby
- **bash-language-server** - Bash/Shell

### Functional Languages
- **haskell-language-server** - Haskell
- **elixir-ls** - Elixir

### JVM/Scala
- **jdtls** - Java
- **metals** - Scala

### Web Technologies
- **vscode-json-language-server** - JSON
- **vscode-css-language-server** - CSS/SCSS/Less
- **vscode-html-language-server** - HTML
- **svelte-language-server** - Svelte
- **vue-language-server** - Vue

### Markup/Config
- **yaml-language-server** - YAML
- **taplo** - TOML
- **texlab** - LaTeX
- **marksman** - Markdown
- **dockerfile-language-server** - Dockerfile

## Adding New LSP Servers

### Manual Method

1. Create a new `.toml` file in this directory following the format above
2. Rebuild the project to embed the new configuration

### Semi-Automated Method

Use the sync script to generate configurations:

```bash
cargo run --bin sync-mason-registry
```

This will fetch package definitions from the Mason registry and convert them to our TOML format.

## Source Types

- **External**: LSP already installed on the system (e.g., via package manager)
- **Npm**: Install via `npm install -g <package>`
- **Cargo**: Install via `cargo install <crate>`
- **Pip**: Install via `pip install <package>`
- **GithubRelease**: Download from GitHub releases

## Installation Instructions

LSMCP doesn't automatically install LSP servers. Users must install them manually:

### Common Installation Commands

```bash
# Rust
rustup component add rust-analyzer

# TypeScript/JavaScript
npm install -g typescript-language-server typescript

# Python
npm install -g pyright

# Go
go install golang.org/x/tools/gopls@latest

# Lua
brew install lua-language-server  # macOS
# or download from https://github.com/LuaLS/lua-language-server

# YAML, JSON, HTML, CSS (from VSCode)
npm install -g vscode-langservers-extracted

# And so on...
```

## Registry Updates

The registry is synchronized from the [Mason Registry](https://github.com/mason-org/mason-registry), which maintains a comprehensive database of LSP servers, formatters, and linters.

To update the registry:

1. Run `scripts/sync-mason-registry.rs` to fetch latest definitions
2. Review changes
3. Commit updated `.toml` files
4. Rebuild LSMCP to embed the new configurations

## Notes

- Registry files are embedded at compile time via `include_dir!` macro
- Configurations use a 3-tier precedence: user config > registry > built-in defaults
- The registry complements the 4 built-in defaults (TypeScript, Python, Rust, Go)
