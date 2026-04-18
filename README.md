# AI Sound Notify

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A lightweight sound notification system that plays distinct audio cues in your browser when AI coding agents (Claude Code, Gemini CLI, Codex CLI) complete tasks or need user input.

## Features

- **Multi-agent support** - Works with Claude Code, Gemini CLI, and Codex CLI
- **Distinct sounds** - Each AI source has a unique frequency so you can tell them apart by ear
- **Two event types** - Different sound patterns for "task complete" (rising tone) and "needs input" (double beep)
- **Browser-based** - No desktop app required; runs in any modern browser tab
- **Real-time** - WebSocket push delivers notifications instantly
- **Zero audio files** - All sounds synthesized via Web Audio API
- **Remote-friendly** - Perfect for managing multiple SSH sessions (e.g., via Nexterm)
- **Volume control** - Adjustable volume and per-source mute toggles in the UI
- **Notification history** - Keeps up to 50 recent notifications visible on the page

## Install as Claude Code Plugin (Recommended)

The easiest way to use AI Sound Notify with Claude Code is via the plugin:

```bash
# 1. Add the plugin marketplace
/plugin marketplace add yogiant333/ai-sound-notify

# 2. Install the plugin (via /plugin UI, go to Discover tab)
/plugin install ai-sound-notify@yogiant333-ai-sound-notify

# 3. Configure your server URL
/sound-notify-config
```

The plugin automatically registers hooks for `Stop` and `Notification` events. You just need to tell it your server URL (e.g., `http://localhost:9800` or `https://your-domain.com`).

The server URL is stored in `~/.claude/ai-sound-notify.local.md` and can be changed anytime by running `/sound-notify-config` again.

> **Note:** You still need to deploy the notification server separately (see [Quick Start](#quick-start) below).

## Windows Desktop Client

If your browser tab keeps going to sleep and swallowing notifications, install the Windows desktop client. It runs in the system tray, plays sounds without any autoplay restriction, can pick local audio files for each event, and raises a distinct alarm when the notification server becomes unreachable.

**Download:** Grab `AI Sound Notify_<version>_x64-setup.exe` from the latest release.

**Features:**

- ~5 MB installer, ~40 MB RAM idle
- System tray, close-to-tray, single instance, optional auto-start with Windows
- Native Windows toast notifications
- Server offline alarm (3 consecutive health-check failures over 45 s)
- Works against the public server at `https://ainotify.keymantek.com:777` out of the box, or any other deployment
- Per-source x event custom sound picker (browse any `.wav` / `.mp3` / `.ogg` / `.flac` / `.m4a`)

**Build from source (Windows only):**

```powershell
winget install Rustlang.Rustup
rustup default stable-msvc
winget install Microsoft.VisualStudio.2022.BuildTools   # select "Desktop development with C++"
cd desktop
npm install
npm run build
```

Output: `desktop/src-tauri/target/release/bundle/nsis/AI Sound Notify_*.exe`

## Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/yogiant333/ai-sound-notify.git
cd ai-sound-notify

# 2. Install dependencies
cd server && npm install

# 3. Start the server
npm start
```

Open your browser to **http://localhost:9800** and click anywhere on the page to enable audio (required by browser autoplay policy).

Then configure your AI tool's hooks (see [Configuration Guide](#configuration-guide) below).

## How It Works

```
Claude Code / Gemini CLI / Codex CLI
         | curl HTTP POST /notify
         v
  +------------------------------+
  | Node.js Server (port 9800)   |
  | - Express  (HTTP API)        |
  | - WebSocket (ws)             |
  +------------------------------+
         | WebSocket push
         v
  +------------------------------+
  | Browser Notification Page    |
  | - Web Audio API sounds       |
  | - 6 distinct sound combos    |
  | - Notification history       |
  +------------------------------+
```

1. Your AI tool finishes a task (or needs input) and fires its hook.
2. The hook runs a `curl` command that POSTs a JSON payload to the server.
3. The server broadcasts the payload to every connected browser tab via WebSocket.
4. The browser plays a synthesized sound and logs the notification.

## Configuration Guide

### Claude Code

Merge the following into **`~/.claude/settings.json`** (create the file if it does not exist).

If the file already has a `hooks` key, merge the `Stop` and `Notification` arrays into it.

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"claude-code\",\"event\":\"task_complete\"}'",
            "timeout": 5
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "idle_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"claude-code\",\"event\":\"need_input\",\"message\":\"Claude is idle, needs your input\"}'",
            "timeout": 5
          }
        ]
      },
      {
        "matcher": "permission_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"claude-code\",\"event\":\"need_input\",\"message\":\"Claude needs permission\"}'",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

- **Stop** fires when Claude Code finishes its turn -> `task_complete`
- **Notification (idle_prompt)** fires when Claude is waiting for input -> `need_input`
- **Notification (permission_prompt)** fires when Claude needs tool approval -> `need_input`

### Gemini CLI

Merge the following into **`~/.gemini/settings.json`**:

```json
{
  "hooks": {
    "AfterAgent": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"gemini\",\"event\":\"task_complete\"}'",
            "timeout": 5000
          }
        ]
      }
    ],
    "Notification": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"gemini\",\"event\":\"need_input\"}'",
            "timeout": 5000
          }
        ]
      }
    ]
  }
}
```

- **AfterAgent** fires after Gemini completes its agent turn -> `task_complete`
- **Notification** fires when Gemini needs user attention -> `need_input`

> **Note:** Gemini CLI uses milliseconds for timeout (5000 = 5 seconds).

### Codex CLI

Add the following to **`~/.codex/config.toml`**:

```toml
notify = ["bash", "-c", "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"codex\",\"event\":\"task_complete\"}'"]

