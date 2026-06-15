# 🐱 Abencat Browser · 阿笨猫指纹浏览器

오픈소스 안티디텍트 지문 브라우저 — 다중 환경 격리, 자동화 HTTP API 및 MCP 서버 내장, Windows / Linux, 헤드/헤드리스. 영구 무료.

🌐 [browser.abencat.com](https://browser.abencat.com) · **언어:** [English](../README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · 한국어 · [Русский](README.ru.md) · [Español](README.es.md) · [Français](README.fr.md) · [Deutsch](README.de.md)

> **지문 커널 출처:** 지문 격리는 **[CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ**（소스 레벨 지문 패치를 가진 스텔스 Chromium）가 제공합니다. 이 저장소는 해당 커널의 **컨트롤러/런처**이며 커널을 재배포하지 않습니다. 아래 스크립트로 받으세요. [NOTICE](../NOTICE) 참고.

## 기능
- 지문 격리（Canvas/WebGL/오디오/폰트/WebRTC, 환경별 시드, 봇 탐지 우회）
- 자동화 API（실행 시 디버그 포트 + CDP WebSocket, Puppeteer/Playwright/Selenium）
- MCP 서버（AI가 환경 직접 제어）— [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)
- 프록시 풀 + GeoIP / 로컬 암호화（AES-256）/ 크로스 플랫폼·헤드리스 / 다국어（8개）

## ⬇️ 다운로드

**완전판（커널 포함・압축 해제 후 실행）:**
- 🪟 Windows: [abencat-windows-x64-full.zip](https://browser.abencat.com/downloads/abencat-windows-x64-full.zip) (225MB)
- 🐧 Linux: [abencat-linux-x64-full.tar.gz](https://browser.abencat.com/downloads/abencat-linux-x64-full.tar.gz) (210MB)

**컨트롤러판（소형・최초 실행 시 커널 자동 다운로드）:** [Win .exe](https://browser.abencat.com/downloads/AbencatBrowser-0.1.0-x64-setup.exe) · [Win headless](https://browser.abencat.com/downloads/abencat-headless-windows-x64.exe) · [Linux headless](https://browser.abencat.com/downloads/abencat-headless-linux-x64)

> 자세히는 [English](../README.md) / [简体中文](README.zh-CN.md)

## 빠른 시작
```bash
./scripts/download-browser.sh
export CLOAKBROWSER_BINARY_PATH=/opt/cloakbrowser/chrome
./cloak-headless                              # → http://127.0.0.1:50327 + token
```
자세히: [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md) / [English README](../README.md)

## 라이선스
[MIT](../LICENSE) © Abencat Browser. 지문 커널 © CloakHQ.
