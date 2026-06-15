# 商用化能力说明

本版本在原有指纹浏览器控制器基础上，补齐了四项商用核心能力，并将后端 `lib.rs`
拆分为模块化结构（`models / util / crypto / store / profile / inject / proxy /
fingerprint / launch / api / commands`）。

## 1. 本地自动化 API

启动任意环境时，会自动分配空闲的 `--remote-debugging-port`，并通过内置 HTTP 服务
暴露给 Selenium / Puppeteer / Playwright。

- 监听地址：`http://127.0.0.1:<port>`（默认尝试 `50327`，被占用时回退到随机端口）
- 仅绑定回环地址，写操作需带 `token`（应用内「自动化」面板可查看与复制）

| 方法/路径 | 说明 |
| --- | --- |
| `GET /api/v1/status` | 服务状态与运行中的环境 id |
| `GET /api/v1/profiles` | 列出全部环境 |
| `GET /api/v1/browser/start?id=<id>&token=<token>` | 启动环境，返回 `ws` / `debugPort` / `debuggerAddress` |
| `GET /api/v1/browser/stop?id=<id>&token=<token>` | 关闭环境 |
| `GET /api/v1/browser/active` | 当前运行中的端点 |

启动返回示例：

```json
{ "code": 0, "data": {
  "id": "xxx",
  "ws": "ws://127.0.0.1:51234/devtools/browser/....",
  "debugPort": 51234,
  "debuggerAddress": "127.0.0.1:51234"
}}
```

Selenium 用 `debuggerAddress` 接管，Puppeteer/Playwright 用 `ws` 连接。

## 2. 数据加密与安全（at-rest）

- 敏感字段（代理、代理账号/密码、Cookie，代理池条目的账号/密码/URL）以
  **AES-256-GCM** 加密落盘，格式 `enc:v1:<base64(nonce|ciphertext)>`。
- 密钥为随机 256-bit，保存在数据目录下的 `.cloak_key`，**不随配置文件外泄**——
  把 `profiles.json` 拷到其他机器无法解出明文。
- 兼容旧的明文存档：读取时若无 `enc:` 前缀按明文处理，下次保存自动加密。
- 内存中始终是明文，业务代码无感知（`store.rs` 在 load/save 处透明加解密）。

## 3. 代理池管理

- 独立代理列表（侧栏「代理池」）：单条新增、按行批量导入
  （`ip:port` / `ip:port:user:pass` / 完整 URL）。
- 一键「检测」：经该代理请求 GeoIP，回写出口 IP / 国家 / 可用状态。
- 「绑定所选」：把代理批量分配给列表中勾选的环境。

## 4. 指纹质量与检测

- 环境编辑器内「指纹自检」：对一致性做启发式打分（0–100）并列出问题/警告，
  覆盖 seed 缺失、噪声关闭、WebRTC 泄露、代理与语言/时区不匹配、CPU/内存/分辨率
  反常、GPU 厂商与渲染器矛盾等。
- 「检测站」按钮：在**当前指纹浏览器内**直接打开 BrowserLeaks / Pixelscan /
  CreepJS / AmIUnique / WebRTC 泄露检测（经 `launch_profile_at` 单次跳转，不改主页）。

## 5. 主密码解锁层（合规可选）

- 默认仍是机器绑定密钥（零摩擦，无需输入）。
- 在「安全」面板可设置**主密码**：密钥改由 Argon2id 从密码派生，敏感数据用
  它加密；启动时进入**解锁界面**，输入正确密码才能进入主界面。
- 设置/修改/移除主密码会自动用新密钥重新加密整个存档。
- 锁定状态下后端 `get_state` 不触碰加密存档，避免误清空密文（关键安全保护）。
- 相关命令：`get_security_status` / `unlock` / `set_master_password` /
  `remove_master_password`。

> 本项目为**开源免费**，无 License/收费/试用限制（早期的 Ed25519 授权与试用额度已移除）。

## 7. 前端组件化

`App.tsx` 由 1756 行降到 ~1300 行，公共部分拆出：
`src/types.ts`（类型）、`src/constants.ts`（选项/预设/纯函数）、
`src/i18n.ts`（多语言）、`src/components/ui.tsx`（Modal/字段/CopyRow）、
`src/components/LockScreen.tsx`、`src/components/SecurityModal.tsx`。

## 8. 国际化（中 / 英）

- `src/i18n.ts` 基于 `useSyncExternalStore` 的轻量方案，无需 Provider；语言持久化到
  `localStorage`，左下角一键切换。
- **多语言**：中/英全量 + 日/韩/俄/西/法/德 核心界面(缺失键回退英文),顶部下拉切换。
- 覆盖:侧栏导航、项目区、统计卡、操作栏、批量栏、表头、页脚、解锁界面、安全弹窗、
  编辑器全部字段、设置、批量创建/移动、代理池、自动化、关于、右键菜单、常用提示语。

## 9. 跨平台与无头服务版本

后端核心与 Tauri 解耦，桌面 GUI 置于 `gui` cargo feature 之后。可编译两种产物：
桌面 GUI（Win/Linux/macOS）与 **`cloak-headless` 无头服务**（不依赖 Tauri/webkit，
适合 Ubuntu Server，浏览器以 `--headless=new` 启动，纯 HTTP API 驱动，新增
`/api/v1/profile/create|delete`）。浏览器探测、curl、路径处理均按平台适配。
详见 `LINUX_HEADLESS.md`。

## 构建

```powershell
# Windows 桌面版
$env:Path = "C:\Users\dachuan\.cargo\bin;$env:Path"
cd src-tauri; cargo build                       # 后端(桌面版)
cd ..; .\node_modules\.bin\tsc.cmd --noEmit; .\node_modules\.bin\vite.cmd build
npm run tauri:build                            # NSIS 安装包
```

```bash
# Ubuntu 无头服务（仅需 Rust，无需 webkit）
cargo build --release --bin cloak-headless --no-default-features
./target/release/cloak-headless
```