[tui]
notifications = true
notification_method = "bel"
```

- **notify** fires on `agent-turn-complete` -> sends `task_complete`
- **approval-requested** events are TUI-only (terminal bell); there is no external hook support for this event yet

### Remote Server

If the notification server is running on a remote machine instead of localhost, replace `localhost:9800` with the server's actual IP address and port in **all** curl commands above.

For example, if your server is at `192.168.1.100`:

```
http://localhost:9800/notify  ->  http://192.168.1.100:9800/notify
```

Make sure port 9800 is accessible from the machine where your AI tool is running (check firewall rules).

## API Reference

### POST /notify

Send a notification to all connected browser clients.

**Request body** (JSON):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `source` | string | Yes | AI tool identifier. One of: `claude-code`, `gemini`, `codex` |
| `event` | string | Yes | Event type. One of: `task_complete`, `need_input` |
| `message` | string | No | Optional human-readable description |
| `session_id` | string | No | Optional session identifier for filtering |

**Example request:**

```bash
curl -X POST http://localhost:9800/notify \
  -H 'Content-Type: application/json' \
  -d '{"source":"claude-code","event":"task_complete","message":"Build succeeded"}'
```

**Success response** (`200`):

```json
{
  "ok": true,
  "notification": {
    "source": "claude-code",
    "event": "task_complete",
    "message": "Build succeeded",
    "session_id": null,
    "timestamp": "2026-03-05T12:00:00.000Z"
  }
}
```

**Error response** (`400`):

```json
{
  "error": "Invalid source. Must be one of: claude-code, gemini, codex"
}
```

### GET /api/health

Health check endpoint.

**Response** (`200`):

```json
{
  "status": "ok"
}
```

## Sound Reference

All sounds are generated with the Web Audio API using sine wave oscillators. No audio files are needed.

| Source | Event | Frequency | Pattern |
|--------|-------|-----------|---------|
| Claude Code | task_complete | 880 Hz (high) | Single rising tone (880 -> 1100 Hz, 0.3s) |
| Claude Code | need_input | 880 Hz (high) | Double beep (2 x 0.15s with 0.1s gap) |
| Gemini | task_complete | 660 Hz (mid) | Single rising tone (660 -> 825 Hz, 0.3s) |
| Gemini | need_input | 660 Hz (mid) | Double beep (2 x 0.15s with 0.1s gap) |
| Codex | task_complete | 440 Hz (low) | Single rising tone (440 -> 550 Hz, 0.3s) |
| Codex | need_input | 440 Hz (low) | Double beep (2 x 0.15s with 0.1s gap) |

- **task_complete**: A single sine wave that rises 25% in pitch over 0.3 seconds, then fades out.
- **need_input**: Two short sine wave beeps (0.15s each) separated by a 0.1s gap -- sounds more urgent.

## Customization

### Changing the Port

Set the `PORT` environment variable before starting the server:

```bash
PORT=3000 npm start
```

Remember to update the port number in all your hook configurations accordingly.

### How Sounds Work

The browser page uses the [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API) to synthesize sounds on the fly. No audio files are downloaded or bundled. You can modify the frequencies and patterns by editing the `FREQ` map and `playTaskComplete` / `playNeedInput` functions in `web/index.html`.

### Volume and Mute

The web UI provides:
- A **volume slider** in the header to adjust sound level
- A **Mute All** button to silence all notifications
- Per-source **toggle switches** to enable/disable individual AI sources

## Troubleshooting

### Browser autoplay policy

Modern browsers block audio playback until the user interacts with the page. **Click anywhere on the notification page** after opening it to unlock audio. The page initializes the Audio Context lazily on first user interaction.

### WebSocket connection shows "Disconnected"

- Make sure the server is running (`npm start` in the `server/` directory).
- Check that you are opening the browser at the correct address (default: `http://localhost:9800`).
- The page automatically reconnects every 3 seconds if the connection drops.

