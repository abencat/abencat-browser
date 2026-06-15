# 🐱 Abencat Browser · 阿笨猫指纹浏览器

Антидетект-браузер с открытым кодом — изоляция профилей, встроенный API автоматизации и MCP-сервер, Windows / Linux, headed и headless. Навсегда бесплатно.

🌐 [browser.abencat.com](https://browser.abencat.com) · **Языки:** [English](../README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · Русский · [Español](README.es.md) · [Français](README.fr.md) · [Deutsch](README.de.md)

> **Происхождение ядра:** изоляцию отпечатка обеспечивает **[CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ** — скрытный Chromium с патчами на уровне кода. Этот репозиторий — **контроллер/лаунчер** ядра, само ядро не распространяется. Загрузите его скриптом ниже. См. [NOTICE](../NOTICE).

## Возможности
- Изоляция отпечатка (Canvas/WebGL/Audio/шрифты/WebRTC, сид на профиль, обход антибота)
- API автоматизации (порт отладки + CDP WebSocket, Puppeteer/Playwright/Selenium)
- MCP-сервер (ИИ управляет средами) — [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)
- Пул прокси + GeoIP / локальное шифрование (AES-256) / кроссплатформенность·headless / 8 языков

## ⬇️ Загрузка

**Полная (с ядром, готова к запуску):**
- 🪟 Windows: [abencat-windows-x64-full.zip](https://browser.abencat.com/downloads/abencat-windows-x64-full.zip) (225MB)
- 🐧 Linux: [abencat-linux-x64-full.tar.gz](https://browser.abencat.com/downloads/abencat-linux-x64-full.tar.gz) (210MB)

**Только контроллер (авто-загрузка ядра):** [Win .exe](https://browser.abencat.com/downloads/AbencatBrowser-0.1.0-x64-setup.exe) · [Win headless](https://browser.abencat.com/downloads/abencat-headless-windows-x64.exe) · [Linux headless](https://browser.abencat.com/downloads/abencat-headless-linux-x64)

> Подробнее в [English](../README.md) / [简体中文](README.zh-CN.md)

## Быстрый старт
```bash
./scripts/download-browser.sh
export CLOAKBROWSER_BINARY_PATH=/opt/cloakbrowser/chrome
./cloak-headless                              # → http://127.0.0.1:50327 + token
```
Подробнее: [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md) / [English README](../README.md)

## Лицензия
[MIT](../LICENSE) © Abencat Browser. Ядро отпечатка © CloakHQ.
