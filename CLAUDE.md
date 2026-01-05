# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A high-performance sing-box subscription converter built with Rust and compiled to WebAssembly for Cloudflare Workers. Converts proxy subscription URLs (VMess, VLESS, Trojan, Shadowsocks) into sing-box configuration JSON.

## Development Commands

### Build and Deploy
```bash
# Install worker-build tool (one-time)
cargo install worker-build

# Build for production (creates WebAssembly)
worker-build --release

# Local development server
wrangler dev

# Deploy to Cloudflare Workers
wrangler deploy

# Deploy with verbose output
wrangler deploy --verbose
```

### Testing and Debugging
```bash
# Run Rust tests
cargo test

# View live logs from deployed worker
wrangler tail

# Check Wrangler authentication status
wrangler whoami

# Clean build artifacts
cargo clean
rm -rf build/
```

### Troubleshooting
```bash
# Re-login to Cloudflare
wrangler login

# Clean rebuild
cargo clean && rm -rf build/ && worker-build --release
```

## Architecture

### Request Flow
1. **Entry Point** (`src/lib.rs`): Router handles incoming HTTP requests
   - Root `/`: Returns web UI for interactive conversion (`ui.html`)
   - `/sub`: Main subscription conversion endpoint via `handle_config()`

2. **Subscription Processing** (`handle_config()` in `src/lib.rs`):
   - Parses query parameters including `urls` (subscription URLs, pipe-separated)
   - Fetches subscription content with custom User-Agent
   - If `config` parameter provided, fetches remote template; otherwise uses built-in template
   - Delegates to parsers for protocol detection
   - Applies filters (emoji flags, prefix)
   - Merges nodes into config template
   - Returns JSON response

