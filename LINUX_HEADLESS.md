# Linux / Ubuntu 与无头(headless)服务版本

后端核心(store / launch / proxy / api / crypto / license / …)已与 Tauri 解耦，
通过 `gui` feature 隔离桌面 GUI。因此可以编译出两种产物：

| 产物 | 说明 | 依赖 |
| --- | --- | --- |
| 桌面 GUI (`cloak-fingerprint-controller`) | Windows/Linux/macOS 带界面版 | Tauri + webkit |
| 无头服务 (`cloak-headless`) | 纯自动化 API 服务，浏览器以 `--headless=new` 启动 | **不依赖 Tauri/webkit** |

## 跨平台改动

- **浏览器探测**：按平台查找 bundle 目录与系统浏览器：
  - Windows：`cloakbrowser-windows-x64/chrome.exe`
  - Linux：`cloakbrowser-linux-x64/chrome`，回退 `/usr/bin/google-chrome(-stable)`、
    `/usr/bin/chromium(-browser)`、`/snap/bin/chromium`
  - macOS：`Google Chrome.app` / `Chromium.app`
  - 始终优先环境变量 `CLOAKBROWSER_BINARY_PATH`
- **路径处理**：`clean_path_text` 仅在 Windows 转 `\`，POSIX 路径保持原样。
- **GeoIP curl**：Windows 用 `curl.exe`，其余用 `curl`。
- **无头启动参数**：`--headless=new --disable-dev-shm-usage`，Linux 追加 `--no-sandbox`。

## 构建无头版（Ubuntu Server，无需图形栈）

```bash
# 仅需 Rust 工具链，无需 webkit2gtk
cargo build --release --bin cloak-headless --no-default-features
./target/release/cloak-headless
```

启动后打印 API 地址与 token：

```
Cloak headless automation API ready
  endpoint : http://127.0.0.1:50327
  token    : <token>
  browser  : /usr/bin/google-chrome
  dataRoot : .../controller-data
```

配置浏览器路径/数据目录：设置环境变量 `CLOAKBROWSER_BINARY_PATH`，或把桌面版生成的
`controller-data/settings.json` 放到运行目录（`default_data_root` 会读取）。

## 无头 API（在桌面版基础上新增）

写操作需 `&token=<token>`：

| 路径 | 说明 |
| --- | --- |
| `GET /api/v1/status` | 状态（含 `headless: true`） |
| `GET /api/v1/profiles` | 列出环境 |
| `GET /api/v1/profile/create?name=&projectId=&token=` | 新建环境（受试用额度约束）→ 返回 id |
| `GET /api/v1/profile/delete?id=&token=` | 删除环境 |
| `GET /api/v1/browser/start?id=&url=&token=` | 启动（headless），返回 ws / debugPort |
| `GET /api/v1/browser/stop?id=&token=` | 关闭 |
| `GET /api/v1/browser/active` | 运行中的端点 |

示例（Puppeteer 连接服务器上的无头浏览器）：

```js
const base = "http://SERVER_IP:50327", token = "<token>";
const id = (await (await fetch(`${base}/api/v1/profile/create?name=job1&token=${token}`)).json()).data.id;
const { data } = await (await fetch(`${base}/api/v1/browser/start?id=${id}&token=${token}`)).json();
const browser = await puppeteer.connect({ browserWSEndpoint: data.ws });
```

> 远程访问时务必只在内网/经隧道暴露端口，并妥善保管 token（服务仅监听 127.0.0.1，
> 跨机访问请用 SSH 隧道或反向代理）。

## 桌面 GUI（Linux）

需要 webkit2gtk 等依赖：

```bash
sudo apt install -y libwebkit2gtk-4.1-dev build-essential libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev
npm install && npm run tauri:build      # 默认即 gui feature
```
