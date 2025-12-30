# sing-box Subscription Converter - Rust + Cloudflare Workers Edition

A Rust + Cloudflare Workers port of the original Python sing-box-subscribe project, providing high-performance subscription conversion service.

## ✨ Features

- 🚀 **High Performance** - Rust compiled to WebAssembly, running on Cloudflare's edge network
- 🌍 **Global Deployment** - Automatically deployed to 200+ data centers worldwide
- 💰 **Free Tier** - 100,000 free requests per day with Cloudflare Workers
- 🔒 **Secure & Reliable** - Edge computing, no self-hosted server needed
- ⚡ **Instant Response** - Low latency from nearest edge node

## 📦 Supported Protocols

- ✅ VMess
- ✅ VLESS (including REALITY)
- ✅ Trojan
- ✅ Shadowsocks
- ⏳ Hysteria (planned)
- ⏳ TUIC (planned)

## 🚀 Quick Start

### 1. Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v16+)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/)
- Cloudflare account

### 2. Install Wrangler

```bash
npm install -g wrangler
```

### 3. Login to Cloudflare

```bash
wrangler login
```

### 4. Clone and Build

```bash
cd sing-box-worker-rust
cargo install worker-build
worker-build --release
```

### 5. Deploy to Cloudflare

```bash
wrangler deploy
```

## 📖 Usage

### Basic Usage

```
https://your-worker.workers.dev/config/<subscription-url>
```

### URL Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `emoji` | Add country flag emoji (1=on, 0=off) | `emoji=1` |
| `prefix` | Node name prefix | `prefix=MyVPN` |
| `file` | Config template index (0=basic) | `file=0` |
| `ua` or `UA` | Custom User-Agent | `ua=v2rayng` |
| `enn` | Exclude node names (regex supported) | `enn=expired\|test` |

### Complete Example

```
https://your-worker.workers.dev/config/https://example.com/subscribe?token=abc123&emoji=1&prefix=HK&file=0
```

### Multiple Subscriptions

Use `|` to separate multiple subscription URLs:

```
https://your-worker.workers.dev/config/sub1|sub2|sub3?emoji=1
```

## 🔧 Configuration

### Custom Domain

1. Add domain in Cloudflare Dashboard
2. Edit `wrangler.toml`:

```toml
[[routes]]
pattern = "your-domain.com/*"
zone_name = "your-domain.com"
```

3. Redeploy:

```bash
wrangler deploy
```

### Custom Config Templates

1. Add new JSON templates in `templates/` directory
2. Modify `src/config.rs` to add template loading logic
3. Rebuild and deploy

## ⚠️ Limitations

Due to Cloudflare Workers limitations:

- **CPU Time**: Max 50ms (paid), 10ms (free)
- **Memory**: 128MB
- **Request Size**: Max 100MB
- **Response Size**: Recommended < 10MB

For subscriptions with many nodes, use filter parameters to reduce node count.

## 🆚 Comparison with Original

| Feature | Python (Vercel) | Rust (Cloudflare) |
|---------|----------------|-------------------|
| Platform | Vercel Serverless | Cloudflare Workers |
| Runtime | Python + Flask | Rust + WebAssembly |
| Cold Start | ~500ms | ~5ms |
| Global Nodes | Limited | 200+ edge nodes |
| Free Tier | 100GB/month bandwidth | 100,000 requests/day |
| Custom Domain | ✅ | ✅ |
| Web UI | ✅ | ❌ (API only) |

## 🐛 Troubleshooting

### Build Failed

```bash
# Clean and rebuild
cargo clean
rm -rf build/
worker-build --release
```

### Deploy Failed

```bash
# Check wrangler login status
wrangler whoami

# Re-login
wrangler login

# View detailed errors
wrangler deploy --verbose
```

### Runtime Errors

Check Cloudflare Dashboard Workers logs:
1. Go to Workers & Pages
2. Select your Worker
3. View Logs tab

## 📚 Development

### Local Testing

```bash
wrangler dev
```

Visit `http://localhost:8787`

### View Logs

```bash
wrangler tail
```

### Run Tests

```bash
cargo test
```

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📄 License

Based on [Toperlock/sing-box-subscribe](https://github.com/Toperlock/sing-box-subscribe)

## 🔗 Links

- [Original Python Project](https://github.com/Toperlock/sing-box-subscribe)
- [Cloudflare Workers Docs](https://developers.cloudflare.com/workers/)
- [worker-rs Docs](https://github.com/cloudflare/workers-rs)
- [sing-box Official Docs](https://sing-box.sagernet.org/)

## ⭐ Star History

If this project helps you, please give it a Star ⭐

---

**Disclaimer**: This project is for educational purposes only. Please comply with local laws and regulations.
