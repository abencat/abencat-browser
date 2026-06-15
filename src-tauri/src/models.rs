//! Serializable data models and their serde defaults.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub browser_path: String,
    pub data_root: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProfile {
    pub id: String,
    pub project_id: String,
    pub name: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub proxy: String,
    #[serde(default)]
    pub proxy_ip: String,
    #[serde(default)]
    pub proxy_country: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    #[serde(default = "default_platform")]
    pub platform: String,
    #[serde(default)]
    pub seed: String,
    #[serde(default)]
    pub gpu_vendor: String,
    #[serde(default)]
    pub gpu_renderer: String,
    #[serde(default = "default_brand")]
    pub brand: String,
    #[serde(default)]
    pub brand_version: String,
    #[serde(default)]
    pub platform_version: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub webrtc_ip: String,
    #[serde(default)]
    pub user_agent: String,
    #[serde(default = "default_webrtc_policy")]
    pub webrtc_policy: String,
    #[serde(default)]
    pub geolocation: String,
    #[serde(default = "default_noise_mode")]
    pub canvas_noise: String,
    #[serde(default = "default_noise_mode")]
    pub webgl_noise: String,
    #[serde(default = "default_noise_mode")]
    pub audio_noise: String,
    #[serde(default)]
    pub fonts: String,
    #[serde(default)]
    pub speech_voices: String,
    #[serde(default)]
    pub do_not_track: bool,
    #[serde(default)]
    pub cookies_json: String,
    #[serde(default)]
    pub chrome_version: String,
    #[serde(default)]
    pub ua_full_version: String,
    #[serde(default)]
    pub sec_ch_ua: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default = "default_proxy_mode")]
    pub proxy_mode: String,
    #[serde(default = "default_proxy_protocol")]
    pub proxy_protocol: String,
    #[serde(default)]
    pub proxy_host: String,
    #[serde(default)]
    pub proxy_port: String,
    #[serde(default)]
    pub proxy_username: String,
    #[serde(default)]
    pub proxy_password: String,
    #[serde(default)]
    pub proxy_api: String,
    /// Optional reference to a shared proxy-pool entry (id). Empty = inline proxy.
    #[serde(default)]
    pub proxy_pool_id: String,
    #[serde(default = "default_location_policy")]
    pub location_policy: String,
    #[serde(default = "default_geolocation_precision")]
    pub geolocation_precision: String,
    #[serde(default = "default_screen_mode")]
    pub screen_mode: String,
    #[serde(default = "default_font_mode")]
    pub font_mode: String,
    #[serde(default = "default_noise_mode")]
    pub webgl_image: String,
    #[serde(default = "default_noise_mode")]
    pub client_rects: String,
    #[serde(default)]
    pub device_name: String,
    #[serde(default)]
    pub mac_address: String,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default)]
    pub ssl_disabled: String,
    #[serde(default = "default_port_scan_mode")]
    pub port_scan_mode: String,
    #[serde(default)]
    pub port_whitelist: String,
    #[serde(default = "default_true")]
    pub gpu_enabled: bool,
    #[serde(default = "default_media_devices")]
    pub media_devices: String,
    #[serde(default)]
    pub injection_scripts: Vec<String>,
    #[serde(default)]
    pub one_shot_injection_scripts: Vec<String>,
    #[serde(default)]
    pub extra_args: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub last_launched_at: String,
    #[serde(default = "default_hardware_concurrency")]
    pub hardware_concurrency: u32,
    #[serde(default = "default_device_memory")]
    pub device_memory: u32,
    #[serde(default = "default_screen_width")]
    pub screen_width: u32,
    #[serde(default = "default_screen_height")]
    pub screen_height: u32,
    #[serde(default)]
    pub storage_quota: u32,
    #[serde(default = "default_taskbar_height")]
    pub taskbar_height: u32,
    #[serde(default)]
    pub debug_port: u32,
    #[serde(default = "default_true")]
    pub noise_enabled: bool,
    #[serde(default)]
    pub auto_locale: bool,
    #[serde(default)]
    pub auto_timezone: bool,
    #[serde(default)]
    pub auto_locale_timezone: bool,
}

