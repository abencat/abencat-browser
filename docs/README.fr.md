# 🐱 Abencat Browser · 阿笨猫指纹浏览器

Navigateur anti-détection open source — profils isolés, API d'automatisation et serveur MCP intégrés, Windows / Linux, avec et sans interface. Gratuit à vie.

🌐 [browser.abencat.com](https://browser.abencat.com) · **Langues :** [English](../README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Español](README.es.md) · Français · [Deutsch](README.de.md)

> **Origine du noyau :** l'isolation d'empreinte est fournie par **[CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ** (un Chromium furtif avec des correctifs au niveau du code). Ce dépôt est un **contrôleur/lanceur** de ce noyau, qui n'est pas redistribué. Téléchargez-le avec le script ci-dessous. Voir [NOTICE](../NOTICE).

## Fonctions
- Isolation d'empreinte (Canvas/WebGL/Audio/Polices/WebRTC, graine par profil)
- API d'automatisation (port de débogage + WebSocket CDP ; Puppeteer/Playwright/Selenium)
- Serveur MCP (l'IA pilote les environnements) — [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)
- Pool de proxies + GeoIP / chiffrement local (AES-256) / multiplateforme·headless / 8 langues

## ⬇️ Téléchargement

**Complète (noyau inclus, prête à lancer):**
- 🪟 Windows: [abencat-windows-x64-full.zip](https://browser.abencat.com/downloads/abencat-windows-x64-full.zip) (225MB)
- 🐧 Linux: [abencat-linux-x64-full.tar.gz](https://browser.abencat.com/downloads/abencat-linux-x64-full.tar.gz) (210MB)

**Contrôleur seul (télécharge le noyau):** [Win .exe](https://browser.abencat.com/downloads/AbencatBrowser-0.1.0-x64-setup.exe) · [Win headless](https://browser.abencat.com/downloads/abencat-headless-windows-x64.exe) · [Linux headless](https://browser.abencat.com/downloads/abencat-headless-linux-x64)

> Plus dans [English](../README.md) / [简体中文](README.zh-CN.md)

## Démarrage rapide
```bash
./scripts/download-browser.sh
export CLOAKBROWSER_BINARY_PATH=/opt/cloakbrowser/chrome
./cloak-headless                              # → http://127.0.0.1:50327 + token
```
Plus : [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md) / [English README](../README.md)

## Licence
[MIT](../LICENSE) © Abencat Browser. Noyau d'empreinte © CloakHQ.
