// Lightweight i18n: a module-level language store exposed via useSyncExternalStore
// so any component re-renders on language change without a context provider.

import { useSyncExternalStore } from "react";

export type Lang = string;

/** Languages offered in the UI selector. zh/en are fully translated; the rest
 *  localize the high-traffic surfaces and fall back to English elsewhere. */
export const LANGS: { code: string; name: string }[] = [
  { code: "zh", name: "中文" },
  { code: "en", name: "English" },
  { code: "ja", name: "日本語" },
  { code: "ko", name: "한국어" },
  { code: "ru", name: "Русский" },
  { code: "es", name: "Español" },
  { code: "fr", name: "Français" },
  { code: "de", name: "Deutsch" },
];
const SUPPORTED = new Set(LANGS.map((l) => l.code));

const STORAGE_KEY = "cloak-lang";

function initialLang(): Lang {
  if (typeof localStorage !== "undefined") {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved && SUPPORTED.has(saved)) return saved;
  }
  return "zh";
}

let current: Lang = initialLang();
const listeners = new Set<() => void>();

export function setLang(lang: Lang) {
  current = SUPPORTED.has(lang) ? lang : "en";
  if (typeof localStorage !== "undefined") localStorage.setItem(STORAGE_KEY, current);
  listeners.forEach((cb) => cb());
}

function subscribe(cb: () => void) {
  listeners.add(cb);
  return () => listeners.delete(cb);
}

type Entry = { zh: string; en: string };

