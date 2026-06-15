// Shared TypeScript models mirroring the Rust backend (camelCase).

export interface AppSettings {
  browserPath: string;
  dataRoot: string;
}

export interface ProjectInfo {
  id: string;
  name: string;
}

export interface BrowserProfile {
  id: string;
  projectId: string;
  name: string;
  note: string;
  proxy: string;
  proxyIp: string;
  proxyCountry: string;
  locale: string;
  timezone: string;
  platform: string;
  seed: string;
  gpuVendor: string;
  gpuRenderer: string;
  brand: string;
  brandVersion: string;
  platformVersion: string;
  location: string;
  webrtcIp: string;
  userAgent: string;
  webrtcPolicy: string;
  geolocation: string;
  canvasNoise: string;
  webglNoise: string;
  audioNoise: string;
  fonts: string;
  speechVoices: string;
  doNotTrack: boolean;
  cookiesJson: string;
  chromeVersion: string;
  uaFullVersion: string;
  secChUa: string;
  homepage: string;
  proxyMode: string;
  proxyProtocol: string;
  proxyHost: string;
  proxyPort: string;
  proxyUsername: string;
  proxyPassword: string;
  proxyApi: string;
  proxyPoolId: string;
  locationPolicy: string;
  geolocationPrecision: string;
  screenMode: string;
  fontMode: string;
  webglImage: string;
  clientRects: string;
  deviceName: string;
  macAddress: string;
  sslMode: string;
  sslDisabled: string;
  portScanMode: string;
  portWhitelist: string;
  gpuEnabled: boolean;
  mediaDevices: string;
  injectionScripts: string[];
  oneShotInjectionScripts: string[];
  extraArgs: string[];
  tags: string[];
  createdAt: string;
  updatedAt: string;
  lastLaunchedAt: string;
  hardwareConcurrency: number;
  deviceMemory: number;
  screenWidth: number;
  screenHeight: number;
  storageQuota: number;
  taskbarHeight: number;
  debugPort: number;
  noiseEnabled: boolean;
  autoLocaleTimezone: boolean;
  autoLocale: boolean;
  autoTimezone: boolean;
}

export interface ProxyEntry {
  id: string;
  name: string;
  protocol: string;
  host: string;
  port: string;
  username: string;
  password: string;
  url: string;
  lastIp: string;
  lastCountry: string;
  lastCheckedAt: string;
  status: string;
  createdAt: string;
}

export interface RunningEndpoint {
  profileId: string;
  debugPort: number;
  wsEndpoint: string;
  httpEndpoint: string;
}

export interface ControllerState {
  settings: AppSettings;
  projects: ProjectInfo[];
  activeProjectId: string;
  profiles: BrowserProfile[];
  proxies: ProxyEntry[];
  runningIds: string[];
  endpoints: Record<string, RunningEndpoint>;
  apiPort: number;
}

export interface LaunchResult {
  profile: BrowserProfile;
  commandPreview: string;
  proxyLookupPending?: boolean;
  proxyLookupError?: string;
  endpoint?: RunningEndpoint;
}

export interface FingerprintAudit {
  score: number;
  issues: string[];
  warnings: string[];
}

export interface AutomationInfo {
  apiPort: number;
  apiBase: string;
  token: string;
  endpoints: Record<string, RunningEndpoint>;
}

export interface ProxyCheckResult {
  ip: string;
  countryCode: string;
  locale: string;
  timezone: string;
  proxy: string;
}

export interface SecurityStatus {
  hasMasterPassword: boolean;
  locked: boolean;
}

export type ProxyApplyTarget = "proxy" | "locale" | "timezone";

export type ModalMode =
  | "profile"
  | "settings"
  | "automation"
  | "batchCreate"
  | "batchMove"
  | "about"
  | "proxyPool"
  | "security"
  | null;

export type ContextMenuState =
  | { kind: "project" | "profile"; id: string; x: number; y: number }
  | null;

export type TableColumnKey =
  | "select"
  | "status"
  | "browser"
  | "proxy"
  | "region"
  | "fingerprint"
  | "scripts"
  | "last";
