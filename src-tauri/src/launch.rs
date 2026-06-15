//! Chromium argument building, process launch/stop and runtime tracking.

use std::{
    collections::HashMap,
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::Mutex,
    time::{Duration, Instant},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::inject::{build_injection_extension, combined_scripts, enabled_scripts, write_profile_cookies};
use crate::models::*;
use crate::profile::profile_dir;
use crate::proxy::{update_profiles_geoip_async, GeoIpLookupTarget};
use crate::store::{load_settings, load_store, save_store};
use crate::util::{clean_path_text, now_iso, quote_preview, CommandResult};

#[cfg(windows)]
use crate::util::CREATE_NO_WINDOW;

/// A live browser process plus the automation endpoint it exposes.
pub struct RunningBrowser {
    pub child: Child,
    pub endpoint: RunningEndpoint,
}

#[derive(Default)]
pub struct RuntimeState {
    pub running: Mutex<HashMap<String, RunningBrowser>>,
    /// Port the embedded automation HTTP API is bound to (0 = not started).
    pub api_port: Mutex<u16>,
    /// Bearer/query token required by mutating automation-API endpoints.
    pub api_token: Mutex<String>,
    /// When true (headless server build), browsers launch with --headless=new.
    pub headless: bool,
}

impl RuntimeState {
    pub fn endpoint_snapshot(&self) -> HashMap<String, RunningEndpoint> {
        self.running
            .lock()
            .unwrap()
            .iter()
            .map(|(id, browser)| (id.clone(), browser.endpoint.clone()))
            .collect()
    }
}

/// Reserve an OS-assigned free TCP port on loopback for remote debugging.
fn free_loopback_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
        .unwrap_or(0)
}

/// Poll the DevTools `/json/version` endpoint to discover the browser-level
/// WebSocket URL that Puppeteer/Playwright connect to.
fn discover_ws_endpoint(port: u16) -> Option<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()
        .ok()?;
    let url = format!("http://127.0.0.1:{port}/json/version");
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(resp) = client.get(&url).send() {
            if let Ok(value) = resp.json::<serde_json::Value>() {
                if let Some(ws) = value.get("webSocketDebuggerUrl").and_then(|v| v.as_str()) {
                    return Some(ws.to_string());
                }
            }
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    None
}

fn add_value_arg(args: &mut Vec<String>, name: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        args.push(format!("--{}={}", name, value));
    }
}

fn add_int_arg(args: &mut Vec<String>, name: &str, value: u32) {
    if value > 0 {
        args.push(format!("--{}={}", name, value));
    }
}

