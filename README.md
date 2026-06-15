<div align="center">

# 🐱 Abencat Browser · 阿笨猫指纹浏览器

**Open-source anti-detect fingerprint browser** — multi-profile isolation, a built-in
automation HTTP API + MCP server, Windows / Linux, headed & headless. Free forever.

🌐 [browser.abencat.com](https://browser.abencat.com) ·
**Languages:** English · [简体中文](docs/README.zh-CN.md) · [日本語](docs/README.ja.md) ·
[한국어](docs/README.ko.md) · [Русский](docs/README.ru.md) · [Español](docs/README.es.md) ·
[Français](docs/README.fr.md) · [Deutsch](docs/README.de.md)

</div>

---

> **Fingerprint kernel credit:** the fingerprint isolation is provided by
> **[CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ** — a stealth
> Chromium with source-level fingerprint patches. Abencat Browser is an open-source
> *controller / launcher* around that kernel. The kernel is **not** redistributed in
> this repo; download it with the script below. All credit for the browser engine goes
> to CloakHQ. See [NOTICE](NOTICE).

## ✨ Features

- **Fingerprint isolation** — Canvas / WebGL / Audio / Fonts / WebRTC, per-profile seed; passes bot detection (via CloakBrowser kernel).
- **Automation API** — every launch auto-allocates a debug port and returns a CDP WebSocket; connect Puppeteer / Playwright / Selenium instantly.
- **MCP server** — AI agents (Claude, Cursor, …) create / start / control environments directly. See [AGENT_AUTOMATION.md](AGENT_AUTOMATION.md).
- **Proxy pool + GeoIP** — bulk import, health checks, auto-sync language / timezone / exit IP from the proxy.
- **Local encryption** — proxy passwords & cookies encrypted at rest (AES-256-GCM), optional Argon2id master password.
- **Cross-platform & headless** — Windows desktop + Linux server; switch headed/headless per launch.
- **i18n** — UI in 8 languages (zh/en full + ja/ko/ru/es/fr/de).

## 📦 Download

All downloads are hosted on **[browser.abencat.com](https://browser.abencat.com)** (the GitHub
Release carries source archives only).

**Full bundle — includes the browser kernel, ready to run (no extra download):**

| Platform | Download |
|---|---|
| 🪟 Windows x64 | [abencat-windows-x64-full.zip](https://browser.abencat.com/downloads/abencat-windows-x64-full.zip) (225 MB) — unzip & run `Abencat Browser.exe` |
| 🐧 Linux x86_64 | [abencat-linux-x64-full.tar.gz](https://browser.abencat.com/downloads/abencat-linux-x64-full.tar.gz) (210 MB) — extract & run `./cloak-headless` |

**Controller only — tiny, auto-downloads the kernel on first run:**

| Platform | Headed (GUI) | Headless (server) |
|---|---|---|
| Windows | [installer](https://browser.abencat.com/downloads/AbencatBrowser-0.1.0-x64-setup.exe) (5 MB) | [cloak-headless.exe](https://browser.abencat.com/downloads/abencat-headless-windows-x64.exe) |
| Linux x86_64 | desktop build (needs webkit2gtk) / `xvfb-run` | [cloak-headless](https://browser.abencat.com/downloads/abencat-headless-linux-x64) |

> Fingerprint kernel (standalone): [Windows zip](https://browser.abencat.com/downloads/cloakbrowser-windows-x64.zip) · [Linux tar.gz](https://browser.abencat.com/downloads/cloakbrowser-linux-x64.tar.gz)

## 🚀 Quick start

```bash
# 1) Get the CloakBrowser fingerprint kernel (not shipped in this repo)
./scripts/download-browser.sh           # Linux/macOS
#  scripts\download-browser.ps1          # Windows (PowerShell)

# 2a) Desktop app — just run the installer / executable.

# 2b) Headless server (Windows/Linux)
export CLOAKBROWSER_BINARY_PATH=/opt/cloakbrowser/chrome   # or use the script's output
./cloak-headless
#   → http://127.0.0.1:50327  + a token (printed once)
```

Drive it over HTTP or MCP — full guide in **[AGENT_AUTOMATION.md](AGENT_AUTOMATION.md)**:

```bash
GET /api/v1/browser/start?id=<id>&headless=1&token=<token>
# → { "ws": "ws://127.0.0.1:42555/devtools/browser/...", "debuggerAddress": "127.0.0.1:42555" }
```

## 🛠 Build from source

```bash
# Frontend + desktop app (Windows/macOS/Linux desktop)
npm install
npm run tauri:build                 # → installer under src-tauri/target/release/bundle

# Headless server binary (no Tauri/webkit, great for Ubuntu servers)
cd src-tauri
cargo build --release --bin cloak-headless --no-default-features
```

Toolchain: Rust (stable), Node ≥18. Linux desktop build needs `libwebkit2gtk-4.1-dev`
and friends — see [LINUX_HEADLESS.md](LINUX_HEADLESS.md).

## 📂 Layout

```
src-tauri/src/   Rust backend (models, store, crypto, proxy, launch, api, commands, …)
src/             React/TS frontend (App.tsx, i18n.ts, components/)
mcp/             MCP server (cloak-mcp.mjs)
website/         landing page (browser.abencat.com, 8 languages)
docs/            multi-language README + guides
scripts/         download-browser.{sh,ps1}
```

## 📜 License

[MIT](LICENSE) © Abencat Browser. Fingerprint kernel © CloakHQ ([CloakBrowser](https://github.com/CloakHQ/CloakBrowser)).