### No sound when notification arrives

1. Check the browser console for errors.
2. Make sure the **Mute All** button is not active (red).
3. Make sure the toggle for the relevant source is enabled (turned on).
4. Verify your system volume is not muted.
5. Try clicking one of the **Test** buttons on the page first to confirm audio works.

### CORS issues

The server sets `Access-Control-Allow-Origin: *` by default, so cross-origin requests should work. If you place the server behind a reverse proxy, make sure the proxy forwards the CORS headers and supports WebSocket upgrades.

### curl command fails in hooks

- Verify the server is reachable from the machine running the AI tool: `curl http://localhost:9800/api/health`
- For remote servers, ensure the firewall allows traffic on port 9800.
- Check that `curl` is installed on the machine.

## License

[MIT](LICENSE)

---

# 中文说明

## 项目简介

AI Sound Notify 是一个轻量级的声音通知系统。当 AI 编程助手（Claude Code、Gemini CLI、Codex CLI）完成任务或需要用户输入时，系统会在浏览器中播放不同的提示音。

本项目专为同时管理多个远程 SSH 会话的用户设计（例如通过 Nexterm 等工具），让你不用盯着终端也能知道 AI 的工作状态。

**主要特点：**

- 支持三种 AI 工具：Claude Code、Gemini CLI、Codex CLI
- 每个 AI 工具有独特的音调频率，靠听就能分辨来源
- 两种事件类型：任务完成（上升音）和需要输入（双响急促音）
- 纯浏览器运行，无需安装桌面应用
- 通过 Web Audio API 合成声音，无需音频文件
- WebSocket 实时推送，通知即时到达

## 通过 Claude Code 插件安装（推荐）

最简单的 Claude Code 集成方式是安装插件：

```bash
# 1. 添加插件市场源
/plugin marketplace add yogiant333/ai-sound-notify

# 2. 安装插件（通过 /plugin 界面，进入 Discover 标签页）
/plugin install ai-sound-notify@yogiant333-ai-sound-notify

# 3. 配置服务器地址
/sound-notify-config
```

插件会自动注册 `Stop` 和 `Notification` 事件的 hooks。你只需输入服务器地址（如 `http://localhost:9800` 或 `https://your-domain.com`）。

服务器地址保存在 `~/.claude/ai-sound-notify.local.md` 中，随时可通过 `/sound-notify-config` 修改。

> **注意：** 通知服务器仍需单独部署（见下方快速开始）。

## Windows 桌面客户端

浏览器标签页休眠收不到声音？装这个常驻后台的桌面版。它会托盘化运行，没有浏览器 autoplay 限制，可以给每个事件单独挑本机音频文件，而且当服务器连不上时会发出警报声。

**下载：** 从最新 Release 下载 `AI Sound Notify_<版本号>_x64-setup.exe`。

**功能：**

- ~5MB 安装包，空闲内存 ~40MB
- 系统托盘、关闭即最小化到托盘、单实例运行、可选开机自启
- 原生 Windows 通知气泡
- 服务器离线警报（45 秒内连续 3 次健康检查失败触发）
- 开箱默认连接 `https://ainotify.keymantek.com:777`，也可在设置里改成你自己的部署地址
- 每个"来源 × 事件"组合可以单独挑一个本机音频文件（支持 .wav / .mp3 / .ogg / .flac / .m4a）

**从源码构建（仅 Windows）：**

