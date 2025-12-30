# 部署指南 / Deployment Guide

## 方式一：Cloudflare Workers (推荐)

### 步骤 1: 准备环境

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

### 步骤 2: 登录 Cloudflare

```bash
wrangler login
```

浏览器会打开 Cloudflare 授权页面，点击"允许"。

### 步骤 3: 修改配置

编辑 `wrangler.toml`:

```toml
name = "sing-box-worker"  # 修改为你的 Worker 名称
main = "build/worker/shim.mjs"
compatibility_date = "2024-01-01"

[build]
command = "cargo install -q worker-build && worker-build --release"

# 如果要使用自定义域名，取消注释并修改
# [[routes]]
# pattern = "your-domain.com/*"
# zone_name = "your-domain.com"
```

### 步骤 4: 构建项目

```bash
# 安装 worker-build
cargo install worker-build

# 构建 WebAssembly
worker-build --release
```

### 步骤 5: 部署

```bash
wrangler deploy
```

成功后会显示：
```
✅ Deployed sing-box-worker
   https://sing-box-worker.your-subdomain.workers.dev
```

### 步骤 6: 测试

```bash
curl "https://sing-box-worker.your-subdomain.workers.dev/config/你的订阅链接?emoji=1"
```

## 方式二：Cloudflare Pages (可选)

Pages 提供更长的 CPU 时间限制，适合处理大量节点的订阅。

### 步骤 1: 创建 GitHub 仓库

1. 在 GitHub 创建新仓库
2. 推送代码：

```bash
git init
git add .
git commit -m "Initial commit"
git remote add origin https://github.com/your-username/sing-box-worker.git
git push -u origin main
```

### 步骤 2: 连接到 Cloudflare Pages

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com)
2. 进入 "Workers & Pages"
3. 点击 "Create application" → "Pages" → "Connect to Git"
4. 选择你的 GitHub 仓库
5. 配置构建设置：
   - **Build command**: `cargo install worker-build && worker-build --release`
   - **Build output directory**: `build/worker`
   - **Root directory**: `/`

### 步骤 3: 部署

点击 "Save and Deploy"，Cloudflare 会自动构建和部署。

## 自定义域名配置

### 在 Cloudflare Workers 中使用

1. 在 Cloudflare Dashboard 添加你的域名
2. 等待 DNS 生效
3. 修改 `wrangler.toml`:

```toml
[[routes]]
pattern = "api.your-domain.com/*"
zone_name = "your-domain.com"
```

4. 重新部署：

```bash
wrangler deploy
```

### 在 Cloudflare Pages 中使用

1. 进入你的 Pages 项目
2. 点击 "Custom domains"
3. 添加你的域名
4. Cloudflare 会自动配置 DNS

## 环境变量 (可选)

如果需要设置环境变量（如 API 密钥）：

### Workers

```bash
wrangler secret put SECRET_KEY
```

### Pages

在 Pages 项目的 "Settings" → "Environment variables" 中添加。

## 性能优化

### 1. 优化编译体积

在 `Cargo.toml` 中：

```toml
[profile.release]
opt-level = "z"        # 优化体积
lto = true             # 链接时优化
codegen-units = 1      # 更好的优化
strip = true           # 移除符号
```

### 2. 启用缓存

在 Worker 中添加 Cache API：

```rust
// 缓存订阅内容 5 分钟
let cache = Cache::default();
let cached = cache.get(&request, false).await?;
```

### 3. 限制节点数量

对于大量节点的订阅，建议：
- 使用 `enn` 参数排除不需要的节点
- 只选择特定地区的节点
- 使用多个 Worker 分流

## 故障排查

### 构建失败

```bash
# 清理缓存
cargo clean
rm -rf build/

# 更新依赖
cargo update

# 重新构建
worker-build --release
```

### 部署失败

```bash
# 检查登录状态
wrangler whoami

# 查看详细日志
wrangler deploy --verbose
```

### 运行时错误

```bash
# 实时查看日志
wrangler tail

# 在 Cloudflare Dashboard 查看
# Workers & Pages → 你的 Worker → Logs
```

### CPU 超时

如果遇到 "CPU time limit exceeded":

1. 减少节点数量（使用过滤参数）
2. 简化配置模板
3. 升级到 Cloudflare Workers 付费版（50ms CPU 时间）
4. 考虑使用 Pages Functions（更长的执行时间）

## 监控和日志

### 查看实时日志

```bash
wrangler tail
```

### 查看分析数据

在 Cloudflare Dashboard:
1. 进入你的 Worker
2. 查看 "Metrics" 标签
3. 监控请求数、错误率、CPU 使用率

## 更新部署

```bash
# 拉取最新代码
git pull

# 重新构建
worker-build --release

# 部署
wrangler deploy
```

## 回滚版本

```bash
# 查看部署历史
wrangler deployments list

# 回滚到指定版本
wrangler rollback [deployment-id]
```

## 成本估算

### Cloudflare Workers 免费版
- 每天 100,000 次请求
- 每次请求 10ms CPU 时间
- 适合个人使用

### Cloudflare Workers 付费版 ($5/月)
- 每月 1000 万次请求（超出 $0.50/百万）
- 每次请求 50ms CPU 时间
- 适合中小型服务

### Cloudflare Pages (免费)
- 无限次请求
- 更长的 CPU 时间
- 适合静态网站 + API

## 安全建议

1. **启用访问控制**
   - 使用 Worker 内置的认证
   - 添加 IP 白名单
   - 使用 Cloudflare Access

2. **限流保护**
   ```rust
   // 添加速率限制
   if request_count > 100 {
       return Response::error("Rate limit exceeded", 429);
   }
   ```

3. **HTTPS Only**
   - Cloudflare 默认强制 HTTPS
   - 确保订阅链接也是 HTTPS

## 需要帮助？

- GitHub Issues: [提交问题](https://github.com/your-repo/issues)
- Cloudflare 社区: [community.cloudflare.com](https://community.cloudflare.com)
- Rust Workers 文档: [workers.cloudflare.com](https://workers.cloudflare.com)
