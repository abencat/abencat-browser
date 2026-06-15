# 🐱 Abencat Browser · 阿笨猫指纹浏览器

Open-Source Antidetect-Browser — isolierte Profile, integrierte Automatisierungs-API und MCP-Server, Windows / Linux, mit und ohne Oberfläche. Für immer kostenlos.

🌐 [b.abencat.com](https://b.abencat.com) · **Sprachen:** [English](../README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Español](README.es.md) · [Français](README.fr.md) · Deutsch

> **Kernel-Herkunft:** Die Fingerprint-Isolierung stammt von **[CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ** (ein Stealth-Chromium mit Patches auf Quellcode-Ebene). Dieses Repo ist ein **Controller/Launcher** für diesen Kernel, der nicht mitverteilt wird. Lade ihn mit dem Skript unten. Siehe [NOTICE](../NOTICE).

## Funktionen
- Fingerprint-Isolierung (Canvas/WebGL/Audio/Fonts/WebRTC, Seed pro Profil)
- Automatisierungs-API (Debug-Port + CDP-WebSocket; Puppeteer/Playwright/Selenium)
- MCP-Server (KI steuert Umgebungen) — [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)
- Proxy-Pool + GeoIP / lokale Verschlüsselung (AES-256) / plattformübergreifend·headless / 8 Sprachen

## Schnellstart
```bash
./scripts/download-browser.sh
export CLOAKBROWSER_BINARY_PATH=/opt/cloakbrowser/chrome
./cloak-headless                              # → http://127.0.0.1:50327 + token
```
Mehr: [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md) / [English README](../README.md)

## Lizenz
[MIT](../LICENSE) © Abencat Browser. Fingerprint-Kernel © CloakHQ.