const dict: Record<string, Entry> = {
  "app.subtitle": { zh: "阿笨猫指纹浏览器", en: "Abencat Fingerprint Browser" },
  "nav.dashboard": { zh: "仪表盘", en: "Dashboard" },
  "nav.browsers": { zh: "浏览器", en: "Browsers" },
  "nav.proxy": { zh: "代理池", en: "Proxy Pool" },
  "nav.automation": { zh: "自动化", en: "Automation" },
  "nav.settings": { zh: "设置", en: "Settings" },
  "nav.security": { zh: "安全", en: "Security" },
  "projects.title": { zh: "项目分组", en: "Projects" },
  "projects.add": { zh: "+ 新增", en: "+ Add" },
  "projects.rename": { zh: "改名", en: "Rename" },
  "projects.delete": { zh: "删除", en: "Delete" },
  "about": { zh: "关于", en: "About" },
  "header.browsers": { zh: "浏览器", en: "Browsers" },
  "header.browserPath": { zh: "浏览器路径", en: "Browser Path" },
  "stats.total": { zh: "浏览器总数", en: "Total Browsers" },
  "stats.currentProject": { zh: "当前项目", en: "Current project" },
  "stats.running": { zh: "运行中", en: "Running" },
  "stats.scripts": { zh: "注入脚本", en: "Injected Scripts" },
  "stats.scriptsFoot": { zh: "随浏览器启动加载", en: "Loaded on launch" },
  "stats.projects": { zh: "项目数", en: "Projects" },
  "stats.switchLeft": { zh: "左侧列表切换", en: "Switch on the left" },
  "action.new": { zh: "新浏览器", en: "New Browser" },
  "action.start": { zh: "启动", en: "Start" },
  "action.stop": { zh: "停止", en: "Stop" },
  "action.edit": { zh: "编辑", en: "Edit" },
  "action.clone": { zh: "克隆", en: "Clone" },
  "action.moveTo": { zh: "移到项目", en: "Move to" },
  "action.search": { zh: "搜索浏览器、代理、语言...", en: "Search browsers, proxy, locale..." },
  "action.import": { zh: "导入浏览器", en: "Import" },
  "action.loadScripts": { zh: "载入脚本", en: "Load Scripts" },
  "action.delete": { zh: "删除", en: "Delete" },
  "batch.selected": { zh: "已选择", en: "Selected" },
  "batch.units": { zh: "个浏览器", en: "browsers" },
  "batch.launch": { zh: "批量启动", en: "Batch Start" },
  "batch.stop": { zh: "批量关闭", en: "Batch Stop" },
  "batch.loadScripts": { zh: "批量载入脚本", en: "Batch Load Scripts" },
  "batch.clearScripts": { zh: "批量清除脚本", en: "Batch Clear Scripts" },
  "batch.move": { zh: "批量移到项目", en: "Batch Move" },
  "batch.create": { zh: "批量创建", en: "Batch Create" },
  "batch.delete": { zh: "批量删除", en: "Batch Delete" },
  "col.status": { zh: "状态", en: "Status" },
  "col.browser": { zh: "浏览器", en: "Browser" },
  "col.proxy": { zh: "代理", en: "Proxy" },
  "col.region": { zh: "地区", en: "Region" },
  "col.fingerprint": { zh: "设备指纹", en: "Fingerprint" },
  "col.scripts": { zh: "脚本", en: "Scripts" },
  "col.last": { zh: "最后启动", en: "Last Launch" },
  "common.cancel": { zh: "取消", en: "Cancel" },
  "common.save": { zh: "保存", en: "Save" },
  "common.ok": { zh: "知道了", en: "Got it" },
  "common.done": { zh: "完成", en: "Done" },
  "common.refresh": { zh: "刷新", en: "Refresh" },
  "common.copy": { zh: "复制", en: "Copy" },
  "ready": { zh: "系统已就绪", en: "System ready" },
  "empty.noEnv": { zh: "No more environments", en: "No more environments" },
  "loading": { zh: "正在读取配置...", en: "Loading…" },
  // Security / master password
  "sec.title": { zh: "安全 · 主密码", en: "Security · Master Password" },
  "sec.mismatch": { zh: "两次输入不一致", en: "Passwords do not match" },
  "sec.master": { zh: "主密码", en: "Master Password" },
  "sec.masterOn": { zh: "已启用：启动需解锁，敏感数据用主密码加密。", en: "Enabled: unlock on start; secrets encrypted with your password." },
  "sec.masterOff": { zh: "未启用：使用机器绑定密钥加密（零摩擦）。", en: "Disabled: machine-bound key encryption (no prompt)." },
  "sec.setMaster": { zh: "设置主密码", en: "Set Master Password" },
  "sec.changeMaster": { zh: "修改主密码", en: "Change Password" },
  "sec.removeMaster": { zh: "移除主密码", en: "Remove Password" },
  "sec.newPassword": { zh: "新主密码（至少 4 位）", en: "New password (min 4 chars)" },
  "sec.confirmPassword": { zh: "确认密码", en: "Confirm password" },
  "sec.machineId": { zh: "设备 ID", en: "Machine ID" },
  "lock.title": { zh: "已锁定", en: "Locked" },
  "lock.hint": { zh: "请输入主密码以解锁", en: "Enter master password to unlock" },
  "lock.unlock": { zh: "解锁", en: "Unlock" },
  "lock.password": { zh: "主密码", en: "Master password" },
  // License
  "lic.title": { zh: "授权", en: "License" },
  "lic.status": { zh: "授权状态", en: "License Status" },
  "lic.activated": { zh: "已激活", en: "Activated" },
  "lic.inactive": { zh: "未激活（试用）", en: "Not activated (trial)" },
  "lic.invalid": { zh: "授权无效", en: "License invalid" },
  "lic.licensee": { zh: "授权给", en: "Licensed to" },
  "lic.plan": { zh: "套餐", en: "Plan" },
  "lic.expires": { zh: "到期", en: "Expires" },
  "lic.key": { zh: "授权码", en: "License Key" },
  "lic.activate": { zh: "激活", en: "Activate" },
  "lic.deactivate": { zh: "取消激活", en: "Deactivate" },
  "lic.keyPlaceholder": { zh: "粘贴授权码…", en: "Paste license key…" },
  "lic.machineHint": { zh: "购买机器绑定授权时，请把设备 ID 提供给供应商。", en: "For machine-bound licenses, send the Machine ID to your vendor." },
  "trial.banner": { zh: "试用版：环境 {used}/{limit}。激活授权后解除数量限制。", en: "Trial: {used}/{limit} environments. Activate a license to remove the cap." },
  "trial.activate": { zh: "去激活", en: "Activate" },
  // Profile editor
  "editor.new": { zh: "新浏览器", en: "New Browser" },
  "editor.edit": { zh: "编辑浏览器", en: "Edit Browser" },
  "editor.name": { zh: "浏览器名称", en: "Name" },
  "editor.proxy": { zh: "代理", en: "Proxy" },
  "editor.getProxyInfo": { zh: "获取代理信息", en: "Check Proxy" },
  "editor.getting": { zh: "获取中", en: "Checking" },
  "editor.exit": { zh: "出口", en: "Exit" },
  "editor.locale": { zh: "语言", en: "Language" },
  "editor.timezone": { zh: "时区", en: "Timezone" },
  "editor.autoByProxy": { zh: "自动根据代理", en: "Auto from proxy" },
  "editor.get": { zh: "获取", en: "Get" },
  "editor.reset": { zh: "重置", en: "Reset" },
  "editor.commonScreen": { zh: "常用屏幕", en: "Screen preset" },
  "editor.custom": { zh: "自定义", en: "Custom" },
  "editor.width": { zh: "宽度", en: "Width" },
  "editor.height": { zh: "高度", en: "Height" },
  "editor.random": { zh: "随机", en: "Random" },
  "editor.cpu": { zh: "CPU", en: "CPU" },
  "editor.memory": { zh: "内存 GB", en: "Memory GB" },
  "editor.gpu": { zh: "显卡", en: "GPU" },
  "editor.note": { zh: "备注", en: "Note" },
  "editor.notePlaceholder": { zh: "写账号用途、项目备注、登录状态说明等", en: "Account purpose, notes, login state…" },
  "editor.scriptsLabel": { zh: "注入脚本路径，每行一个 .js/.user.js", en: "Injection scripts, one .js/.user.js path per line" },
  "editor.chooseFile": { zh: "选择文件", en: "Choose files" },
  "editor.audit": { zh: "指纹自检", en: "Fingerprint Audit" },
  "editor.runAudit": { zh: "运行自检", en: "Run Audit" },
  "editor.auditing": { zh: "检测中", en: "Auditing" },
  "editor.saveFirst": { zh: "保存后即可运行自检。", en: "Save first to run the audit." },
  "editor.auditOk": { zh: "✓ 未发现明显问题", en: "✓ No obvious issues" },
  "editor.detectionSites": { zh: "检测站（在指纹浏览器内打开）", en: "Detection sites (open in fingerprint browser)" },
  // Settings
  "settings.title": { zh: "控制器设置", en: "Settings" },
  "settings.browserProgram": { zh: "浏览器程序", en: "Browser executable" },
  "settings.choose": { zh: "选择", en: "Browse" },
  "settings.dataDir": { zh: "数据目录", en: "Data directory" },
  "settings.kernel": { zh: "指纹内核 (CloakBrowser)", en: "Fingerprint kernel (CloakBrowser)" },
  "settings.kernelInstalled": { zh: "✓ 已安装", en: "✓ Installed" },
  "settings.kernelMissing": { zh: "未安装", en: "Not installed" },
  "settings.downloadKernel": { zh: "下载浏览器内核", en: "Download kernel" },
  "settings.downloading": { zh: "下载中…(约 200MB)", en: "Downloading… (~200MB)" },
  "msg.kernelReady": { zh: "浏览器内核已就绪", en: "Browser kernel ready" },
  // Batch create / move
  "batchCreate.title": { zh: "批量创建浏览器", en: "Batch Create" },
  "batchCreate.count": { zh: "创建数量", en: "Count" },
  "batchCreate.proxy": { zh: "统一代理", en: "Shared proxy" },
  "batchCreate.autoLocale": { zh: "启动时根据代理自动同步语言", en: "Auto-sync language from proxy on launch" },
  "batchCreate.autoTimezone": { zh: "启动时根据代理自动同步时区", en: "Auto-sync timezone from proxy on launch" },
  "batchCreate.create": { zh: "创建", en: "Create" },
  "batchMove.title": { zh: "批量移到项目", en: "Move to Project" },
  "batchMove.single": { zh: "移动项目", en: "Move Project" },
  "batchMove.target": { zh: "选择目标项目", en: "Target project" },
  // Proxy pool
  "pool.title": { zh: "代理池", en: "Proxy Pool" },
  "pool.protocol": { zh: "协议", en: "Protocol" },
  "pool.host": { zh: "主机", en: "Host" },
  "pool.port": { zh: "端口", en: "Port" },
  "pool.username": { zh: "用户名", en: "Username" },
  "pool.password": { zh: "密码", en: "Password" },
  "pool.add": { zh: "添加到代理池", en: "Add to pool" },
  "pool.bulkLabel": { zh: "批量导入（每行一个：ip:port / ip:port:user:pass / 完整 URL）", en: "Bulk import (one per line: ip:port / ip:port:user:pass / full URL)" },
  "pool.bulkImport": { zh: "批量导入", en: "Bulk Import" },
  "pool.name": { zh: "名称", en: "Name" },
  "pool.exitIp": { zh: "出口 IP", en: "Exit IP" },
  "pool.status": { zh: "状态", en: "Status" },
  "pool.actions": { zh: "操作", en: "Actions" },
  "pool.check": { zh: "检测", en: "Check" },
  "pool.checking": { zh: "检测中", en: "Checking" },
  "pool.bind": { zh: "绑定所选", en: "Bind selected" },
  "pool.delete": { zh: "删除", en: "Delete" },
  "pool.statusOk": { zh: "可用", en: "OK" },
  "pool.statusFail": { zh: "失败", en: "Fail" },
  "pool.statusUnknown": { zh: "未检测", en: "Untested" },
  "pool.empty": { zh: "代理池为空，添加或批量导入代理。", en: "Pool is empty — add or import proxies." },
  "pool.bindHint": { zh: "提示：先在列表勾选浏览器，再点「绑定所选」可把代理批量分配给这些环境。", en: "Tip: select browsers first, then Bind selected to assign this proxy to them." },
  // Automation
  "auto.title": { zh: "本地自动化 API", en: "Local Automation API" },
  "auto.intro": { zh: "启动浏览器时会自动分配 remote-debugging-port，并暴露本地 HTTP API，供 Selenium / Puppeteer / Playwright 对接。仅监听 127.0.0.1，调用受 token 保护。", en: "On launch a remote-debugging-port is auto-assigned and a local HTTP API is exposed for Selenium / Puppeteer / Playwright. Loopback only, token-protected." },
  "auto.apiAddr": { zh: "API 地址", en: "API URL" },
  "auto.token": { zh: "Token", en: "Token" },
  "auto.starting": { zh: "启动中…", en: "starting…" },
  "auto.httpApi": { zh: "HTTP 接口", en: "HTTP Endpoints" },
  "auto.runningEndpoints": { zh: "运行中的端点", en: "Active endpoints" },
  "auto.envCol": { zh: "环境", en: "Environment" },
  "auto.portCol": { zh: "调试端口", en: "Debug port" },
  "auto.discovering": { zh: "(发现中…)", en: "(discovering…)" },
  "auto.noRunning": { zh: "暂无运行中的环境，启动一个浏览器即可看到端点。", en: "No running environments — launch one to see endpoints." },
  "auto.examples": { zh: "对接示例", en: "Integration examples" },
  // About
  "about.title": { zh: "关于", en: "About" },
  "about.body": { zh: "本机指纹浏览器控制器，用于管理项目分组、浏览器配置、脚本注入和浏览器启动。", en: "Local fingerprint-browser controller for managing projects, profiles, script injection and launches." },
  // Common toasts
  "msg.selectFirst": { zh: "请先选择浏览器。", en: "Select a browser first." },
  "msg.copied": { zh: "已复制到剪贴板", en: "Copied to clipboard" },
  "msg.copyFail": { zh: "复制失败，请手动选择文本", en: "Copy failed — select the text manually" },
  "msg.previewOnly": { zh: "当前是前端预览，真实操作请打开 Tauri 应用", en: "Web preview only — open the Tauri app for real actions" },
  // Context menu
  "ctx.switch": { zh: "切换项目", en: "Switch project" },
  "ctx.rename": { zh: "改名", en: "Rename" },
  "ctx.delete": { zh: "删除", en: "Delete" },
  "ctx.start": { zh: "启动", en: "Start" },
  "ctx.stop": { zh: "停止", en: "Stop" },
  "ctx.edit": { zh: "编辑", en: "Edit" },
  "ctx.clone": { zh: "克隆", en: "Clone" },
  "ctx.moveTo": { zh: "移到项目", en: "Move to project" },
  "ctx.loadScripts": { zh: "载入脚本", en: "Load scripts" },
  "ctx.openFolder": { zh: "打开目录", en: "Open folder" },
  "about.list1": { zh: "环境数据按项目保存在 controller-data 下。", en: "Environment data is stored per project under controller-data." },
  "about.list2": { zh: "敏感字段加密落盘，可选主密码解锁。", en: "Secrets are encrypted at rest; optional master-password unlock." },
  "about.list3": { zh: "内置本地自动化 API，项目/浏览器列表支持右键。", en: "Built-in local automation API; right-click projects/browsers for actions." },
};