```powershell
winget install Rustlang.Rustup
rustup default stable-msvc
winget install Microsoft.VisualStudio.2022.BuildTools   # 勾选 "Desktop development with C++"
cd desktop
npm install
npm run build
```

产出：`desktop/src-tauri/target/release/bundle/nsis/AI Sound Notify_*.exe`

## 快速开始

```bash
# 1. 克隆仓库
git clone https://github.com/yogiant333/ai-sound-notify.git
cd ai-sound-notify

# 2. 安装依赖
cd server && npm install

# 3. 启动服务器
npm start
```

在浏览器中打开 **http://localhost:9800**，然后点击页面任意位置以启用音频（浏览器自动播放策略要求）。

## 配置指南

### Claude Code

将以下内容合并到 **`~/.claude/settings.json`** 文件中（如果文件不存在则新建）：

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"claude-code\",\"event\":\"task_complete\"}'",
            "timeout": 5
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "idle_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"claude-code\",\"event\":\"need_input\",\"message\":\"Claude is idle, needs your input\"}'",
            "timeout": 5
          }
        ]
      },
      {
        "matcher": "permission_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"claude-code\",\"event\":\"need_input\",\"message\":\"Claude needs permission\"}'",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

### Gemini CLI

将以下内容合并到 **`~/.gemini/settings.json`** 文件中：

```json
{
  "hooks": {
    "AfterAgent": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"gemini\",\"event\":\"task_complete\"}'",
            "timeout": 5000
          }
        ]
      }
    ],
    "Notification": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"gemini\",\"event\":\"need_input\"}'",
            "timeout": 5000
          }
        ]
      }
    ]
  }
}
```

> **注意：** Gemini CLI 的 timeout 单位是毫秒（5000 = 5 秒）。

### Codex CLI

将以下内容添加到 **`~/.codex/config.toml`** 文件中：

```toml
notify = ["bash", "-c", "curl -s -X POST http://localhost:9800/notify -H 'Content-Type: application/json' -d '{\"source\":\"codex\",\"event\":\"task_complete\"}'"]

[tui]
notifications = true
notification_method = "bel"
```

> **注意：** Codex CLI 目前仅支持通过外部命令发送 `task_complete` 通知。`approval-requested` 事件仅在 TUI 中通过终端响铃提醒，暂不支持外部 Hook。

### 远程服务器配置

如果通知服务器运行在远程机器上，需要将上述所有 curl 命令中的 `localhost:9800` 替换为服务器的实际 IP 地址和端口。

例如，服务器 IP 为 `192.168.1.100`：

```
http://localhost:9800/notify  ->  http://192.168.1.100:9800/notify
```

请确保端口 9800 在防火墙中已开放，并且 AI 工具所在的机器可以访问该地址。

## API 参考

### POST /notify

向所有已连接的浏览器客户端发送通知。

**请求体** (JSON)：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `source` | string | 是 | AI 工具标识。可选值：`claude-code`、`gemini`、`codex` |
| `event` | string | 是 | 事件类型。可选值：`task_complete`、`need_input` |
| `message` | string | 否 | 可选的描述信息 |
| `session_id` | string | 否 | 可选的会话标识符 |

**请求示例：**

```bash
curl -X POST http://localhost:9800/notify \
  -H 'Content-Type: application/json' \
  -d '{"source":"claude-code","event":"task_complete","message":"构建成功"}'
```

### GET /api/health

健康检查端点。返回 `{"status": "ok"}`。

## 常见问题

### 浏览器没有声音

现代浏览器要求用户与页面进行交互后才允许播放音频。打开通知页面后，请**点击页面任意位置**以解锁音频播放。

### WebSocket 显示"Disconnected"

- 确认服务器正在运行（在 `server/` 目录下执行 `npm start`）
- 确认浏览器访问的地址正确（默认：`http://localhost:9800`）
- 页面会每 3 秒自动重连

### 收到通知但没有声音

1. 检查浏览器控制台是否有错误
2. 确认页面上的 "Mute All" 按钮未激活
3. 确认对应来源的开关已打开
4. 确认系统音量未静音
5. 先点击页面上的 "Test" 按钮测试音频是否正常

### curl 命令在 Hook 中执行失败

- 检查服务器是否可达：`curl http://localhost:9800/api/health`
- 远程服务器请确认防火墙已开放端口 9800
- 确认机器上已安装 `curl`

## 许可证

[MIT](LICENSE)