/// A reusable proxy entry stored in the shared proxy pool.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyEntry {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_proxy_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    /// Full proxy URL form, kept in sync with the structured fields above.
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub last_ip: String,
    #[serde(default)]
    pub last_country: String,
    #[serde(default)]
    pub last_checked_at: String,
    /// "unknown" | "ok" | "fail"
    #[serde(default = "default_proxy_status")]
    pub status: String,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreFile {
    #[serde(default)]
    pub seeded: bool,
    #[serde(default = "default_projects")]
    pub projects: Vec<ProjectInfo>,
    #[serde(default = "default_active_project_id")]
    pub active_project_id: String,
    #[serde(default)]
    pub profiles: Vec<BrowserProfile>,
    #[serde(default)]
    pub proxies: Vec<ProxyEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerState {
    pub settings: AppSettings,
    pub projects: Vec<ProjectInfo>,
    pub active_project_id: String,
    pub profiles: Vec<BrowserProfile>,
    pub proxies: Vec<ProxyEntry>,
    pub running_ids: Vec<String>,
    /// Map of profile id -> automation endpoint info for running browsers.
    pub endpoints: std::collections::HashMap<String, RunningEndpoint>,
    pub api_port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningEndpoint {
    pub profile_id: String,
    pub debug_port: u16,
    pub ws_endpoint: String,
    pub http_endpoint: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub browser_path: String,
    pub data_root: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub profile: BrowserProfile,
    pub command_preview: String,
    #[serde(default)]
    pub proxy_lookup_pending: bool,
    pub proxy_lookup_error: Option<String>,
    pub endpoint: Option<RunningEndpoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportExportPayload {
    #[serde(default)]
    pub exported_at: String,
    #[serde(default)]
    pub projects: Vec<ProjectInfo>,
    #[serde(default)]
    pub profiles: Vec<BrowserProfile>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateRequest {
    pub project_id: String,
    pub count: u32,
    pub proxy: String,
    #[serde(default)]
    pub auto_locale: bool,
    #[serde(default)]
    pub auto_timezone: bool,
    #[serde(default)]
    pub auto_locale_timezone: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyCheckResult {
    pub ip: String,
    pub country_code: String,
    pub locale: String,
    pub timezone: String,
    pub proxy: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyApiResult {
    pub proxy: String,
    pub protocol: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
}

/// Result of a local fingerprint self-check / quality score.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FingerprintAudit {
    pub score: u32,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn default_locale() -> String {
    "zh-CN".to_string()
}

pub fn default_timezone() -> String {
    "Asia/Shanghai".to_string()
}

pub fn default_platform() -> String {
    "windows".to_string()
}

pub fn default_brand() -> String {
    "Chrome".to_string()
}

pub fn default_webrtc_policy() -> String {
    "protect".to_string()
}

pub fn default_noise_mode() -> String {
    "noise".to_string()
}

pub fn default_proxy_mode() -> String {
    "custom".to_string()
}

pub fn default_proxy_protocol() -> String {
    "SOCKS5".to_string()
}

pub fn default_proxy_status() -> String {
    "unknown".to_string()
}

pub fn default_location_policy() -> String {
    "ask".to_string()
}

pub fn default_geolocation_precision() -> String {
    "100".to_string()
}

pub fn default_screen_mode() -> String {
    "random".to_string()
}

pub fn default_font_mode() -> String {
    "random".to_string()
}

pub fn default_ssl_mode() -> String {
    "enable".to_string()
}

pub fn default_port_scan_mode() -> String {
    "protect".to_string()
}

pub fn default_media_devices() -> String {
    "random".to_string()
}

pub fn default_hardware_concurrency() -> u32 {
    8
}

pub fn default_device_memory() -> u32 {
    8
}

pub fn default_screen_width() -> u32 {
    1920
}

pub fn default_screen_height() -> u32 {
    1080
}

pub fn default_taskbar_height() -> u32 {
    48
}

pub fn default_true() -> bool {
    true
}

pub fn default_active_project_id() -> String {
    "default".to_string()
}

pub fn default_projects() -> Vec<ProjectInfo> {
    vec![
        ProjectInfo {
            id: "default".to_string(),
            name: "默认项目".to_string(),
        },
        ProjectInfo {
            id: "ecommerce".to_string(),
            name: "电商-test".to_string(),
        },
        ProjectInfo {
            id: "payment".to_string(),
            name: "支付-test".to_string(),
        },
        ProjectInfo {
            id: "search".to_string(),
            name: "搜索-test".to_string(),
        },
    ]
}
