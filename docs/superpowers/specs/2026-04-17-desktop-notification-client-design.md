# Desktop Notification Client — Design Spec

**Date:** 2026-04-17
**Status:** Draft — pending user approval
**Author:** yy + Claude

## Problem

The current web-based notification page lives in a browser tab. Modern browsers
aggressively throttle or suspend background tabs, so the user misses sound alerts
when the AI agent finishes a task. A dedicated desktop process on Windows that
always runs (tray + anti-suspend) solves the problem and also unlocks:

- Playing arbitrary user-selected audio files from the local filesystem
- No autoplay restriction, no tab-sleep throttling, no playback time limit
- Native Windows toast notifications
- Detection of "remote notification server is offline" as its own alarm

## Goals

1. Windows desktop client that receives WebSocket pushes from the existing
   notification server at `https://ainotify.keymantek.com:777` and plays sounds.
2. Lightweight: installer ≤ 15 MB, memory ≤ 50 MB at idle.
3. Reuse the existing web UI (`web/index.html`) with minimum changes.
4. Let the user pick a local audio file (`.wav` / `.mp3` / `.ogg`) for any
   source × event combination via a native file dialog.
5. Monitor the remote server; when it becomes unreachable, play a distinct alarm.
6. Auto-start with Windows; minimize to system tray; prevent app suspension.

## Non-Goals

- Rewriting the existing Node.js server (it stays untouched, auto-starts in WSL).
- Supporting macOS / Linux desktop binaries in v1.
- Custom sound authoring / waveform editing.
- Multi-user profile sync across machines.

## Architecture

Pure client. The Tauri app does not run an HTTP server. It connects outbound
(WSS) to the existing server and makes periodic HTTPS GETs for health.

```
Claude Code / Gemini / Codex (remote workstations or WSL)
    | HTTP POST (curl hooks — unchanged)
    v
Node.js server @ https://ainotify.keymantek.com:777  (existing, untouched)
    | WSS push
    v
+------------------------------------------------------+
| Tauri Desktop Client (Windows .exe)                  |
|                                                      |
|  Rust side                  |  Web side (WebView2)   |
|  ---------                  |  -------------------   |
|  - Window + tray            |  - Existing index.html |
|  - powerSaveBlocker         |  - WebSocket client    |
|  - Single-instance lock     |  - Audio playback      |
|  - Auto-start toggle        |    (Web Audio + <audio>|
|  - File picker IPC          |     for local files)   |
|  - Persistent config store  |  - Settings panel      |
|  - Remote health monitor    |    (server URL,        |
|  - Native toast notify      |     offline target,    |
|                             |     per-source sounds) |
+------------------------------------------------------+
```

### Why Tauri + Rust

Chosen after brainstorming. Electron rejected for size/memory. Tauri produces
a ~6 MB `.exe` using the system WebView2 runtime (pre-installed on Windows 11,
auto-installed on Windows 10). The Rust surface area is small (~300 LOC) because
no HTTP server is needed — only window chrome, IPC, file dialog, and health
polling.

## Components

### 1. Rust main process (`desktop/src-tauri/src/`)

- **`main.rs`** — entry, app builder, registers IPC commands, wires tray, loads
  persisted config, starts the health monitor task.
- **`commands.rs`** — IPC handlers invoked from JS:
  - `pick_audio_file() -> Option<PathBuf>` — opens native file dialog filtered
    to audio types, returns absolute path or `None` if cancelled.
  - `get_config() -> AppConfig` / `set_config(cfg: AppConfig)` — config CRUD
    persisted via `tauri-plugin-store`.
  - `set_autostart(enabled: bool)` — wraps `tauri-plugin-autostart`.
  - `show_window()` / `hide_to_tray()` — window visibility control.
- **`monitor.rs`** — background async task; every 15 s performs
  `reqwest::get(config.server_url + "/api/health")` with a 5 s timeout. State
  machine: `Online → (3 consecutive failures) → Offline → (1 success) → Online`.
  Emits `monitor-status-changed` Tauri event to the frontend on transitions.
- **`tray.rs`** — tray icon with menu: `Show / Hide / Quit`. Left-click toggles
  window.

### 2. Web renderer (`web/index.html` + small deltas)

- Detect `window.__TAURI__` to switch between browser-mode and desktop-mode.
- **New settings section** "Server Connection":
  - Input for server URL (default `https://ainotify.keymantek.com:777`)
  - Computed WS URL: `http → ws`, `https → wss`
  - Connection status reuses existing `statusDot`.
- **Per-row file picker** on each `sound-row`: a 📁 button next to the existing
  `select` and preview button. Clicking it calls `invoke('pick_audio_file')`,
  and if a path comes back, stores it in `soundPreferences[key]` with a
  `file://` prefix. Audio playback uses `new Audio(path)` which Tauri's
  `asset` protocol / `convertFileSrc` will resolve.
- **Remote monitor panel**:
  - Listens for `monitor-status-changed` events.
  - On `offline` transition: plays a distinct alarm (default: triple rising
    tone at 1200 → 1600 Hz, repeated 3×) and fires a native toast via Tauri.
  - On `online` transition: short ascending chord ("recovered").
  - Alarm sound is also user-customizable from the settings panel.

### 3. Packaging

- `tauri-cli` with Windows NSIS installer target.
- Produces `AI-Sound-Notify_1.0.0_x64_en-US.msi` **and** `AI-Sound-Notify.exe`
  (portable). Both ≤ 15 MB.
- Code signing: deferred (v1 ships unsigned; document SmartScreen warning).

## Data Flow

### Happy path (task complete notification)

1. Claude finishes a task on the remote box; hook `curl`s the Node server.
2. Node server broadcasts JSON to all WS clients, including this desktop app.
3. Renderer's `ws.onmessage` handler parses `{source, event, message}`, looks up
   the configured sound for `source:event`:
   - `synthesized` → existing Web Audio oscillator code
   - internal WAV name → existing `/sounds/*.wav` asset
   - `file://…` path → `new Audio(convertFileSrc(path))`
4. Plays via same gain node; fires Tauri toast if minimized.
5. Appends to history list (unchanged logic).

### Offline detection path

1. `monitor.rs` timer fires every 15 s.
2. Failure 1, 2, 3 within 45 s → state transitions to `Offline`.
3. Rust emits `monitor-status-changed: offline`.
4. Renderer catches event → plays alarm → fires toast "Server unreachable".
5. Monitor keeps polling. On first success → state `Online`, recovery sound.

### Config persistence

- File: `%APPDATA%\ai-sound-notify\config.json` (managed by
  `tauri-plugin-store`). Schema:
  ```json
  {
    "server_url": "https://ainotify.keymantek.com:777",
    "sound_preferences": { "claude-code:task_complete": "synthesized", … },
    "custom_audio_paths": { "<preference-key>": "C:/path/to/file.wav" },
    "volume": 0.6,
    "global_muted": false,
    "source_enabled": { "claude-code": true, "gemini": true, "codex": true },
    "autostart": true,
    "offline_alarm_enabled": true
  }
  ```
- Loaded into Rust on startup; hydrated into JS on window-ready via an IPC
  call so the web page state survives restart.

## Error Handling

| Scenario                              | Behaviour                                           |
|---------------------------------------|-----------------------------------------------------|
| Invalid server URL entered            | Frontend shows inline error under the input; no WS  |
| WS disconnect                         | Existing 3s reconnect loop; status dot turns red    |
| Health check times out                | Counts toward 3-strike offline threshold            |
| User picks non-audio file             | `pick_audio_file` filters; Rust side double-checks  |
|                                       | extension, returns `None` with log if invalid       |
| Custom file deleted after pick        | `new Audio` errors → fallback to synthesized sound  |
|                                       | and mark the preference as missing in UI            |
| `autostart` registry write denied     | Command returns `Err`; UI shows toast, keeps toggle |
|                                       | in its previous state                               |
| WebView2 runtime missing (Windows 10) | NSIS bootstrapper prompts installation              |

## Testing

- **Manual smoke test checklist** (documented in `docs/testing.md`):
  - Install, launch, verify tray + window
  - Trigger all 6 source×event combos via `Test` buttons (no network required)
  - Send real notification via `curl` from WSL → hear correct sound
  - Disconnect internet → verify offline alarm fires within 60 s
  - Reconnect → verify recovery sound fires
  - Pick a local `.wav` for claude-code:task_complete → retest → new sound plays
  - Toggle autostart; reboot; verify behaviour
  - Close the window → tray icon remains; right-click Quit fully exits
- **Rust unit tests**: the state machine in `monitor.rs` (pure logic, no I/O) —
  3 failures → offline, 1 success → online, etc.
- **No automated E2E**: Tauri E2E tooling is immature; accepted risk given
  project size.

## File Layout

```
person-202603-claude-ai-sound-notify/
├── server/              # untouched (existing Node server, runs in WSL)
├── web/                 # existing + small edits to index.html
├── desktop/             # NEW — Tauri project root
│   ├── src-tauri/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands.rs
│   │   │   ├── monitor.rs
│   │   │   └── tray.rs
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   └── icons/       # Windows app icons in multiple sizes
│   ├── dist/            # build output (web assets copied here pre-build)
│   └── package.json     # tauri-cli + scripts
└── docs/
    └── superpowers/specs/2026-04-17-desktop-notification-client-design.md  # this file
```

Tauri's `distDir` points at `desktop/dist`. A pre-build npm script copies
`web/index.html` and `web/sounds/` into `desktop/dist/`, so the web UI remains
the single source of truth.

## Build & Dev Workflow

- Windows is the build host. Install:
  - `rustup default stable-msvc`
  - VS 2022 Build Tools (Desktop development with C++)
  - Node.js 20+ (for tauri-cli glue + build scripts)
- Commands (run from Windows PowerShell in the repo root):
  ```
  cd desktop
  npm install
  npm run tauri dev       # launches WebView2 window against local files
  npm run tauri build     # produces installer + portable .exe in src-tauri/target/release/bundle/
  ```
- WSL is fine for code editing, git, and rust-analyzer. Cross-compilation via
  `cargo-xwin` is documented as a v2 nice-to-have.

## Open Questions — resolved

- ✅ Server vs client split: client only, server stays in WSL.
- ✅ Offline detection: HTTP health polling.
- ✅ Sound picking UX: native file dialog per row.
- ✅ Autostart: yes, via `tauri-plugin-autostart`.
- ✅ Default server URL: `https://ainotify.keymantek.com:777` (verified
  `/api/health` returns `{"status":"ok"}` at spec-write time).

## Deliverables

- `desktop/` directory with full Tauri source
- Edits to `web/index.html`
- `AI-Sound-Notify_1.0.0_x64.msi` + portable `.exe`
- Updated `README.md` section: "Windows Desktop Client"
- `docs/testing.md` smoke test checklist
