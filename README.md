# sing-box Subscription Converter - Rust + Cloudflare Workers Edition

这是原 Python 版本 sing-box-subscribe 的 Rust + Cloudflare Workers 移植版本，提供高性能的订阅转换服务。

## ✨ 特性

- 🚀 **高性能** - Rust 编译为 WebAssembly，运行在 Cloudflare 边缘网络
- 🌍 **全球部署** - 自动部署到全球 200+ 数据中心
- 💰 **免费额度** - Cloudflare Workers 每天 100,000 次免费请求
- 🔒 **安全可靠** - 边缘计算，无需自建服务器
- ⚡ **即时响应** - 边缘节点就近响应，延迟极低

## 📦 支持的协议

- ✅ VMess
- ✅ VLESS (包括 REALITY)
- ✅ Trojan
- ✅ Shadowsocks
- ⏳ Hysteria (计划中)
- ⏳ TUIC (计划中)

## 🚀 快速开始

### 1. 前置要求

- [Rust](https://rustup.rs/) (最新稳定版)
- [Node.js](https://nodejs.org/) (v16+)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/)
- Cloudflare 账号

### 2. 安装 Wrangler

```bash
npm install -g wrangler
```

### 3. 登录 Cloudflare

```bash
wrangler login
```

### 4. 克隆并构建项目

```bash
cd sing-box-worker-rust
cargo install worker-build
worker-build --release
```

### 5. 部署到 Cloudflare

```bash
wrangler deploy
```

## 📖 使用方法

### 基本用法

```
https://your-worker.workers.dev/config/<订阅链接>
```

### URL 参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `emoji` | 添加国旗 emoji (1=开启, 0=关闭) | `emoji=1` |
| `prefix` | 节点名称前缀 | `prefix=MyVPN` |
| `file` | 配置模板索引 (0=基础模板) | `file=0` |
| `ua` 或 `UA` | 自定义 User-Agent | `ua=v2rayng` |
| `enn` | 排除节点名称关键词 (支持正则) | `enn=过期|到期` |

### 完整示例

```
https://your-worker.workers.dev/config/https://example.com/subscribe?token=abc123&emoji=1&prefix=HK&file=0
```

### 多订阅聚合

使用 `|` 分隔多个订阅链接：

```
https://your-worker.workers.dev/config/订阅1|订阅2|订阅3?emoji=1
```

## 🔧 配置

### 自定义域名

1. 在 Cloudflare Dashboard 添加域名
2. 修改 `wrangler.toml`：

```toml
[[routes]]
pattern = "your-domain.com/*"
zone_name = "your-domain.com"
```

3. 重新部署：

```bash
wrangler deploy
```

### 自定义配置模板

1. 在 `templates/` 目录添加新的 JSON 模板
2. 修改 `src/config.rs` 添加模板加载逻辑
3. 重新构建和部署

## ⚠️ 限制

由于 Cloudflare Workers 的限制，请注意：

- **CPU 时间**: 最多 50ms (付费版)，10ms (免费版)
- **内存**: 128MB
- **请求体积**: 最大 100MB
- **响应体积**: 建议 < 10MB

对于大量节点的订阅，建议使用过滤参数减少节点数量。

## 🆚 与原版对比

| 特性 | Python 版 (Vercel) | Rust 版 (Cloudflare) |
|------|-------------------|----------------------|
| 部署平台 | Vercel Serverless | Cloudflare Workers |
| 运行时 | Python + Flask | Rust + WebAssembly |
| 冷启动 | ~500ms | ~5ms |
| 全球节点 | 有限 | 200+ 边缘节点 |
| 免费额度 | 100GB/月流量 | 100,000 次/天请求 |
| 自定义域名 | ✅ | ✅ |
| Web UI | ✅ | ❌ (仅 API) |

## 🐛 故障排除

### 构建失败

```bash
# 清理并重新构建
cargo clean
rm -rf build/
worker-build --release
```

### 部署失败

```bash
# 检查 wrangler 登录状态
wrangler whoami

# 重新登录
wrangler login

# 查看详细错误
wrangler deploy --verbose
```

### 运行时错误

查看 Cloudflare Dashboard 的 Workers 日志：
1. 进入 Workers & Pages
2. 选择你的 Worker
3. 查看 Logs 标签

## 📚 开发

### 本地测试

```bash
wrangler dev
```

访问 `http://localhost:8787`

### 查看日志

```bash
wrangler tail
```

### 运行测试

```bash
cargo test
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

基于原项目 [Toperlock/sing-box-subscribe](https://github.com/Toperlock/sing-box-subscribe) 修改

## 🔗 相关链接

- [原版 Python 项目](https://github.com/Toperlock/sing-box-subscribe)
- [Cloudflare Workers 文档](https://developers.cloudflare.com/workers/)
- [worker-rs 文档](https://github.com/cloudflare/workers-rs)
- [sing-box 官方文档](https://sing-box.sagernet.org/)

## ⭐ Star History

如果这个项目对你有帮助，请给个 Star ⭐

---

**免责声明**: 本项目仅供学习交流使用，请遵守当地法律法规。
