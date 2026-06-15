# 🐱 Abencat Browser · 阿笨猫指纹浏览器

Navegador antidetección de código abierto — perfiles aislados, API de automatización y servidor MCP integrados, Windows / Linux, con y sin interfaz. Gratis para siempre.

🌐 [browser.abencat.com](https://browser.abencat.com) · **Idiomas:** [English](../README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · Español · [Français](README.fr.md) · [Deutsch](README.de.md)

> **Origen del núcleo:** el aislamiento de huella lo proporciona **[CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ** (un Chromium sigiloso con parches a nivel de código). Este repositorio es un **controlador/lanzador** de ese núcleo, que no se redistribuye. Descárgalo con el script de abajo. Ver [NOTICE](../NOTICE).

## Funciones
- Aislamiento de huella (Canvas/WebGL/Audio/Fuentes/WebRTC, semilla por perfil)
- API de automatización (puerto de depuración + CDP WebSocket; Puppeteer/Playwright/Selenium)
- Servidor MCP (la IA controla los entornos) — [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)
- Pool de proxies + GeoIP / cifrado local (AES-256) / multiplataforma·headless / 8 idiomas

## ⬇️ Descarga

**Completa (con núcleo, lista para usar):**
- 🪟 Windows: [abencat-windows-x64-full.zip](https://browser.abencat.com/downloads/abencat-windows-x64-full.zip) (225MB)
- 🐧 Linux: [abencat-linux-x64-full.tar.gz](https://browser.abencat.com/downloads/abencat-linux-x64-full.tar.gz) (210MB)

**Solo controlador (auto-descarga el núcleo):** [Win .exe](https://browser.abencat.com/downloads/AbencatBrowser-0.1.0-x64-setup.exe) · [Win headless](https://browser.abencat.com/downloads/abencat-headless-windows-x64.exe) · [Linux headless](https://browser.abencat.com/downloads/abencat-headless-linux-x64)

> Más en [English](../README.md) / [简体中文](README.zh-CN.md)

## Inicio rápido
```bash
./scripts/download-browser.sh
export CLOAKBROWSER_BINARY_PATH=/opt/cloakbrowser/chrome
./cloak-headless                              # → http://127.0.0.1:50327 + token
```
Más: [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md) / [English README](../README.md)

## Licencia
[MIT](../LICENSE) © Abencat Browser. Núcleo de huella © CloakHQ.
