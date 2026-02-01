# sing-box Subscription Converter - Rust + Cloudflare Workers Edition

这是原 Python 版本 sing-box-subscribe 的 Rust + Cloudflare Workers 移植版本，提供高性能的订阅转换服务。

## 特性

- **高性能** - Rust 编译为 WebAssembly，运行在 Cloudflare 边缘网络
- **全球部署** - 自动部署到全球 200+ 数据中心
- **免费额度** - Cloudflare Workers 每天 100,000 次免费请求
- **安全可靠** - 边缘计算，无需自建服务器
- **即时响应** - 边缘节点就近响应，延迟极低

## 支持的协议

- VMess
- VLESS (包括 REALITY)
- Trojan
- Shadowsocks
- Hysteria2
- TUIC

## 快速开始

### 1. 前置要求

- [Rust](https://rustup.rs/) (最新稳定版)
- [Node.js](https://nodejs.org/) (v16+)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/)
- Cloudflare 账号

### 2. 准备环境

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Node.js (如果没有)
# macOS
brew install node
# Ubuntu/Debian
sudo apt install nodejs npm

# 安装 Wrangler CLI
npm install -g wrangler
```

### 3. 登录 Cloudflare

```bash
wrangler login
```

浏览器会打开 Cloudflare 授权页面，点击"允许"。

### 4. 克隆并构建项目

```bash
cd sing-box-worker
cargo install worker-build
worker-build --release
```

### 5. 部署到 Cloudflare

```bash
wrangler deploy
```

成功后会显示：
```
Deployed sing-box-worker
https://sing-box-worker.your-subdomain.workers.dev
```

### 6. 测试

```bash
curl "https://sing-box-worker.your-subdomain.workers.dev/sub?urls=https://example.com/subscribe?token=abc123&emoji=1"
```

## 使用方法

### 方式一：Web UI（推荐）

访问 Worker 根路径，使用交互式界面：

```
https://your-worker.workers.dev/
```

支持：
- 在线输入多个订阅地址
- 自定义模板 URL
- 节点前缀和 Emoji 选项
- 一键转换并下载配置
- 复制 API 链接

### 方式二：API 调用

#### 基本用法

```
https://your-worker.workers.dev/sub?urls=<订阅链接>
```

#### URL 参数

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `urls` | **必需** | 订阅链接，多个用 `\|` 分隔 | `urls=https://sub1.com\|https://sub2.com` |
| `config` | 可选 | 自定义远程模板 URL | `config=https://example.com/config.json` |
| `emoji` | 可选 | 添加国旗 emoji (1=开启, 0=关闭) | `emoji=1` |
| `prefix` | 可选 | 节点名称前缀 | `prefix=MyVPN` |
| `file` | 可选 | 配置模板索引 (0=基础模板) | `file=0` |
| `ua` 或 `UA` | 可选 | 自定义 User-Agent | `ua=v2rayng` |
| `enn` | 可选 | 排除节点名称关键词 (支持正则) | `enn=过期\|到期` |

#### 使用示例

**单个订阅 + 内置模板**
```
https://your-worker.workers.dev/sub?urls=https://example.com/subscribe?token=abc123&emoji=1
```

**多个订阅聚合**
```
https://your-worker.workers.dev/sub?urls=https://sub1.com/api?token=xxx|https://sub2.com/api?token=yyy&emoji=1
```

**使用自定义远程模板**
```
https://your-worker.workers.dev/sub?urls=https://example.com/subscribe?token=abc123&config=https://example.com/template.json&emoji=1
```

**添加节点前缀和排除规则**
```
https://your-worker.workers.dev/sub?urls=https://example.com/subscribe?token=abc123&prefix=HK&enn=过期|到期&emoji=1
```

## 部署方式

### 方式一：Cloudflare Workers（推荐）

即上文"快速开始"中的方式。修改 `wrangler.toml` 自定义 Worker 名称：

```toml
name = "sing-box-worker"  # 修改为你的 Worker 名称
main = "build/worker/shim.mjs"
compatibility_date = "2024-01-01"

[build]
command = "cargo install -q worker-build && worker-build --release"
```

### 方式二：Cloudflare Pages

Pages 提供更长的 CPU 时间限制，适合处理大量节点的订阅。

1. 在 GitHub 创建新仓库并推送代码：

```bash
git init
git add .
git commit -m "Initial commit"
git remote add origin https://github.com/your-username/sing-box-worker.git
git push -u origin main
```

2. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com)，进入 "Workers & Pages"
3. 点击 "Create application" → "Pages" → "Connect to Git"
4. 选择你的 GitHub 仓库
5. 配置构建设置：
   - **Build command**: `cargo install worker-build && worker-build --release`
   - **Build output directory**: `build/worker`
   - **Root directory**: `/`
6. 点击 "Save and Deploy"

## 配置

### 自定义域名

#### Workers

1. 在 Cloudflare Dashboard 添加域名，等待 DNS 生效
2. 修改 `wrangler.toml`：

```toml
[[routes]]
pattern = "api.your-domain.com/*"
zone_name = "your-domain.com"
```

3. 重新部署：

```bash
wrangler deploy
```

#### Pages

1. 进入你的 Pages 项目
2. 点击 "Custom domains"
3. 添加你的域名，Cloudflare 会自动配置 DNS

### 环境变量

如果需要设置环境变量（如 API 密钥）：

- **Workers**: `wrangler secret put SECRET_KEY`
- **Pages**: 在 Pages 项目的 "Settings" → "Environment variables" 中添加

### 自定义配置模板

#### 方式一：使用远程模板（推荐）

通过 `config` 参数直接指定远程模板 URL，无需修改代码：

```
https://your-worker.workers.dev/sub?urls=https://example.com/subscribe&config=https://example.com/my-template.json
```

#### 方式二：修改内置模板

1. 编辑 `templates/basic.json`
2. 重新构建和部署

### 模板高级语法

#### `{all}` 占位符

自动替换为所有节点标签：

```json
{
  "tag": "auto-select",
  "type": "urltest",
  "outbounds": ["{all}"]
}
```

转换后：
```json
{
  "tag": "auto-select",
  "type": "urltest",
  "outbounds": ["node1", "node2", "node3"]
}
```

#### Filter 过滤规则

使用 `filter` 字段按关键词过滤节点：

```json
{
  "tag": "US-nodes",
  "type": "selector",
  "outbounds": ["{all}"],
  "filter": [
    {
      "action": "include",
      "keywords": ["🇺🇸|US|美国|United States"]
    },
    {
      "action": "exclude",
      "keywords": ["频道|订阅|过期"]
    }
  ]
}
```

- `include`: 仅包含匹配的节点（支持正则）
- `exclude`: 排除匹配的节点（支持正则）
- 多个关键词用 `|` 分隔
- `filter` 字段会自动从最终输出中移除

#### 自动清理

如果某个 outbound 过滤后为空，会自动：
1. 删除该 outbound
2. 从其他 outbound 的引用中移除

## 限制

由于 Cloudflare Workers 的限制，请注意：

| 项目 | 免费版 | 付费版 ($5/月) |
|------|--------|----------------|
| CPU 时间 | 10ms | 50ms |
| 内存 | 128MB | 128MB |
| 每日请求 | 100,000 次 | 1000 万次/月 |
| 响应体积 | 建议 < 10MB | 建议 < 10MB |

对于大量节点的订阅，建议使用过滤参数减少节点数量。

## 性能优化

### 编译体积优化

`Cargo.toml` 已配置：

```toml
[profile.release]
opt-level = "z"        # 优化体积
lto = true             # 链接时优化
codegen-units = 1      # 更好的优化
strip = true           # 移除符号
```

### 减少节点数量

对于大量节点的订阅，建议：
- 使用 `enn` 参数排除不需要的节点
- 只选择特定地区的节点
- 使用多个 Worker 分流

## 与原版对比

| 特性 | Python 版 (Vercel) | Rust 版 (Cloudflare) |
|------|-------------------|----------------------|
| 部署平台 | Vercel Serverless | Cloudflare Workers |
| 运行时 | Python + Flask | Rust + WebAssembly |
| 冷启动 | ~500ms | ~5ms |
| 全球节点 | 有限 | 200+ 边缘节点 |
| 免费额度 | 100GB/月流量 | 100,000 次/天请求 |
| 自定义域名 | 支持 | 支持 |
| Web UI | 支持 | 支持 |
| 远程模板 | 不支持 | 支持 |

## 开发

### 本地测试

```bash
wrangler dev
```

访问 Web UI：`http://localhost:8787`

测试 API：
```bash
# 单个订阅
curl "http://localhost:8787/sub?urls=https://example.com/subscribe?token=abc123&emoji=1"

# 多个订阅
curl "http://localhost:8787/sub?urls=https://sub1.com/api|https://sub2.com/api&emoji=1"

# 使用自定义模板
curl "http://localhost:8787/sub?urls=https://example.com/subscribe&config=https://example.com/template.json"
```

### 查看日志

```bash
wrangler tail
```

### 运行测试

```bash
cargo test
```

## 故障排查

### 构建失败

```bash
# 清理并重新构建
cargo clean
rm -rf build/
worker-build --release

# 更新依赖
cargo update
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

```bash
# 实时查看日志
wrangler tail
```

也可以在 Cloudflare Dashboard 查看：Workers & Pages → 你的 Worker → Logs

### CPU 超时

如果遇到 "CPU time limit exceeded":

1. 减少节点数量（使用过滤参数）
2. 简化配置模板
3. 升级到 Cloudflare Workers 付费版（50ms CPU 时间）
4. 考虑使用 Pages Functions（更长的执行时间）

## 更新和回滚

```bash
# 更新部署
git pull
worker-build --release
wrangler deploy

# 查看部署历史
wrangler deployments list

# 回滚到指定版本
wrangler rollback [deployment-id]
```

## 监控

在 Cloudflare Dashboard:
1. 进入你的 Worker
2. 查看 "Metrics" 标签
3. 监控请求数、错误率、CPU 使用率

实时日志：
```bash
wrangler tail
```

## 相关链接

- [原版 Python 项目](https://github.com/Toperlock/sing-box-subscribe)
- [Cloudflare Workers 文档](https://developers.cloudflare.com/workers/)
- [worker-rs 文档](https://github.com/cloudflare/workers-rs)
- [sing-box 官方文档](https://sing-box.sagernet.org/)

## 许可证

基于原项目 [Toperlock/sing-box-subscribe](https://github.com/Toperlock/sing-box-subscribe) 修改

---

**免责声明**: 本项目仅供学习交流使用，请遵守当地法律法规。