3. **Protocol Parsers** (`src/parsers/`):
   - `mod.rs`: Orchestrates parsing, handles base64 decoding, routes to protocol-specific parsers
   - `vmess.rs`: VMess protocol (base64 JSON format)
   - `vless.rs`: VLESS protocol including REALITY support
   - `trojan.rs`: Trojan protocol
   - `shadowsocks.rs`: Shadowsocks protocol (ss:// links)
   - Each parser returns `serde_json::Value` representing a sing-box outbound

4. **Config Template** (`src/config.rs`):
   - Loads template: custom remote template (if provided) or built-in template from `templates/basic.json`
   - Processes template placeholders and filters:
     - `{all}` placeholder: Replaced with all node tags
     - `filter` field: Applies include/exclude rules based on keywords
   - Removes `filter` fields from final output (not sing-box standard)
   - Removes outbounds with empty outbounds list (prevents sing-box startup errors)
   - Cleans up references to deleted outbounds in other outbounds (maintains dependency integrity)
   - Appends full node configs to outbounds array

5. **Utilities** (`src/utils.rs`):
   - `add_emoji()`: Adds country flag emojis based on regex patterns
   - Helper functions for filtering and deduplication (unused in current flow)

### Data Flow
```
HTTP Request (/sub?urls=...&config=...)
  → Router (lib.rs)
    → Parse query parameters
      → Extract urls parameter (pipe-separated subscription URLs)
      → Fetch subscription content for each URL
      → Fetch remote template (if config param provided)
        → Parse (parsers/mod.rs)
          → Protocol-specific parser (vmess/vless/trojan/shadowsocks)
            → Vec<serde_json::Value> nodes
              → Apply filters (emoji, prefix)
                → Merge into template (config.rs)
                  → JSON Response
```

### Key Design Patterns
- **Protocol abstraction**: Each parser module exposes a `parse()` function returning `Result<Value>`
- **Template-based config**: Nodes injected into pre-defined sing-box config structure
- **Stateless processing**: No persistence, pure request-response transformation
- **Lazy regex compilation**: `OnceLock` pattern for emoji regex patterns (initialized once)
- **Query-based routing**: All parameters passed via query string for clarity and flexibility

## Configuration Files

### wrangler.toml
- `name`: Worker name (appears as subdomain: `{name}.{account}.workers.dev`)
- `compatibility_date`: Cloudflare Workers API version
- `[build]`: Build command executed during deployment
- `[observability]`: Logging settings for Cloudflare dashboard

### Cargo.toml
- `crate-type = ["cdylib"]`: Compiles to WebAssembly library
- `[profile.release]`: Aggressive optimization for WASM size
  - `opt-level = "z"`: Optimize for size
  - `lto = true`: Link-time optimization
  - `strip = true`: Remove debug symbols

### templates/basic.json
- Base sing-box configuration with DNS, inbounds, outbounds, routing rules
- `outbounds[0]` must be a selector tagged `"Proxy"` where nodes are injected
- Template structure: log → dns → inbounds (tun) → outbounds (selector, direct, block, dns) → route

## Cloudflare Workers Constraints

- **CPU Time**: 10ms (free tier), 50ms (paid) - optimize for parsing speed
- **Memory**: 128MB limit
- **Response Size**: Keep configs < 10MB (filter large node lists)
- **No filesystem**: Templates embedded via `include_str!()` macro at compile time

## Query Parameters

| Parameter | Type | Purpose | Example |
|-----------|------|---------|---------|
| `urls` | URL(s) | Subscription URL(s), pipe-separated for multiple | `urls=https://sub1.com\|https://sub2.com` |
| `config` | URL | Remote template URL to use instead of built-in template | `config=https://example.com/config.json` |
| `emoji` | 0/1 | Add country flags to node names | `emoji=1` |
| `prefix` | string | Prepend text to all node names | `prefix=HK` |
| `file` | number | Template index (only 0 supported, ignored if `config` is provided) | `file=0` |
| `ua` / `UA` | string | Custom User-Agent for fetching | `ua=v2rayng` |
| `enn` | regex | Exclude nodes by name pattern | `enn=过期\|到期` |

## Adding New Features

### Adding a New Protocol Parser
1. Create `src/parsers/{protocol}.rs`
2. Implement `pub fn parse(uri: &str) -> Result<Value>`
3. Add module to `src/parsers/mod.rs`
4. Add protocol detection in `parse_node()` function

### Adding a New Template
1. Add JSON file to `templates/` directory
2. Add `const TEMPLATE_X: &str = include_str!("../templates/{name}.json");` in `config.rs`
3. Update `process_config()` match statement with new index

### Adding a New Filter
1. Implement filter logic in `src/utils.rs`
2. Parse query parameter in `handle_config()` (`src/lib.rs`)
3. Apply filter to `processed_nodes` before config generation

## Web UI

访问根路径（`/`）会显示一个交互式 Web 界面：
- **功能**: 在线转换订阅并下载配置文件
- **特性**:
  - 支持多个订阅地址（每行一个）
  - 可选自定义模板 URL
  - 节点前缀和 Emoji 选项
  - 一键转换并下载
  - 复制 API 链接
- **文件**: `src/ui.html` (编译时嵌入到 WASM 中)

## Testing URLs

```bash
# 使用 Web UI
https://your-worker.workers.dev/

# Local testing with built-in template
http://localhost:8787/sub?urls=https://example.com/api/v1/subscribe?token=abc123&emoji=1

# Local testing with custom remote template
http://localhost:8787/sub?urls=https://example.com/api/v1/subscribe?token=abc123&config=https://example.com/config.json

# Local testing with multiple subscriptions
http://localhost:8787/sub?urls=https://sub1.com/api?token=xxx|https://sub2.com/api?token=yyy&emoji=1

# Production testing with multiple subscriptions and custom template
https://{worker-name}.{account}.workers.dev/sub?urls=https://sub1.com/api?token=xxx|https://sub2.com/api&config=https://example.com/config.json&emoji=1
```

### URL Format
All parameters are passed via query string:
- **Required**: `urls` - Single or pipe-separated subscription URLs
- **Optional**: `config`, `emoji`, `prefix`, `file`, `ua`, `enn`

**Example:**
```bash
wget "http://localhost:8787/sub?urls=https://sub1.com/api?token=abc|https://sub2.com/api&config=https://example.com/template.json&emoji=1" -O config.json
```

### Template Syntax Support

The converter supports advanced template syntax in custom templates:

#### `{all}` Placeholder
Automatically replaced with all node tags from subscriptions:
```json
{
  "tag": "auto-select",
  "type": "urltest",
  "outbounds": ["{all}"]
}
```
Becomes:
```json
{
  "tag": "auto-select",
  "type": "urltest",
  "outbounds": ["node1", "node2", "node3"]
}
```

#### Filter Rules
Filter nodes by keywords using `filter` field:
```json
{
  "tag": "US-nodes",
  "type": "selector",
  "outbounds": ["{all}"],
  "filter": [
    {
      "action": "include",
      "keywords": ["🇺🇸|US|us|美国|美|United States"]
    },
    {
      "action": "exclude",
      "keywords": ["频道|订阅|过期"]
    }
  ]
}
```

**Filter actions:**
- `include`: Only include nodes matching keywords (regex supported)
- `exclude`: Exclude nodes matching keywords (regex supported)

**Example of automatic cleanup:**

Template has:
```json
{
  "tag": "🚀 节点选择",
  "outbounds": ["♻️ 自动选择", "🇯🇵 日本节点", "🇺🇸 美国节点", "DIRECT"]
},
{
  "tag": "🇯🇵 日本节点",
  "outbounds": ["{all}"],
  "filter": [{"action": "include", "keywords": ["日本|JP"]}]
}
```

If no Japanese nodes in subscription:
1. `🇯🇵 日本节点` has empty outbounds → removed
2. Reference to `🇯🇵 日本节点` removed from `🚀 节点选择`

Result:
```json
{
  "tag": "🚀 节点选择",
  "outbounds": ["♻️ 自动选择", "🇺🇸 美国节点", "DIRECT"]
}
// 🇯🇵 日本节点 completely removed
```

**Notes:**
- Multiple filter rules are applied sequentially
- Keywords use pipe `|` as OR separator (regex syntax)
- `filter` fields are automatically removed from final output
- Outbounds with empty outbounds list are automatically removed (prevents sing-box errors)
- References to removed outbounds are automatically cleaned from other outbounds

## Common Issues

- **Build fails**: Ensure `wasm32-unknown-unknown` target installed: `rustup target add wasm32-unknown-unknown`
- **Large configs timeout**: Filter nodes using `enn` parameter or split subscriptions
- **Emoji not showing**: Ensure UTF-8 encoding and check regex patterns in `utils.rs`
- **Parse errors**: Check protocol parser expects correct URI format (see sing-box/v2ray specs)
