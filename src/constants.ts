// Static option data, default/preview state, and small pure helpers.

import type {
  BrowserProfile,
  ControllerState,
  ProxyEntry,
  TableColumnKey,
} from "./types";

export const tableColumnDefaults: Record<TableColumnKey, number> = {
  select: 40,
  status: 58,
  browser: 170,
  proxy: 160,
  region: 112,
  fingerprint: 178,
  scripts: 56,
  last: 96,
};

export const tableColumnMins: Record<TableColumnKey, number> = {
  select: 38,
  status: 58,
  browser: 136,
  proxy: 150,
  region: 108,
  fingerprint: 156,
  scripts: 56,
  last: 96,
};

export const isTauriRuntime = () =>
  typeof window !== "undefined" &&
  "__TAURI_INTERNALS__" in (window as Window & { __TAURI_INTERNALS__?: unknown });

export const emptyState: ControllerState = {
  settings: { browserPath: "", dataRoot: "" },
  projects: [],
  activeProjectId: "default",
  profiles: [],
  proxies: [],
  runningIds: [],
  endpoints: {},
  apiPort: 0,
};

export const emptyProxyEntry: ProxyEntry = {
  id: "",
  name: "",
  protocol: "SOCKS5",
  host: "",
  port: "",
  username: "",
  password: "",
  url: "",
  lastIp: "",
  lastCountry: "",
  lastCheckedAt: "",
  status: "unknown",
  createdAt: "",
};

export const detectionSites = [
  { label: "BrowserLeaks", url: "https://browserleaks.com/" },
  { label: "Pixelscan", url: "https://pixelscan.net/" },
  { label: "CreepJS", url: "https://abrahamjuliot.github.io/creepjs/" },
  { label: "AmIUnique", url: "https://amiunique.org/fingerprint" },
  { label: "WebRTC", url: "https://browserleaks.com/webrtc" },
];

export const screenPresets = [
  "1280x720",
  "1280x800",
  "1366x768",
  "1440x900",
  "1536x864",
  "1600x900",
  "1680x1050",
  "1920x1080",
  "1920x1200",
  "2048x1152",
  "2256x1504",
  "2560x1080",
  "2560x1440",
  "2560x1600",
  "2880x1800",
  "3000x2000",
  "3200x1800",
  "3440x1440",
  "3840x2160",
];

export const localeOptions = ["zh-CN", "zh-TW", "en-US", "en-GB", "ja-JP", "ko-KR", "de-DE", "fr-FR", "es-ES", "ru-RU", "pt-BR"];

export const timezoneOptions = [
  "Asia/Shanghai",
  "Asia/Taipei",
  "Asia/Tokyo",
  "Asia/Seoul",
  "America/New_York",
  "America/Los_Angeles",
  "America/Chicago",
  "Europe/London",
  "Europe/Berlin",
  "Europe/Paris",
  "Europe/Madrid",
  "Europe/Moscow",
  "Australia/Sydney",
];

export const cpuOptions = [4, 6, 8, 10, 12, 16];
export const memoryOptions = [4, 8, 12, 16, 24, 32];
export const gpuOptions = [
  "Intel HD Graphics 4000",
  "Intel UHD Graphics 620",
  "Intel UHD Graphics 630",
  "Intel Iris Xe Graphics",
  "NVIDIA GeForce GTX 1050",
  "NVIDIA GeForce GTX 1650",
  "NVIDIA GeForce RTX 2060",
  "NVIDIA GeForce RTX 3060",
  "NVIDIA GeForce RTX 4060",
  "AMD Radeon RX 560",
  "AMD Radeon RX 580",
  "AMD Radeon RX 6600",
  "AMD Radeon RX 7600",
  "AMD Radeon Vega 8",
];

export const pick = <T,>(values: T[]) => values[Math.floor(Math.random() * values.length)];
export const randomSeed = () => String(Math.floor(10000 + Math.random() * 89999));
export const splitLines = (value: string) =>
  value.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
export const listText = (value: string[]) => value.join("\n");

