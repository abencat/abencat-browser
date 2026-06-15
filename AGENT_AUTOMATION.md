# Abencat 指纹浏览器 — AI 自动化对接指南

本文件面向 **AI/Agent**:读完即可生成「创建指纹环境 → 启动浏览器 → 连接 → 自动化操作 → 关闭」的可运行代码。两种驱动方式:**MCP 工具**(让 AI 直接调用)或 **HTTP API**(生成脚本)。底层指纹内核为 [CloakBrowser](https://github.com/CloakHQ/CloakBrowser)(源码级指纹补丁,过 bot 检测)。

---

## 0. 心智模型(给 AI)

- 一个**环境(profile)** = 一套隔离的指纹 + cookie + 代理 + user-data 目录。
- 启动一个环境 → 拉起一个 CloakBrowser 进程(可**有头/无头**),并返回一个 **CDP WebSocket 端点**。
- 你**不直接控制指纹**;指纹由内核按环境配置自动施加。你只需:选/建环境 → start → 拿到 `ws`/`debuggerAddress` → 用 Puppeteer/Playwright/Selenium 接管 → 操作页面。
- 同一套环境数据在桌面版与无头服务间**共享**(同一 `controller-data`)。

典型一次任务:
```
list_profiles → (没有合适的就 create_profile) → start_browser(id) →
用返回的 ws 连接 Puppeteer/Playwright → 打开页面、填表、点按、取数 → stop_browser(id)
```

---

## 1. 运行形态与产物(Windows / Linux × 有头 / 无头)

| 平台 | 有头(带界面) | 无头(服务器) |
|---|---|---|
| **Windows** | 安装包 `Abencat Browser_*_x64-setup.exe`;或便携版 `Abencat Browser.exe`(均见 [browser.abencat.com](https://browser.abencat.com)) | `cloak-headless.exe` |
| **Linux/Ubuntu** | 桌面版(需 webkit2gtk,见 `LINUX_HEADLESS.md`);或服务器上用 **Xvfb** 跑有头(见 §6) | `cloak-headless`(`cargo build --release --bin cloak-headless --no-default-features`) |

无论哪种形态,都会在 `127.0.0.1:50327` 暴露**同一套自动化 HTTP API**,并可被 MCP 服务器包装。
**有头/无头是「每次启动」可切换的**(API 加 `headless=0|1`,MCP 工具加 `headless` 参数);桌面版默认有头,`cloak-headless` 默认无头。

---

## 2. 方式 A:MCP(推荐让 AI 直接调用)

MCP 服务器:`mcp/cloak-mcp.mjs`(零依赖,Node ≥18,stdio JSON-RPC)。它把 HTTP API 包成工具。

### 2.1 配置(Claude Desktop / Claude Code / Cursor 等)

`mcpServers` 配置(把路径换成实际位置,token 见 §4.1):
```json
{
  "mcpServers": {
    "cloak-fingerprint": {
      "command": "node",
      "args": ["E:/newpro/newchrome/FingerprintControllerRust/mcp/cloak-mcp.mjs"],
      "env": {
        "CLOAK_API_BASE": "http://127.0.0.1:50327",
        "CLOAK_API_TOKEN": "<从服务日志读取的 token>"
      }
    }
  }
}
```
**远程服务器(SSH-stdio,推荐)**:服务器上已部署 MCP(`/opt/cloak-mcp/cloak-mcp.mjs` + 启动器 `/usr/local/bin/cloak-mcp`,自动从服务日志读取实时 token)。AI 主机直接用 ssh 作为 MCP 命令,stdio 走 SSH 管道:
```json
{
  "mcpServers": {
    "cloak-remote": {
      "command": "ssh",
      "args": ["-p", "10196", "root@154.222.31.19", "/usr/local/bin/cloak-mcp"]
    }
  }
}
```
启动器会自动注入当前 token,无需手填(服务重启换 token 也不受影响)。需免密登录(配置 SSH key)。
另一种:建隧道 `ssh -p <port> -L 50327:127.0.0.1:50327 root@HOST` 后在本机跑 MCP,`CLOAK_API_BASE=http://127.0.0.1:50327`。

### 2.2 工具清单(MCP tools)

| 工具 | 入参 | 返回 |
|---|---|---|
| `status` | — | 版本、是否 headless、运行中 id |
| `list_profiles` | — | 全部环境 `{id,name,projectId,running}` |
| `create_profile` | `name?`, `projectId?` | `{id}` |
| `delete_profile` | `id` | `{id}` |
| `start_browser` | `id`, `url?`, `headless?` | `{ws, debuggerAddress, debugPort, http}` |
| `stop_browser` | `id` | `{id}` |
| `active_endpoints` | — | 运行中环境的 CDP 端点表 |

`start_browser` 返回的 `ws` 给 Puppeteer/Playwright,`debuggerAddress` 给 Selenium。

### 2.3 AI 调用流程(伪代码)

```
profiles = list_profiles()
id = profiles 里挑一个 / 或 create_profile(name="task-1").id
ep = start_browser(id)               // ep.ws, ep.debuggerAddress
→ 用 ep.ws 连 Puppeteer 执行任务
stop_browser(id)
```

---

## 3. 方式 B:HTTP API(生成独立脚本)

Base:`http://127.0.0.1:50327`。写操作必须带 `&token=<TOKEN>`,仅监听回环。

| 方法/路径 | 说明 |
|---|---|
| `GET /api/v1/status` | 状态(含 `headless`) |
| `GET /api/v1/profiles` | 列出环境 |
| `GET /api/v1/profile/create?name=&projectId=&token=` | 新建 → `{data:{id}}` |
| `GET /api/v1/profile/delete?id=&token=` | 删除 |
| `GET /api/v1/browser/start?id=&url=&headless=&token=` | 启动 → `{data:{ws,debuggerAddress,debugPort,http}}` |
| `GET /api/v1/browser/stop?id=&token=` | 关闭 |
| `GET /api/v1/browser/active` | 运行中端点 |

`headless`:`1/true` 无头,`0/false` 有头,省略=该服务默认。

start 返回示例:
```json
{ "code": 0, "data": {
  "id": "xxx",
  "ws": "ws://127.0.0.1:42555/devtools/browser/....",
  "debuggerAddress": "127.0.0.1:42555",
  "debugPort": 42555,
  "http": "http://127.0.0.1:42555"
}}
```

---

## 4. 连接自动化框架(完整可运行示例)

### 4.1 取 token
- 无头服务:`journalctl -u cloak-headless | grep token | tail -1`(systemd)或看进程启动输出。
- 桌面版:应用内「自动化」面板可查看/复制。

### 4.2 Puppeteer(Node,用 `ws`)
```js
const TOKEN = process.env.CLOAK_API_TOKEN;
const API = "http://127.0.0.1:50327";
const id = "<profileId>";                                  // 或先 create
const r = await (await fetch(`${API}/api/v1/browser/start?id=${id}&token=${TOKEN}`)).json();
const puppeteer = require("puppeteer-core");
const browser = await puppeteer.connect({ browserWSEndpoint: r.data.ws });
const page = (await browser.pages())[0] || await browser.newPage();
await page.goto("https://browserleaks.com/javascript");
console.log(await page.evaluate(() => navigator.platform));  // 受指纹控制
await browser.disconnect();
await fetch(`${API}/api/v1/browser/stop?id=${id}&token=${TOKEN}`);
```

### 4.3 Playwright(Node,用 CDP `ws`)
```js
const { chromium } = require("playwright");
const r = await (await fetch(`${API}/api/v1/browser/start?id=${id}&token=${TOKEN}`)).json();
const browser = await chromium.connectOverCDP(r.data.ws);
const ctx = browser.contexts()[0];
const page = ctx.pages()[0] || await ctx.newPage();
await page.goto("https://example.com");
await browser.close();
await fetch(`${API}/api/v1/browser/stop?id=${id}&token=${TOKEN}`);
```

### 4.4 Selenium(Python,用 `debuggerAddress`)
```python
import requests
from selenium import webdriver
from selenium.webdriver.chrome.options import Options

API, TOKEN, pid = "http://127.0.0.1:50327", "<token>", "<profileId>"
data = requests.get(f"{API}/api/v1/browser/start", params={"id": pid, "token": TOKEN}).json()["data"]
opts = Options()
opts.add_experimental_option("debuggerAddress", data["debuggerAddress"])
driver = webdriver.Chrome(options=opts)        # 需与内核匹配的 chromedriver(随 CloakBrowser 附带)
driver.get("https://example.com")
print(driver.title)
driver.quit()
requests.get(f"{API}/api/v1/browser/stop", params={"id": pid, "token": TOKEN})
```
> CloakBrowser 包内自带匹配的 `chromedriver`(与 `chrome` 同目录)。

---

## 5. 验证指纹是否生效

启动后,取 `debugPort`,请求 `http://127.0.0.1:<port>/json/version`:若环境 `platform=windows`,在 **Linux 服务器**上 `User-Agent` 也会显示 `Windows NT 10.0` —— 说明源码级指纹补丁生效(已实测)。更全面可用上面脚本打开 `browserleaks.com` / `pixelscan.net` 比对。

---

## 6. 有头模式(Linux 服务器上跑「带界面」的浏览器)

服务器无物理显示器,用虚拟显示 Xvfb 即可跑有头:
```bash
apt-get install -y xvfb
# 让整个无头服务在虚拟显示下运行(之后 start 时传 headless=0 即为有头)
xvfb-run -a /usr/local/bin/cloak-headless
# 或给 systemd 服务加: ExecStart=/usr/bin/xvfb-run -a /usr/local/bin/cloak-headless
```
然后 `GET /api/v1/browser/start?id=<id>&headless=0&token=<token>` 即在 Xvfb 中启动有头浏览器(可配合截图/录屏)。Windows 桌面/服务器直接 `headless=0` 即可弹窗。

---

## 7. 注意事项

- **Token**:每次服务重启会重新生成;脚本/MCP 配置需同步更新。
- **开源免费**:无 License/试用/数量限制,环境数量不受限。
- **代理/时区**:环境可绑定代理,启动时按代理自动回填出口 IP / 国家(GeoIP),Linux 同样生效。
- **远程暴露**:服务仅监听 `127.0.0.1`,跨机请用 SSH 隧道或反向代理 + 鉴权,勿裸暴露端口。
- **浏览器路径**:设 `CLOAKBROWSER_BINARY_PATH` 指向 CloakBrowser 的 `chrome`;或放在 `cloakbrowser-<os>-x64/` 约定目录由程序自动探测。
