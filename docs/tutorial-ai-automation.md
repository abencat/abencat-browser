# Tutorial: Let AI drive an anti-detect browser — Abencat Browser + MCP

> Automate fingerprint-isolated browsers with Puppeteer / Playwright / Selenium, or
> let an AI agent (Claude, Cursor) control them directly via **MCP**. Open-source & free.
> 🌐 [browser.abencat.com](https://browser.abencat.com) · [简体中文教程](tutorial-ai-automation.zh-CN.md)

Most anti-detect browsers (AdsPower, Multilogin, BitBrowser) are closed-source and
paid. **Abencat Browser** (阿笨猫指纹浏览器) is an MIT-licensed, free controller around
the [CloakBrowser](https://github.com/CloakHQ/CloakBrowser) stealth-Chromium kernel,
with two things they lack: a **headless Linux server build** and a **native MCP server**
so AI agents can spin up and drive fingerprint profiles on their own.

This guide builds a complete flow: install → start the automation server → drive a
profile with Puppeteer → do the same through MCP from an AI assistant.

## 1. Install

**Full bundle (includes the browser kernel, easiest):**
- 🪟 [abencat-windows-x64-full.zip](https://browser.abencat.com/downloads/abencat-windows-x64-full.zip) — unzip, run `Abencat Browser.exe`
- 🐧 [abencat-linux-x64-full.tar.gz](https://browser.abencat.com/downloads/abencat-linux-x64-full.tar.gz) — extract, run `./cloak-headless`

**Controller only** (tiny, auto-downloads the kernel on first run): see the
[downloads page](https://browser.abencat.com). On a server you can also fetch the
kernel with `./scripts/download-browser.sh`.

## 2. Start the automation server (headless)

```bash
./cloak-headless
# Abencat Browser — headless automation API ready
#   endpoint : http://127.0.0.1:50327
#   token    : 8f3c…              # printed once; needed for write calls
```

The server binds **loopback only** and is token-protected. On a remote box, tunnel it:

```bash
ssh -L 50327:127.0.0.1:50327 user@server
```

## 3. Drive it over HTTP (any language)

```bash
API=http://127.0.0.1:50327 ; TOKEN=<token>
# create a profile (its fingerprint is generated for you)
ID=$(curl -s "$API/api/v1/profile/create?name=job1&token=$TOKEN" | jq -r .data.id)
# launch it — returns a CDP WebSocket + debuggerAddress
curl -s "$API/api/v1/browser/start?id=$ID&headless=1&token=$TOKEN"
# → {"data":{"ws":"ws://127.0.0.1:42555/devtools/browser/…","debuggerAddress":"127.0.0.1:42555"}}
```

## 4. Connect Puppeteer / Playwright / Selenium

```js
// Puppeteer (Node)
const { data } = await (await fetch(
  `${API}/api/v1/browser/start?id=${ID}&token=${TOKEN}`)).json();
const browser = await require("puppeteer-core").connect({ browserWSEndpoint: data.ws });
const page = (await browser.pages())[0];
await page.goto("https://pixelscan.net");
```

```python
# Selenium (Python) — uses debuggerAddress
import requests; from selenium import webdriver
from selenium.webdriver.chrome.options import Options
d = requests.get(f"{API}/api/v1/browser/start", params={"id": ID, "token": TOKEN}).json()["data"]
o = Options(); o.add_experimental_option("debuggerAddress", d["debuggerAddress"])
driver = webdriver.Chrome(options=o)  # chromedriver ships next to the kernel's chrome
```

Playwright: `chromium.connectOverCDP(data.ws)`.

## 5. The interesting part — let AI drive it (MCP)

Abencat ships an MCP server (`mcp/cloak-mcp.mjs`, Node ≥18, zero deps). Point your AI
host at it and the assistant gets tools: `list_profiles`, `create_profile`,
`start_browser`, `stop_browser`, …

Local (Claude Desktop / Cursor `mcpServers`):

```json
{
  "mcpServers": {
    "abencat": {
      "command": "node",
      "args": ["/path/to/mcp/cloak-mcp.mjs"],
      "env": { "CLOAK_API_BASE": "http://127.0.0.1:50327", "CLOAK_API_TOKEN": "<token>" }
    }
  }
}
```

Remote server (stdio over SSH, token auto-read from the service log):

```json
{ "mcpServers": { "abencat": {
  "command": "ssh", "args": ["-p","22","user@server","/usr/local/bin/cloak-mcp"]
}}}
```

Now you can just tell the assistant: *"create a profile, open pixelscan.net, and tell
me the reported OS"* — it calls `create_profile` → `start_browser` → connects to the
returned `ws` and reads the page. No glue code.

## 6. Verify the fingerprint works

After `start_browser`, hit `http://127.0.0.1:<debugPort>/json/version`. With a
`platform=windows` profile, the User-Agent reports **`Windows NT 10.0` even on a Linux
server** — proof the source-level patches apply. For a full check, open
[pixelscan.net](https://pixelscan.net) or [browserleaks.com](https://browserleaks.com).

## 7. Tips

- **Proxies:** bind a proxy to a profile; on launch, language/timezone/exit-IP auto-sync
  from the proxy's GeoIP. Encrypted at rest (AES-256).
- **Headed vs headless:** add `&headless=0` to `browser/start` for a visible window
  (on a Linux server, run the service under `xvfb-run`).
- **Scale:** unlimited profiles (open-source, no quota). Each `start` allocates its own
  debug port, so run many in parallel.

## Links
- ⭐ Repo: https://github.com/abencat/abencat-browser
- 🌐 Site & downloads: https://browser.abencat.com
- 🤖 Full API/MCP reference: [AGENT_AUTOMATION.md](../AGENT_AUTOMATION.md)
- 🧩 Kernel: [CloakBrowser](https://github.com/CloakHQ/CloakBrowser) © CloakHQ
