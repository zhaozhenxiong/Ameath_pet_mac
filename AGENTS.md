# Aemeath Codex Pet

Q 版像素爱弥斯桌宠。Codex 是默认客户端，也支持本地 Claude Code 与 Hermes Agent。

## 安装

1. 使用 `prebuilt/Aemeath Codex Pet.app`（Apple Silicon），或构建并手动启动 `aemeath-codex`（桌宠显示在桌面并创建托盘图标）
2. 将 [docs/hooks.json](docs/hooks.json) 合并到 `~/.codex/hooks.json`，替换可执行文件路径
3. 将 [docs/config.toml](docs/config.toml) 的 MCP 配置合并到 `~/.codex/config.toml`
4. Claude Code 可合并 [docs/claude-hooks.json](docs/claude-hooks.json)
5. Hermes Agent 可安装 [integrations/hermes/aemeath-pet](integrations/hermes/aemeath-pet) 插件
6. 重新打开客户端或新建会话

hooks 仅向正在运行的桌宠发送事件；桌宠退出后不会自动重启，需要手动打开。

## 端口

- HTTP: 127.0.0.1:9527
- MCP: 127.0.0.1:9528

## 构建与验证

```bash
npm ci
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo build --locked --manifest-path src-tauri/Cargo.toml --release
```

产出位于 `src-tauri/target/release/`。

## 前置要求

- Rust stable
- Node.js >= 18
- Windows 10+ 或当前 Tauri 2 支持的 macOS 版本

## 目录结构

```text
aemeath-codex/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs       # Tauri、HTTP 与 MCP 启动入口
│       ├── codex.rs      # Codex、Claude、Hermes 命令行事件桥接
│       ├── state.rs      # 状态机、动画与气泡映射
│       ├── http.rs       # hook/API 服务 (:9527)
│       ├── mcp.rs        # MCP JSON-RPC 服务 (:9528)
│       └── tray.rs       # 系统托盘
├── src/                  # 桌宠渲染前端与像素素材
├── docs/
│   ├── hooks.json        # 全局 Codex hooks 模板
│   ├── claude-hooks.json # Claude Code hooks 模板
│   └── config.toml       # 全局 Codex MCP 模板
├── integrations/
│   └── hermes/aemeath-pet # Hermes Agent 生命周期插件
└── package.json
```
