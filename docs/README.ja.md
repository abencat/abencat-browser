# 🐱 Abencat Browser · 阿笨猫指纹浏览器

オープンソースのアンチ検知指紋ブラウザ — 複数環境の分離、自動化 HTTP API と MCP サーバー内蔵、Windows / Linux、ヘッド有/無。永久無料。

🌐 [browser.abencat.com](https://browser.abencat.com) · **言語:** [English](../README.md) · [简体中文](README.zh-CN.md) · 日本語 · [한국어](README.ko.md) · [Русский](README.ru.md) · [Español](README.es.md) · [Français](README.fr.md) · [Deutsch](README.de.md)

> **指紋カーネルの帰属:** 指紋分離は **[CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ**（ソースレベルの指紋パッチを持つステルス Chromium）が提供します。本リポジトリはそのカーネルの**コントローラ/ランチャー**であり、カーネルは再配布しません。下記スクリプトで取得してください。詳細は [NOTICE](../NOTICE)。

## 機能
- 指紋分離（Canvas/WebGL/音声/フォント/WebRTC、環境ごとのシード、bot 検知を回避）
- 自動化 API（起動時にデバッグポート割当 + CDP WebSocket、Puppeteer/Playwright/Selenium 対応）
- MCP サーバー（AI が環境を直接操作）— [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)
- プロキシプール + GeoIP / ローカル暗号化（AES-256）/ クロスプラットフォーム・ヘッドレス / 多言語（8 言語）

## ⬇️ ダウンロード

**フル版（内核同梱・解凍してすぐ実行）:**
- 🪟 Windows: [abencat-windows-x64-full.zip](https://browser.abencat.com/downloads/abencat-windows-x64-full.zip) (225MB)
- 🐧 Linux: [abencat-linux-x64-full.tar.gz](https://browser.abencat.com/downloads/abencat-linux-x64-full.tar.gz) (210MB)

**コントローラ版（小さい・初回に内核を自動DL）:** [Win .exe](https://browser.abencat.com/downloads/AbencatBrowser-0.1.0-x64-setup.exe) · [Win headless](https://browser.abencat.com/downloads/abencat-headless-windows-x64.exe) · [Linux headless](https://browser.abencat.com/downloads/abencat-headless-linux-x64)

> 詳細は [English](../README.md) / [简体中文](README.zh-CN.md)

## クイックスタート
```bash
./scripts/download-browser.sh                 # CloakBrowser カーネル取得
export CLOAKBROWSER_BINARY_PATH=/opt/cloakbrowser/chrome
./cloak-headless                              # → http://127.0.0.1:50327 + token
```
詳細は [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md) / [English README](../README.md)。

## ライセンス
[MIT](../LICENSE) © Abencat Browser。指紋カーネル © CloakHQ。