export const makePreviewProfile = (overrides: Partial<BrowserProfile>): BrowserProfile => ({
  id: "preview",
  projectId: "default",
  name: "默认",
  note: "主环境",
  proxy: "",
  proxyIp: "",
  proxyCountry: "",
  locale: "zh-CN",
  timezone: "Asia/Shanghai",
  platform: "windows",
  seed: "74100",
  gpuVendor: "AMD",
  gpuRenderer: "AMD Radeon RX 580",
  brand: "Chrome",
  brandVersion: "",
  platformVersion: "10.0.22631",
  location: "",
  webrtcIp: "",
  userAgent: "",
  webrtcPolicy: "protect",
  geolocation: "",
  canvasNoise: "noise",
  webglNoise: "noise",
  audioNoise: "noise",
  fonts: "",
  speechVoices: "",
  doNotTrack: false,
  cookiesJson: "",
  chromeVersion: "124",
  uaFullVersion: "",
  secChUa: "",
  homepage: "",
  proxyMode: "custom",
  proxyProtocol: "SOCKS5",
  proxyHost: "",
  proxyPort: "",
  proxyUsername: "",
  proxyPassword: "",
  proxyApi: "",
  proxyPoolId: "",
  locationPolicy: "ask",
  geolocationPrecision: "100",
  screenMode: "random",
  fontMode: "random",
  webglImage: "random",
  clientRects: "noise",
  deviceName: "DESKTOP-7F2A91C0",
  macAddress: "A4:83:E7:11:2C:98",
  sslMode: "enable",
  sslDisabled: "",
  portScanMode: "protect",
  portWhitelist: "",
  gpuEnabled: true,
  mediaDevices: "random",
  injectionScripts: [],
  oneShotInjectionScripts: [],
  extraArgs: [],
  tags: [],
  createdAt: "2026-05-25T14:23:26Z",
  updatedAt: "2026-05-27T09:12:53Z",
  lastLaunchedAt: "",
  hardwareConcurrency: 4,
  deviceMemory: 4,
  screenWidth: 1920,
  screenHeight: 1080,
  storageQuota: 0,
  taskbarHeight: 48,
  debugPort: 0,
  noiseEnabled: true,
  autoLocaleTimezone: false,
  autoLocale: false,
  autoTimezone: false,
  ...overrides,
});

export const previewState: ControllerState = {
  settings: {
    browserPath: "E:\\newpro\\newchrome\\cloakbrowser-windows-x64\\chrome.exe",
    dataRoot: "E:\\newpro\\newchrome\\FingerprintController\\build-msvc2019\\release\\controller-data",
  },
  projects: [
    { id: "default", name: "默认项目" },
    { id: "ecommerce", name: "电商-test" },
    { id: "payment", name: "支付-test" },
    { id: "search", name: "搜索-test" },
  ],
  activeProjectId: "default",
  proxies: [],
  runningIds: [],
  endpoints: {},
  apiPort: 50327,
  profiles: [
    makePreviewProfile({ id: "demo-default", name: "默认", note: "主环境", seed: "74100" }),
    makePreviewProfile({ id: "demo-bili-1", name: "b站小号1-test", note: "测试账号 1", seed: "79459", hardwareConcurrency: 6, deviceMemory: 24, lastLaunchedAt: "2026-05-27T09:11:00Z" }),
    makePreviewProfile({ id: "demo-bili-2", name: "b站小号2-test", note: "测试账号 2", seed: "52560", hardwareConcurrency: 6, deviceMemory: 24, lastLaunchedAt: "2026-05-27T09:10:00Z" }),
  ],
};

// Pure display helpers.
export const formatDate = (value: string) => {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "-";
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(date.getMonth() + 1)}/${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
};

export const normalizedCountry = (value: string | undefined) => {
  const text = (value || "").trim();
  return /^[a-z]{2}$/i.test(text) ? text.toUpperCase() : "";
};

export const regionCode = (profile: BrowserProfile) =>
  normalizedCountry(profile.proxyCountry) ||
  normalizedCountry(profile.location) ||
  (profile.locale.split("-")[1] || "CN").slice(0, 2).toUpperCase();

export const proxySummary = (profile: BrowserProfile) =>
  [normalizedCountry(profile.proxyCountry), profile.proxyIp || profile.webrtcIp].filter(Boolean).join(" · ");

export const autoLocale = (profile: BrowserProfile) => profile.autoLocale ?? profile.autoLocaleTimezone ?? false;
export const autoTimezone = (profile: BrowserProfile) => profile.autoTimezone ?? profile.autoLocaleTimezone ?? false;

export const proxyLocationLine = (profile: BrowserProfile) =>
  ["获取IP中", "获取失败"].includes(profile.proxyIp)
    ? profile.proxyIp
    : `${normalizedCountry(profile.proxyCountry) || normalizedCountry(profile.location) || "无"}: ${profile.proxyIp || profile.webrtcIp || "无"}`;