// Additional languages cover the high-traffic UI (nav, actions, table, stats,
// common buttons, lock). Missing keys fall back to English automatically, so
// these dictionaries can be extended incrementally.
const EXTRA: Record<string, Record<string, string>> = {
  ja: {
    "nav.browsers": "ブラウザ", "nav.proxy": "プロキシ", "nav.automation": "自動化", "nav.settings": "設定", "nav.security": "セキュリティ",
    "action.new": "新規ブラウザ", "action.start": "起動", "action.stop": "停止", "action.edit": "編集", "action.clone": "複製", "action.moveTo": "移動", "action.import": "インポート", "action.loadScripts": "スクリプト読込", "action.delete": "削除", "action.search": "ブラウザ・プロキシ・言語を検索...",
    "common.cancel": "キャンセル", "common.save": "保存", "common.ok": "OK", "common.done": "完了", "common.refresh": "更新", "common.copy": "コピー",
    "col.status": "状態", "col.browser": "ブラウザ", "col.proxy": "プロキシ", "col.region": "地域", "col.fingerprint": "指紋", "col.scripts": "スクリプト", "col.last": "最終起動",
    "stats.total": "ブラウザ総数", "stats.running": "実行中", "stats.scripts": "注入スクリプト", "stats.projects": "プロジェクト", "stats.currentProject": "現在のプロジェクト",
    "batch.launch": "一括起動", "batch.stop": "一括停止", "batch.create": "一括作成", "batch.delete": "一括削除",
    "projects.title": "プロジェクト", "ready": "準備完了", "empty.noEnv": "環境がありません", "loading": "読み込み中…", "about": "情報",
    "lock.title": "ロック中", "lock.unlock": "ロック解除", "lock.password": "マスターパスワード", "header.browsers": "ブラウザ", "header.browserPath": "ブラウザのパス",
  },
  ko: {
    "nav.browsers": "브라우저", "nav.proxy": "프록시", "nav.automation": "자동화", "nav.settings": "설정", "nav.security": "보안",
    "action.new": "새 브라우저", "action.start": "시작", "action.stop": "중지", "action.edit": "편집", "action.clone": "복제", "action.moveTo": "이동", "action.import": "가져오기", "action.loadScripts": "스크립트 로드", "action.delete": "삭제", "action.search": "브라우저·프록시·언어 검색...",
    "common.cancel": "취소", "common.save": "저장", "common.ok": "확인", "common.done": "완료", "common.refresh": "새로고침", "common.copy": "복사",
    "col.status": "상태", "col.browser": "브라우저", "col.proxy": "프록시", "col.region": "지역", "col.fingerprint": "지문", "col.scripts": "스크립트", "col.last": "최근 실행",
    "stats.total": "전체 브라우저", "stats.running": "실행 중", "stats.scripts": "주입 스크립트", "stats.projects": "프로젝트", "stats.currentProject": "현재 프로젝트",
    "batch.launch": "일괄 시작", "batch.stop": "일괄 중지", "batch.create": "일괄 생성", "batch.delete": "일괄 삭제",
    "projects.title": "프로젝트", "ready": "시스템 준비됨", "empty.noEnv": "환경이 없습니다", "loading": "불러오는 중…", "about": "정보",
    "lock.title": "잠김", "lock.unlock": "잠금 해제", "lock.password": "마스터 비밀번호", "header.browsers": "브라우저", "header.browserPath": "브라우저 경로",
  },
  ru: {
    "nav.browsers": "Браузеры", "nav.proxy": "Прокси", "nav.automation": "Автоматизация", "nav.settings": "Настройки", "nav.security": "Безопасность",
    "action.new": "Новый браузер", "action.start": "Запустить", "action.stop": "Стоп", "action.edit": "Изменить", "action.clone": "Клонировать", "action.moveTo": "Переместить", "action.import": "Импорт", "action.loadScripts": "Загрузить скрипты", "action.delete": "Удалить", "action.search": "Поиск браузеров, прокси, языка...",
    "common.cancel": "Отмена", "common.save": "Сохранить", "common.ok": "Понятно", "common.done": "Готово", "common.refresh": "Обновить", "common.copy": "Копировать",
    "col.status": "Статус", "col.browser": "Браузер", "col.proxy": "Прокси", "col.region": "Регион", "col.fingerprint": "Отпечаток", "col.scripts": "Скрипты", "col.last": "Последний запуск",
    "stats.total": "Всего браузеров", "stats.running": "Запущено", "stats.scripts": "Внедрённые скрипты", "stats.projects": "Проекты", "stats.currentProject": "Текущий проект",
    "batch.launch": "Запуск пакетом", "batch.stop": "Стоп пакетом", "batch.create": "Создать пакетом", "batch.delete": "Удалить пакетом",
    "projects.title": "Проекты", "ready": "Система готова", "empty.noEnv": "Нет окружений", "loading": "Загрузка…", "about": "О программе",
    "lock.title": "Заблокировано", "lock.unlock": "Разблокировать", "lock.password": "Мастер-пароль", "header.browsers": "Браузеры", "header.browserPath": "Путь к браузеру",
  },
  es: {
    "nav.browsers": "Navegadores", "nav.proxy": "Proxies", "nav.automation": "Automatización", "nav.settings": "Ajustes", "nav.security": "Seguridad",
    "action.new": "Nuevo navegador", "action.start": "Iniciar", "action.stop": "Detener", "action.edit": "Editar", "action.clone": "Clonar", "action.moveTo": "Mover a", "action.import": "Importar", "action.loadScripts": "Cargar scripts", "action.delete": "Eliminar", "action.search": "Buscar navegadores, proxy, idioma...",
    "common.cancel": "Cancelar", "common.save": "Guardar", "common.ok": "Entendido", "common.done": "Listo", "common.refresh": "Actualizar", "common.copy": "Copiar",
    "col.status": "Estado", "col.browser": "Navegador", "col.proxy": "Proxy", "col.region": "Región", "col.fingerprint": "Huella", "col.scripts": "Scripts", "col.last": "Último inicio",
    "stats.total": "Navegadores totales", "stats.running": "En ejecución", "stats.scripts": "Scripts inyectados", "stats.projects": "Proyectos", "stats.currentProject": "Proyecto actual",
    "batch.launch": "Inicio en lote", "batch.stop": "Parada en lote", "batch.create": "Creación en lote", "batch.delete": "Eliminación en lote",
    "projects.title": "Proyectos", "ready": "Sistema listo", "empty.noEnv": "No hay entornos", "loading": "Cargando…", "about": "Acerca de",
    "lock.title": "Bloqueado", "lock.unlock": "Desbloquear", "lock.password": "Contraseña maestra", "header.browsers": "Navegadores", "header.browserPath": "Ruta del navegador",
  },
  fr: {
    "nav.browsers": "Navigateurs", "nav.proxy": "Proxys", "nav.automation": "Automatisation", "nav.settings": "Paramètres", "nav.security": "Sécurité",
    "action.new": "Nouveau navigateur", "action.start": "Démarrer", "action.stop": "Arrêter", "action.edit": "Modifier", "action.clone": "Cloner", "action.moveTo": "Déplacer", "action.import": "Importer", "action.loadScripts": "Charger scripts", "action.delete": "Supprimer", "action.search": "Rechercher navigateurs, proxy, langue...",
    "common.cancel": "Annuler", "common.save": "Enregistrer", "common.ok": "Compris", "common.done": "Terminé", "common.refresh": "Actualiser", "common.copy": "Copier",
    "col.status": "État", "col.browser": "Navigateur", "col.proxy": "Proxy", "col.region": "Région", "col.fingerprint": "Empreinte", "col.scripts": "Scripts", "col.last": "Dernier lancement",
    "stats.total": "Navigateurs au total", "stats.running": "En cours", "stats.scripts": "Scripts injectés", "stats.projects": "Projets", "stats.currentProject": "Projet actuel",
    "batch.launch": "Lancement par lot", "batch.stop": "Arrêt par lot", "batch.create": "Création par lot", "batch.delete": "Suppression par lot",
    "projects.title": "Projets", "ready": "Système prêt", "empty.noEnv": "Aucun environnement", "loading": "Chargement…", "about": "À propos",
    "lock.title": "Verrouillé", "lock.unlock": "Déverrouiller", "lock.password": "Mot de passe maître", "header.browsers": "Navigateurs", "header.browserPath": "Chemin du navigateur",
  },
  de: {
    "nav.browsers": "Browser", "nav.proxy": "Proxys", "nav.automation": "Automatisierung", "nav.settings": "Einstellungen", "nav.security": "Sicherheit",
    "action.new": "Neuer Browser", "action.start": "Starten", "action.stop": "Stoppen", "action.edit": "Bearbeiten", "action.clone": "Klonen", "action.moveTo": "Verschieben", "action.import": "Importieren", "action.loadScripts": "Skripte laden", "action.delete": "Löschen", "action.search": "Browser, Proxy, Sprache suchen...",
    "common.cancel": "Abbrechen", "common.save": "Speichern", "common.ok": "Verstanden", "common.done": "Fertig", "common.refresh": "Aktualisieren", "common.copy": "Kopieren",
    "col.status": "Status", "col.browser": "Browser", "col.proxy": "Proxy", "col.region": "Region", "col.fingerprint": "Fingerabdruck", "col.scripts": "Skripte", "col.last": "Letzter Start",
    "stats.total": "Browser gesamt", "stats.running": "Läuft", "stats.scripts": "Injizierte Skripte", "stats.projects": "Projekte", "stats.currentProject": "Aktuelles Projekt",
    "batch.launch": "Stapel-Start", "batch.stop": "Stapel-Stopp", "batch.create": "Stapel-Erstellen", "batch.delete": "Stapel-Löschen",
    "projects.title": "Projekte", "ready": "System bereit", "empty.noEnv": "Keine Umgebungen", "loading": "Lädt…", "about": "Über",
    "lock.title": "Gesperrt", "lock.unlock": "Entsperren", "lock.password": "Master-Passwort", "header.browsers": "Browser", "header.browserPath": "Browser-Pfad",
  },
};

function translate(lang: Lang, key: string, vars?: Record<string, string | number>): string {
  const entry = dict[key];
  let text: string;
  if (lang === "zh" || lang === "en") {
    text = entry ? entry[lang] : key;
  } else {
    // Extra language → English fallback → key.
    text = EXTRA[lang]?.[key] ?? (entry ? entry.en : undefined) ?? key;
  }
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      text = text.replace(new RegExp(`\\{${k}\\}`, "g"), String(v));
    }
  }
  return text;
}

export function useI18n() {
  const lang = useSyncExternalStore(subscribe, () => current, () => current);
  return {
    lang,
    setLang,
    t: (key: string, vars?: Record<string, string | number>) => translate(lang, key, vars),
  };
}