pub fn build_arguments(profile: &BrowserProfile, dir: &Path, headless: bool) -> Vec<String> {
    let mut args = vec![
        format!("--user-data-dir={}", clean_path_text(dir)),
        "--profile-directory=Default".to_string(),
        "--no-first-run".to_string(),
        "--no-default-browser-check".to_string(),
        "--disable-background-mode".to_string(),
    ];
    if headless {
        // Server/no-GUI launch. --no-sandbox + --disable-dev-shm-usage are the
        // common requirements when running Chromium on headless Linux/containers.
        args.push("--headless=new".to_string());
        args.push("--disable-dev-shm-usage".to_string());
        if cfg!(target_os = "linux") {
            args.push("--no-sandbox".to_string());
        }
    }
    add_value_arg(&mut args, "lang", &profile.locale);
    add_value_arg(&mut args, "proxy-server", &profile.proxy);
    add_int_arg(&mut args, "remote-debugging-port", profile.debug_port);

    let scripts = enabled_scripts(&combined_scripts(profile));
    if !scripts.is_empty() {
        args.push(format!(
            "--load-extension={}",
            clean_path_text(&dir.join("CloakInjectedExtension"))
        ));
    }

    add_value_arg(&mut args, "fingerprint", &profile.seed);
    add_value_arg(&mut args, "fingerprint-platform", &profile.platform);
    add_value_arg(&mut args, "fingerprint-locale", &profile.locale);
    add_value_arg(&mut args, "fingerprint-timezone", &profile.timezone);
    add_value_arg(&mut args, "fingerprint-gpu-vendor", &profile.gpu_vendor);
    add_value_arg(&mut args, "fingerprint-gpu-renderer", &profile.gpu_renderer);
    add_value_arg(&mut args, "fingerprint-brand", &profile.brand);
    add_value_arg(&mut args, "fingerprint-brand-version", &profile.brand_version);
    add_value_arg(
        &mut args,
        "fingerprint-platform-version",
        &profile.platform_version,
    );
    add_value_arg(
        &mut args,
        "fingerprint-chrome-version",
        &profile.chrome_version,
    );
    add_value_arg(
        &mut args,
        "fingerprint-ua-full-version",
        &profile.ua_full_version,
    );
    add_value_arg(&mut args, "fingerprint-sec-ch-ua", &profile.sec_ch_ua);
    add_value_arg(&mut args, "fingerprint-location", &profile.location);
    add_value_arg(
        &mut args,
        "fingerprint-location-policy",
        &profile.location_policy,
    );
    add_value_arg(
        &mut args,
        "fingerprint-geolocation-precision",
        &profile.geolocation_precision,
    );
    add_value_arg(&mut args, "fingerprint-webrtc-ip", &profile.webrtc_ip);
    add_value_arg(&mut args, "user-agent", &profile.user_agent);
    add_value_arg(&mut args, "fingerprint-user-agent", &profile.user_agent);
    add_value_arg(&mut args, "fingerprint-webrtc-policy", &profile.webrtc_policy);
    add_value_arg(&mut args, "fingerprint-geolocation", &profile.geolocation);
    add_value_arg(&mut args, "fingerprint-canvas", &profile.canvas_noise);
    add_value_arg(&mut args, "fingerprint-webgl-image", &profile.webgl_image);
    add_value_arg(&mut args, "fingerprint-webgl-noise", &profile.webgl_noise);
    add_value_arg(&mut args, "fingerprint-audio", &profile.audio_noise);
    add_value_arg(&mut args, "fingerprint-client-rects", &profile.client_rects);
    add_value_arg(&mut args, "fingerprint-fonts", &profile.fonts);
    add_value_arg(&mut args, "fingerprint-font-mode", &profile.font_mode);
    add_value_arg(
        &mut args,
        "fingerprint-speech-voices",
        &profile.speech_voices,
    );
    add_value_arg(&mut args, "fingerprint-screen-mode", &profile.screen_mode);
    add_value_arg(&mut args, "fingerprint-device-name", &profile.device_name);
    add_value_arg(&mut args, "fingerprint-mac-address", &profile.mac_address);
    add_value_arg(&mut args, "fingerprint-ssl-mode", &profile.ssl_mode);
    add_value_arg(&mut args, "fingerprint-ssl-disabled", &profile.ssl_disabled);
    add_value_arg(&mut args, "fingerprint-port-scan", &profile.port_scan_mode);
    add_value_arg(
        &mut args,
        "fingerprint-port-whitelist",
        &profile.port_whitelist,
    );
    add_value_arg(&mut args, "fingerprint-media-devices", &profile.media_devices);
    add_int_arg(
        &mut args,
        "fingerprint-hardware-concurrency",
        profile.hardware_concurrency,
    );
    add_int_arg(&mut args, "fingerprint-device-memory", profile.device_memory);
    add_int_arg(&mut args, "fingerprint-screen-width", profile.screen_width);
    add_int_arg(&mut args, "fingerprint-screen-height", profile.screen_height);
    add_int_arg(&mut args, "fingerprint-storage-quota", profile.storage_quota);
    add_int_arg(
        &mut args,
        "fingerprint-taskbar-height",
        profile.taskbar_height,
    );

    if !profile.noise_enabled {
        args.push("--fingerprint-noise=false".to_string());
    }
    if profile.do_not_track {
        args.push("--fingerprint-do-not-track=1".to_string());
    }
    if !profile.gpu_enabled {
        args.push("--disable-gpu".to_string());
    }
    for arg in &profile.extra_args {
        let arg = arg.trim();
        if !arg.is_empty() {
            args.push(arg.to_string());
        }
    }
    if !profile.homepage.trim().is_empty() {
        args.push(profile.homepage.trim().to_string());
    }
    args
}

pub fn command_preview(
    browser_path: &str,
    profile: &BrowserProfile,
    dir: &Path,
    headless: bool,
) -> String {
    let mut parts = vec![quote_preview(browser_path)];
    parts.extend(
        build_arguments(profile, dir, headless)
            .iter()
            .map(|arg| quote_preview(arg)),
    );
    parts.join(" ")
}

