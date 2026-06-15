# 教程:让 AI 直接驱动指纹浏览器 —— 阿笨猫指纹浏览器 + MCP

> 用 Puppeteer / Playwright / Selenium 自动化指纹隔离的浏览器,或让 AI(Claude、Cursor)
> 通过 **MCP** 直接操控。开源免费。
> 🌐 [browser.abencat.com](https://browser.abencat.com) · [English](tutorial-ai-automation.md)

AdsPower / 比特浏览器这类指纹浏览器大多收费且闭源。**阿笨猫指纹浏览器(Abencat Browser)**
是围绕 [CloakBrowser](https://github.com/CloakHQ/CloakBrowser) 隐身内核的 **MIT 开源、永久免费**
控制器,并多了两样别人没有的东西:**Linux 服务器无头版** 和 **原生 MCP 服务**——让 AI 自己
创建、启动、操控指纹环境。

本文走一遍完整流程:安装 → 起自动化服务 → 用 Puppeteer 驱动 → 再用 MCP 让 AI 直接驱动。

## 1. 安装

**完整版(含浏览器内核,最省事):**
- 🪟 [abencat-windows-x64-full.zip](https://browser.abencat.com/downloads/abencat-windows-x64-full.zip) — 解压,双击 `Abencat Browser.exe`
- 🐧 [abencat-linux-x64-full.tar.gz](https://browser.abencat.com/downloads/abencat-linux-x64-full.tar.gz) — 解压,`./cloak-headless`

**控制器版**(体积小,首次运行自动下载内核):见[下载页](https://browser.abencat.com);服务器也可
`./scripts/download-browser.sh` 拉内核。

## 2. 启动自动化服务(无头)

```bash
./cloak-headless
#   endpoint : http://127.0.0.1:50327
#   token    : 8f3c…          # 只打印一次,写操作要用
```

服务**仅监听本地**、token 鉴权。远程机器用隧道:`ssh -L 50327:127.0.0.1:50327 user@server`。

## 3. 用 HTTP 驱动(任意语言)

```bash
API=http://127.0.0.1:50327 ; TOKEN=<token>
ID=$(curl -s "$API/api/v1/profile/create?name=job1&token=$TOKEN" | jq -r .data.id)   # 建环境
curl -s "$API/api/v1/browser/start?id=$ID&headless=1&token=$TOKEN"                    # 启动→返回 ws
```

## 4. 接 Puppeteer / Playwright / Selenium

```js
// Puppeteer
const { data } = await (await fetch(`${API}/api/v1/browser/start?id=${ID}&token=${TOKEN}`)).json();
const browser = await require("puppeteer-core").connect({ browserWSEndpoint: data.ws });
await (await browser.pages())[0].goto("https://pixelscan.net");
```
Playwright 用 `chromium.connectOverCDP(data.ws)`;Selenium 用 `debuggerAddress`(内核自带匹配的 chromedriver)。

## 5. 重点:让 AI 直接驱动(MCP)

内置 MCP 服务器 `mcp/cloak-mcp.mjs`(Node≥18,零依赖)。配到 AI 客户端后,助手就拥有
`list_profiles / create_profile / start_browser / stop_browser …` 工具。

本地(Claude Desktop / Cursor 的 `mcpServers`):
```json
{ "mcpServers": { "abencat": {
  "command": "node",
  "args": ["/path/to/mcp/cloak-mcp.mjs"],
  "env": { "CLOAK_API_BASE": "http://127.0.0.1:50327", "CLOAK_API_TOKEN": "<token>" }
}}}
```
远程服务器(SSH-stdio,token 自动从日志读):
```json
{ "mcpServers": { "abencat": { "command": "ssh", "args": ["-p","22","user@server","/usr/local/bin/cloak-mcp"] }}}
```

然后你直接对 AI 说:**「建一个环境,打开 pixelscan.net,告诉我它识别的操作系统」**——它会自动
`create_profile` → `start_browser` → 用返回的 `ws` 连接并读页面,无需你写胶水代码。

## 6. 验证指纹生效

启动后访问 `http://127.0.0.1:<debugPort>/json/version`:把环境 `platform` 设成 windows,
**在 Linux 服务器上 UA 也会显示 `Windows NT 10.0`**——证明源码级补丁生效。更全面用
[pixelscan.net](https://pixelscan.net) / [browserleaks.com](https://browserleaks.com) 比对。

## 7. 小贴士
- **代理**:环境绑代理后,启动时按代理 GeoIP 自动同步语言/时区/出口 IP;敏感数据 AES-256 落盘加密。
- **有头/无头**:`browser/start` 加 `&headless=0` 出可见窗口(Linux 服务器用 `xvfb-run` 跑服务)。
- **规模**:开源不限环境数;每次 `start` 各自分配调试端口,可大量并行。

## 链接
- ⭐ 仓库:https://github.com/abencat/abencat-browser
- 🌐 官网/下载:https://browser.abencat.com
- 🤖 完整 API/MCP 参考:[AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)
- 🧩 内核:[CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ
