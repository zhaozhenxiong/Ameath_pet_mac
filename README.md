# Aemeath Codex Pet

Q 版像素爱弥斯桌宠。以 Codex 为默认客户端，同时支持本地 Claude Code 和 Hermes Agent。桌宠会随当前客户端的思考、工具调用、权限确认、子任务、上下文压缩和任务完成切换动画与气泡。

本仓库包含 macOS 版 `0.1.4` 的完整源码及预编译应用包。

## 功能

- 15 种像素动画状态，与 Codex 当前行为实时同步
- Codex 全局默认接入，支持完整生命周期 hooks
- 支持 Claude Code hooks 与 Hermes Agent 插件
- 桌宠运行时接收各客户端事件；退出后 hooks 不会自动重启桌宠
- 内置 Streamable HTTP MCP 服务，可显示自定义气泡和动画
- 透明、置顶、可拖拽窗口，支持系统托盘显示/隐藏
- 本地回环通信，不上传会话或代码内容

| 客户端操作 | 桌宠反馈 |
| --- | --- |
| 提交消息 / 思考 | 组织回复、思考动画 |
| 执行命令 | 奔跑 +“正在执行命令...” |
| 修改文件 | 构建 +“正在写代码...” |
| 网络搜索 | 搜索动画 |
| 调度子任务 | 分析动画 |
| 等待授权 | 招手 +“等待指示...” |
| 完成任务 | 庆祝 +“太棒了!” |

## 安装

### macOS 预编译应用

`prebuilt/Aemeath Codex Pet.app` 为 Apple Silicon（arm64）版本，来自已安装的 `0.1.4` 应用包，可直接使用，无需安装 Rust 或 Node.js。Intel Mac 请从源码构建。

```bash
git clone https://github.com/zhaozhenxiong/Ameath_pet_mac.git
cd Ameath_pet_mac
mkdir -p "$HOME/Applications"
ditto "prebuilt/Aemeath Codex Pet.app" "$HOME/Applications/Aemeath Codex Pet.app"
open "$HOME/Applications/Aemeath Codex Pet.app"
```

此包使用临时签名（ad-hoc），没有 Developer ID 签名或公证。若 macOS 首次打开时阻止启动，可在系统设置的「隐私与安全性」中允许打开你已确认来源的应用。

随后按下方第 2–4 步接入客户端。配置模板使用 `/Applications/Aemeath Codex Pet.app/Contents/MacOS/aemeath-codex`；按以上方式安装时，请改为你的实际用户目录下的路径，例如 `/Users/your-name/Applications/Aemeath Codex Pet.app/Contents/MacOS/aemeath-codex`。

### 1. 构建桌宠

需要 Rust stable、Node.js 18+ 和系统对应的 Tauri 构建环境。

```bash
npm ci
npm run tauri build -- --bundles app
```

也可以只构建可执行文件：

```bash
cargo build --manifest-path src-tauri/Cargo.toml --release
```

macOS 应用包位于 `src-tauri/target/release/bundle/macos/Aemeath Codex Pet.app`。可执行文件位于 `src-tauri/target/release/aemeath-codex`；Windows 下文件名为 `aemeath-codex.exe`。`--bundles app` 用于 macOS，在其他系统构建时使用 `npm run tauri build`。

### 2. 将 Codex 设为默认客户端

将 [docs/hooks.json](docs/hooks.json) 复制为 `~/.codex/hooks.json`，并把其中的 `/Applications/Aemeath Codex Pet.app/Contents/MacOS/aemeath-codex` 全部替换成实际可执行文件路径。如果已有 hooks 文件，请合并各事件数组，不要覆盖原有 hook。

macOS 应用包内的可执行文件通常位于：

```text
/Applications/Aemeath Codex Pet.app/Contents/MacOS/aemeath-codex
```

将 [docs/config.toml](docs/config.toml) 中的 `mcp_servers.aemeath` 合并到 `~/.codex/config.toml`，或执行：

```bash
codex mcp add aemeath --url http://127.0.0.1:9528/mcp
```

这样配置后，所有 Codex 项目都会使用同一个桌宠，不需要为每个仓库重复设置。

### 3. 接入 Claude Code

把 [docs/claude-hooks.json](docs/claude-hooks.json) 的 `hooks` 合并到 `~/.claude/settings.json`，替换其中的可执行文件路径。再添加同一个 MCP 服务：

```bash
claude mcp add --transport http --scope user aemeath http://127.0.0.1:9528/mcp
```

### 4. 接入 Hermes Agent

将 [integrations/hermes/aemeath-pet](integrations/hermes/aemeath-pet) 复制到 `~/.hermes/plugins/aemeath-pet`，然后执行：

```bash
hermes plugins enable aemeath-pet
hermes mcp add aemeath --url http://127.0.0.1:9528/mcp
```

Hermes 插件负责生命周期动画，MCP 负责让 Hermes 主动控制气泡和动画。

### 5. 启动

先手动运行桌宠，随后重新打开 Codex，或在 Claude/Hermes 中开始新会话。桌宠未运行时，hooks 会忽略动画事件；退出后需要手动重新打开。

## 架构

```text
Codex（默认） ── hooks ──────────────┐
Claude Code ── hooks ───────────────┼→ aemeath-codex
Hermes Agent ── lifecycle plugin ───┘

Codex / Claude / Hermes
  └── MCP client → http://127.0.0.1:9528/mcp

Aemeath Codex Pet
  ├── HTTP :9527  → 多客户端事件 + 前端状态轮询
  ├── MCP  :9528  → tools / resources
  └── Tauri       → 透明桌宠窗口 + 托盘
```

## 本地端口

| 端口 | 用途 |
| --- | --- |
| 9527 | 客户端事件接收与桌宠状态 API |
| 9528 | Streamable HTTP MCP |

服务只监听 `127.0.0.1`。

## MCP 能力

- `aemeath_show`：显示自定义气泡
- `aemeath_ask`：用桌宠气泡展示问题
- `aemeath_play`：播放指定动画
- `aemeath://status`：读取当前状态
- `aemeath://history`：读取最近状态记录

## 开发

开发模式使用 `http://localhost:1420`。先在项目根目录的另一个终端启动静态服务器（需要 Python 3）：

```bash
python3 -m http.server 1420 --bind 127.0.0.1 --directory src
```

再在项目根目录执行：

```bash
npm run tauri dev
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

项目结构与验证要求见 [AGENTS.md](AGENTS.md)。

## License

本仓库沿用 [MIT](LICENSE)。原项目作者为 sinlatansen，其 MIT 许可完整保留于 [LICENSE.upstream](LICENSE.upstream)。源码与素材继承自本地 `aemeath_withclaude-in-claude` 项目；本仓库整理了 macOS 应用包与接入说明。