fn launch_profile_from_loaded_store(
    state: &RuntimeState,
    browser_path: &Path,
    data_root: &Path,
    store: &mut StoreFile,
    profile_id: &str,
    url_override: Option<&str>,
    headless_override: Option<bool>,
) -> CommandResult<LaunchResult> {
    let headless = headless_override.unwrap_or(state.headless);
    {
        let running = state.running.lock().unwrap();
        if running.contains_key(profile_id) {
            return Err("该环境已经在运行".to_string());
        }
    }

    let index = store
        .profiles
        .iter()
        .position(|profile| profile.id == profile_id)
        .ok_or_else(|| "找不到该环境".to_string())?;
    let mut current_profile = store.profiles[index].clone();

    // Allocate a debugging port so automation tools always have an endpoint.
    let debug_port: u16 = if current_profile.debug_port > 0 {
        current_profile.debug_port as u16
    } else {
        free_loopback_port()
    };
    current_profile.debug_port = debug_port as u32;

    // A one-off URL (e.g. a fingerprint detection site) overrides the homepage
    // for this launch only; the stored profile is left untouched.
    if let Some(url) = url_override {
        if !url.trim().is_empty() {
            current_profile.homepage = url.trim().to_string();
        }
    }

    let dir = profile_dir(data_root, &current_profile.project_id, &current_profile.id);
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    build_injection_extension(
        &combined_scripts(&current_profile),
        &dir.join("CloakInjectedExtension"),
    )?;
    write_profile_cookies(&current_profile, &dir)?;

    let args = build_arguments(&current_profile, &dir, headless);
    let mut command = Command::new(browser_path);
    command.args(args);
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let child = command
        .spawn()
        .map_err(|err| format!("启动浏览器失败: {err}"))?;

    let http_endpoint = format!("http://127.0.0.1:{debug_port}");
    let ws_endpoint = discover_ws_endpoint(debug_port).unwrap_or_default();
    let endpoint = RunningEndpoint {
        profile_id: profile_id.to_string(),
        debug_port,
        ws_endpoint,
        http_endpoint,
    };

    state.running.lock().unwrap().insert(
        profile_id.to_string(),
        RunningBrowser {
            child,
            endpoint: endpoint.clone(),
        },
    );

    let profile = &mut store.profiles[index];
    profile.proxy_ip = "获取IP中".to_string();
    profile.proxy_country.clear();
    profile.location.clear();
    profile.webrtc_ip.clear();
    profile.last_launched_at = now_iso();
    profile.updated_at = now_iso();
    if !profile.one_shot_injection_scripts.is_empty() {
        profile.one_shot_injection_scripts.clear();
    }
    let stored_profile = profile.clone();
    Ok(LaunchResult {
        command_preview: command_preview(
            browser_path.to_string_lossy().as_ref(),
            &current_profile,
            &dir,
            headless,
        ),
        profile: stored_profile,
        proxy_lookup_pending: true,
        proxy_lookup_error: None,
        endpoint: Some(endpoint),
    })
}

pub fn launch_profiles_batch(
    state: &RuntimeState,
    profile_ids: Vec<String>,
    ignore_already_running: bool,
    url_override: Option<String>,
    headless_override: Option<bool>,
) -> CommandResult<Vec<LaunchResult>> {
    let settings = load_settings();
    let browser_path = PathBuf::from(settings.browser_path.trim());
    if !browser_path.is_file() {
        return Err(format!("找不到浏览器文件: {}", clean_path_text(&browser_path)));
    }

    let data_root = PathBuf::from(settings.data_root);
    let mut store = load_store(&data_root);
    let mut results = Vec::new();
    let mut launched_targets = Vec::new();
    let mut first_error = None;

    for profile_id in profile_ids {
        match launch_profile_from_loaded_store(
            state,
            &browser_path,
            &data_root,
            &mut store,
            &profile_id,
            url_override.as_deref(),
            headless_override,
        ) {
            Ok(result) => {
                launched_targets.push(GeoIpLookupTarget {
                    profile_id,
                    proxy: result.profile.proxy.clone(),
                    protocol: result.profile.proxy_protocol.clone(),
                });
                results.push(result);
            }
            Err(err) if ignore_already_running && err.contains("运行") => {}
            Err(err) => {
                first_error = Some(format!("{profile_id}: {err}"));
                break;
            }
        }
    }

    save_store(&data_root, &store)?;
    update_profiles_geoip_async(data_root, launched_targets);
    if let Some(err) = first_error {
        return Err(err);
    }
    Ok(results)
}

pub fn collect_running_ids(state: &RuntimeState) -> Vec<String> {
    let mut running = state.running.lock().unwrap();
    let mut exited = Vec::new();
    for (id, browser) in running.iter_mut() {
        if matches!(browser.child.try_wait(), Ok(Some(_))) {
            exited.push(id.clone());
        }
    }
    for id in exited {
        running.remove(&id);
    }
    running.keys().cloned().collect()
}

pub fn stop_running(state: &RuntimeState, profile_id: &str) {
    let mut running = state.running.lock().unwrap();
    if let Some(mut browser) = running.remove(profile_id) {
        let _ = browser.child.kill();
        let _ = browser.child.wait();
    }
}
