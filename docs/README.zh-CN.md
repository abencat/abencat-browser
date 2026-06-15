<div align="center">

# 🐱 阿笨猫指纹浏览器 · Abencat Browser

**开源防关联指纹浏览器** —— 多环境隔离、内置自动化 HTTP API 与 MCP 服务、
Windows / Linux、有头与无头。永久免费。

🌐 [b.abencat.com](https://b.abencat.com) ·
**语言：** [English](../README.md) · 简体中文 · [日本語](README.ja.md) ·
[한국어](README.ko.md) · [Русский](README.ru.md) · [Español](README.es.md) ·
[Français](README.fr.md) · [Deutsch](README.de.md)

</div>

---

> **指纹内核来源声明：** 指纹隔离能力由
> **[CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ** 提供 ——
> 一个带源码级指纹补丁的隐身 Chromium。阿笨猫指纹浏览器是围绕该内核的**开源控制器/启动器**。
> 本仓库**不分发**内核,请用下方脚本下载。浏览器引擎版权归 CloakHQ 所有。详见 [NOTICE](../NOTICE)。

## ✨ 功能

- **指纹隔离** —— Canvas / WebGL / 音频 / 字体 / WebRTC,每环境独立种子,过 bot 检测(基于 CloakBrowser 内核)。
- **自动化 API** —— 启动即分配调试端口并返回 CDP WebSocket,秒接 Puppeteer / Playwright / Selenium。
- **MCP 服务** —— AI(Claude、Cursor…)可直接创建/启动/控制环境。详见 [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)。
- **代理池 + GeoIP** —— 批量导入、连通性检测、按代理自动同步语言/时区/出口 IP。
- **本地加密** —— 代理密码与 Cookie 落盘加密(AES-256-GCM),可选 Argon2id 主密码。
- **跨平台 / 无头** —— Windows 桌面 + Linux 服务器,有头/无头按次切换。
- **多语言** —— 界面 8 种语言(中/英全量 + 日/韩/俄/西/法/德)。

## 📦 下载

预编译产物见 [Releases](../../../releases) 或 [b.abencat.com](https://b.abencat.com)。

| | 有头(GUI) | 无头(服务器) |
|---|---|---|
| Windows | `Abencat Browser_*_x64-setup.exe` | `cloak-headless.exe` |
| Linux x86_64 | 桌面版(需 webkit2gtk)/ `xvfb-run` | `cloak-headless` |

## 🚀 快速开始

```bash
# 1) 获取 CloakBrowser 指纹内核(本仓库不附带)
./scripts/download-browser.sh            # Linux/macOS
#  scripts\download-browser.ps1           # Windows

# 2a) 桌面版 —— 直接运行安装包/可执行文件。
# 2b) 无头服务(Win/Linux)
export CLOAKBROWSER_BINARY_PATH=/opt/cloakbrowser/chrome
./cloak-headless        # → http://127.0.0.1:50327 + token
```

通过 HTTP 或 MCP 驱动,完整指南见 **[AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)**。

## 🛠 从源码构建

```bash
npm install && npm run tauri:build      # 桌面版安装包
cd src-tauri && cargo build --release --bin cloak-headless --no-default-features   # 无头(无需 webkit)
```

## 📜 许可

[MIT](../LICENSE) © 阿笨猫指纹浏览器。指纹内核 © CloakHQ([CloakBrowser](https://github.com/CloakHQ/CloakBrowser))。
