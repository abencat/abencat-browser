//! Tauri command surface. Thin handlers that delegate to the domain modules.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::Duration,
};
use tauri::State;
use uuid::Uuid;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::fingerprint::audit_profile;
use crate::launch::{
    collect_running_ids, command_preview, launch_profiles_batch, stop_running, RuntimeState,
};
use crate::models::*;
use crate::profile::{
    move_profile_data, new_profile, next_clone_name, normalize_profile, profile_dir,
    random_screen_size,
};
use crate::proxy::{parse_proxy_line, parse_proxy_payload, resolve_geoip};
use crate::store::{
    data_root_path, find_profile, find_profile_mut, load_settings, load_store, save_settings_file,
    save_store,
};
use crate::util::{now_iso, CommandResult};

#[cfg(windows)]
use crate::util::CREATE_NO_WINDOW;

fn build_state(state: &State<'_, Arc<RuntimeState>>, store: StoreFile, settings: AppSettings) -> ControllerState {
    ControllerState {
        settings,
        projects: store.projects,
        active_project_id: store.active_project_id,
        profiles: store.profiles,
        proxies: store.proxies,
        running_ids: collect_running_ids(state),
        endpoints: state.endpoint_snapshot(),
        api_port: *state.api_port.lock().unwrap(),
    }
}

#[tauri::command]
pub fn get_state(state: State<'_, Arc<RuntimeState>>) -> ControllerState {
    let settings = load_settings();
    // While locked, never touch the encrypted store — loading would blank the
    // secrets and the save below would persist that loss.
    if crate::crypto::is_locked() {
        return ControllerState {
            settings,
            projects: Vec::new(),
            active_project_id: default_active_project_id(),
            profiles: Vec::new(),
            proxies: Vec::new(),
            running_ids: collect_running_ids(&state),
            endpoints: state.endpoint_snapshot(),
            api_port: *state.api_port.lock().unwrap(),
        };
    }
    let store = load_store(Path::new(&settings.data_root));
    let _ = save_store(Path::new(&settings.data_root), &store);
    build_state(&state, store, settings)
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, Arc<RuntimeState>>,
    patch: SettingsPatch,
) -> CommandResult<ControllerState> {
    let settings = AppSettings {
        browser_path: patch.browser_path.trim().to_string(),
        data_root: patch.data_root.trim().to_string(),
    };
    if settings.data_root.is_empty() {
        return Err("数据目录不能为空".to_string());
    }
    save_settings_file(&settings)?;
    fs::create_dir_all(&settings.data_root).map_err(|err| err.to_string())?;
    let store = load_store(Path::new(&settings.data_root));
    Ok(build_state(&state, store, settings))
}

#[tauri::command]
pub fn create_project(name: String) -> CommandResult<ProjectInfo> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let project = ProjectInfo {
        id: Uuid::new_v4().to_string(),
        name: if name.trim().is_empty() {
            "新项目".to_string()
        } else {
            name.trim().to_string()
        },
    };
    store.active_project_id = project.id.clone();
    store.projects.push(project.clone());
    save_store(&data_root, &store)?;
    Ok(project)
}

#[tauri::command]
pub fn rename_project(project_id: String, name: String) -> CommandResult<()> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let project = store
        .projects
        .iter_mut()
        .find(|project| project.id == project_id)
        .ok_or_else(|| "找不到该项目".to_string())?;
    let name = name.trim();
    if !name.is_empty() {
        project.name = name.to_string();
    }
    save_store(&data_root, &store)
}

#[tauri::command]
pub fn delete_project(project_id: String) -> CommandResult<()> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    if store.projects.len() <= 1 {
        return Err("至少保留一个项目".to_string());
    }
    let index = store
        .projects
        .iter()
        .position(|project| project.id == project_id)
        .ok_or_else(|| "找不到该项目".to_string())?;
    let fallback = store
        .projects
        .iter()
        .find(|project| project.id != project_id)
        .map(|project| project.id.clone())
        .unwrap_or_else(default_active_project_id);

    for profile in store
        .profiles
        .iter()
        .filter(|profile| profile.project_id == project_id)
    {
        move_profile_data(&data_root, &profile.id, &project_id, &fallback)?;
    }
    for profile in &mut store.profiles {
        if profile.project_id == project_id {
            profile.project_id = fallback.clone();
        }
    }
    store.projects.remove(index);
    if store.active_project_id == project_id {
        store.active_project_id = fallback;
    }
    save_store(&data_root, &store)
}

#[tauri::command]
pub fn set_active_project(project_id: String) -> CommandResult<()> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    if !store.projects.iter().any(|project| project.id == project_id) {
        return Err("找不到该项目".to_string());
    }
    store.active_project_id = project_id;
    save_store(&data_root, &store)
}

#[tauri::command]
pub fn create_profile(project_id: String, name: Option<String>) -> CommandResult<BrowserProfile> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let project_id = if store.projects.iter().any(|p| p.id == project_id) {
        project_id
    } else {
        store.active_project_id.clone()
    };
    let profile = new_profile(project_id, name);
    store.profiles.push(profile.clone());
    save_store(&data_root, &store)?;
    Ok(profile)
}

#[tauri::command]
pub fn batch_create_profiles(request: BatchCreateRequest) -> CommandResult<Vec<BrowserProfile>> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let project_id = if store
        .projects
        .iter()
        .any(|project| project.id == request.project_id)
    {
        request.project_id
    } else {
        store.active_project_id.clone()
    };
    let count = request.count.clamp(1, 200);
    let mut created = Vec::new();
    for index in 1..=count {
        let mut profile = new_profile(
            project_id.clone(),
            Some(format!(
                "批量环境 {:03}",
                store.profiles.len() + index as usize
            )),
        );
        profile.proxy = request.proxy.trim().to_string();
        let auto_locale = request.auto_locale || request.auto_locale_timezone;
        let auto_timezone = request.auto_timezone || request.auto_locale_timezone;
        profile.auto_locale = auto_locale;
        profile.auto_timezone = auto_timezone;
        profile.auto_locale_timezone = auto_locale && auto_timezone;
        created.push(profile.clone());
        store.profiles.push(profile);
    }
    save_store(&data_root, &store)?;
    Ok(created)
}

#[tauri::command]
pub fn save_profile(profile: BrowserProfile) -> CommandResult<BrowserProfile> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let profile = normalize_profile(profile);
    let index = store
        .profiles
        .iter()
        .position(|item| item.id == profile.id)
        .ok_or_else(|| "找不到该环境".to_string())?;
    store.profiles[index] = profile.clone();
    save_store(&data_root, &store)?;
    Ok(profile)
}

#[tauri::command]
pub fn clone_profile(profile_id: String) -> CommandResult<BrowserProfile> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let original = find_profile(&store, &profile_id)?.clone();
    let existing_names: HashSet<String> = store
        .profiles
        .iter()
        .filter(|profile| profile.project_id == store.active_project_id)
        .map(|profile| profile.name.trim().to_lowercase())
        .collect();
    let mut clone = new_profile(
        store.active_project_id.clone(),
        Some(next_clone_name(&original.name, &existing_names)),
    );
    clone.proxy = original.proxy;
    clone.proxy_mode = original.proxy_mode;
    clone.proxy_protocol = original.proxy_protocol;
    clone.proxy_host = original.proxy_host;
    clone.proxy_port = original.proxy_port;
    clone.proxy_username = original.proxy_username;
    clone.proxy_password = original.proxy_password;
    clone.proxy_api = original.proxy_api;
    clone.proxy_pool_id = original.proxy_pool_id;
    clone.locale = original.locale;
    clone.timezone = original.timezone;
    clone.auto_locale = original.auto_locale;
    clone.auto_timezone = original.auto_timezone;
    clone.auto_locale_timezone = original.auto_locale && original.auto_timezone;
    clone.screen_mode = original.screen_mode;
    if clone.screen_mode == "random" {
        let (width, height) = random_screen_size();
        clone.screen_width = width;
        clone.screen_height = height;
    } else {
        clone.screen_width = original.screen_width;
        clone.screen_height = original.screen_height;
    }
    clone.font_mode = original.font_mode;
    if clone.font_mode != "random" {
        clone.hardware_concurrency = original.hardware_concurrency;
    }
    clone.media_devices = original.media_devices;
    if clone.media_devices != "random" {
        clone.device_memory = original.device_memory;
    }
    clone.webgl_image = original.webgl_image;
    if clone.webgl_image != "random" {
        clone.gpu_vendor = original.gpu_vendor;
        clone.gpu_renderer = original.gpu_renderer;
    }
    clone.tags = original.tags;
    clone.injection_scripts = original.injection_scripts;
    clone.one_shot_injection_scripts = original.one_shot_injection_scripts;
    store.profiles.push(clone.clone());
    save_store(&data_root, &store)?;
    Ok(clone)
}

#[tauri::command]
pub fn delete_profile(profile_id: String) -> CommandResult<()> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let index = store
        .profiles
        .iter()
        .position(|profile| profile.id == profile_id)
        .ok_or_else(|| "找不到该环境".to_string())?;
    store.profiles.remove(index);
    save_store(&data_root, &store)
}

#[tauri::command]
pub fn batch_delete_profiles(profile_ids: Vec<String>) -> CommandResult<usize> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let selected: HashSet<String> = profile_ids.into_iter().collect();
    let before = store.profiles.len();
    store
        .profiles
        .retain(|profile| !selected.contains(&profile.id));
    let removed = before.saturating_sub(store.profiles.len());
    save_store(&data_root, &store)?;
    Ok(removed)
}

#[tauri::command]
pub fn move_profile_to_project(profile_id: String, project_id: String) -> CommandResult<()> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    if !store.projects.iter().any(|project| project.id == project_id) {
        return Err("找不到目标项目".to_string());
    }
    let profile = find_profile_mut(&mut store, &profile_id)?;
    let from_project_id = profile.project_id.clone();
    move_profile_data(&data_root, &profile_id, &from_project_id, &project_id)?;
    profile.project_id = project_id.clone();
    profile.updated_at = now_iso();
    store.active_project_id = project_id;
    save_store(&data_root, &store)
}

#[tauri::command]
pub fn batch_move_profiles(profile_ids: Vec<String>, project_id: String) -> CommandResult<usize> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    if !store.projects.iter().any(|project| project.id == project_id) {
        return Err("找不到目标项目".to_string());
    }

    let selected: HashSet<String> = profile_ids.into_iter().collect();
    let moves: Vec<(String, String)> = store
        .profiles
        .iter()
        .filter(|profile| selected.contains(&profile.id) && profile.project_id != project_id)
        .map(|profile| (profile.id.clone(), profile.project_id.clone()))
        .collect();
    for (profile_id, from_project_id) in &moves {
        move_profile_data(&data_root, profile_id, from_project_id, &project_id)?;
    }
    for profile in &mut store.profiles {
        if selected.contains(&profile.id) {
            profile.project_id = project_id.clone();
            profile.updated_at = now_iso();
        }
    }
    store.active_project_id = project_id;
    let count = moves.len();
    save_store(&data_root, &store)?;
    Ok(count)
}

#[tauri::command]
pub fn batch_launch(
    state: State<'_, Arc<RuntimeState>>,
    profile_ids: Vec<String>,
) -> CommandResult<Vec<LaunchResult>> {
    launch_profiles_batch(&state, profile_ids, true, None, None)
}

#[tauri::command]
pub fn batch_stop(state: State<'_, Arc<RuntimeState>>, profile_ids: Vec<String>) -> CommandResult<()> {
    for profile_id in profile_ids {
        stop_running(&state, &profile_id);
    }
    Ok(())
}

#[tauri::command]
pub async fn test_proxy(proxy: String, protocol: Option<String>) -> CommandResult<ProxyCheckResult> {
    let preferred_protocol = protocol.unwrap_or_else(default_proxy_protocol);
    let proxy_for_result = proxy.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let geo = resolve_geoip(&proxy, &preferred_protocol)?;
        Ok(ProxyCheckResult {
            ip: geo.ip,
            country_code: geo.country_code,
            locale: geo.locale,
            timezone: geo.timezone,
            proxy: proxy_for_result,
        })
    })
    .await
    .map_err(|err| err.to_string())?
}

#[tauri::command]
pub fn extract_proxy_from_api(api_url: String, protocol: String) -> CommandResult<ProxyApiResult> {
    let api_url = api_url.trim();
    if api_url.is_empty() {
        return Err("代理 API 地址不能为空".to_string());
    }
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("CloakFingerprintControllerRust/0.1")
        .build()
        .map_err(|err| err.to_string())?
        .get(api_url)
        .send()
        .map_err(|err| err.to_string())?
        .text()
        .map_err(|err| err.to_string())?;
    parse_proxy_payload(&response, &protocol)
}

#[tauri::command]
pub fn export_profiles(project_id: Option<String>) -> CommandResult<String> {
    let data_root = data_root_path();
    let store = load_store(&data_root);
    let profiles = match project_id {
        Some(project_id) => store
            .profiles
            .into_iter()
            .filter(|profile| profile.project_id == project_id)
            .collect(),
        None => store.profiles,
    };
    let payload = ImportExportPayload {
        exported_at: now_iso(),
        projects: store.projects,
        profiles,
    };
    serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn import_profiles(payload: String) -> CommandResult<usize> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let parsed: ImportExportPayload =
        serde_json::from_str(&payload).map_err(|err| format!("导入 JSON 格式错误: {err}"))?;
    for project in parsed.projects {
        if project.id.trim().is_empty() || project.name.trim().is_empty() {
            continue;
        }
        if !store.projects.iter().any(|item| item.id == project.id) {
            store.projects.push(project);
        }
    }
    let mut count = 0usize;
    for mut profile in parsed.profiles {
        profile.id = Uuid::new_v4().to_string();
        profile.name = format!("{} 导入", profile.name.trim());
        profile.created_at = now_iso();
        profile.updated_at = profile.created_at.clone();
        profile.last_launched_at.clear();
        if !store
            .projects
            .iter()
            .any(|project| project.id == profile.project_id)
        {
            profile.project_id = store.active_project_id.clone();
        }
        store.profiles.push(normalize_profile(profile));
        count += 1;
    }
    save_store(&data_root, &store)?;
    Ok(count)
}

#[tauri::command]
pub fn reorder_profiles(project_id: String, profile_ids: Vec<String>) -> CommandResult<()> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let requested: HashSet<String> = profile_ids.iter().cloned().collect();
    let mut ordered = Vec::new();
    let mut others = Vec::new();
    let mut by_id: HashMap<String, BrowserProfile> = HashMap::new();

    for profile in store.profiles {
        if profile.project_id == project_id {
            by_id.insert(profile.id.clone(), profile);
        } else {
            others.push(profile);
        }
    }
    for id in profile_ids {
        if let Some(profile) = by_id.remove(&id) {
            ordered.push(profile);
        }
    }
    for (_, profile) in by_id {
        if !requested.contains(&profile.id) {
            ordered.push(profile);
        }
    }
    others.extend(ordered);
    store.profiles = others;
    save_store(&data_root, &store)
}

#[tauri::command]
pub fn launch_profile(
    state: State<'_, Arc<RuntimeState>>,
    profile_id: String,
) -> CommandResult<LaunchResult> {
    launch_profiles_batch(&state, vec![profile_id], false, None, None)?
        .into_iter()
        .next()
        .ok_or_else(|| "启动浏览器失败".to_string())
}

/// Launch a profile pointed at a one-off URL (e.g. a fingerprint detection
/// site) without modifying the stored homepage.
#[tauri::command]
pub fn launch_profile_at(
    state: State<'_, Arc<RuntimeState>>,
    profile_id: String,
    url: String,
) -> CommandResult<LaunchResult> {
    launch_profiles_batch(&state, vec![profile_id], true, Some(url), None)?
        .into_iter()
        .next()
        .ok_or_else(|| "启动浏览器失败".to_string())
}

#[tauri::command]
pub fn stop_profile(state: State<'_, Arc<RuntimeState>>, profile_id: String) -> CommandResult<()> {
    stop_running(&state, &profile_id);
    Ok(())
}

#[tauri::command]
pub fn build_launch_command(profile_id: String) -> CommandResult<String> {
    let settings = load_settings();
    let data_root = PathBuf::from(&settings.data_root);
    let store = load_store(&data_root);
    let profile = find_profile(&store, &profile_id)?;
    let dir = profile_dir(&data_root, &profile.project_id, &profile.id);
    Ok(command_preview(&settings.browser_path, profile, &dir, false))
}

#[tauri::command]
pub fn open_profile_folder(profile_id: String) -> CommandResult<()> {
    let settings = load_settings();
    let data_root = PathBuf::from(settings.data_root);
    let store = load_store(&data_root);
    let profile = find_profile(&store, &profile_id)?;
    let dir = profile_dir(&data_root, &profile.project_id, &profile.id);
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("explorer");
        command.arg(&dir);
        command.creation_flags(CREATE_NO_WINDOW);
        command.spawn().map_err(|err| err.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        tauri_plugin_opener::open_path(crate::util::clean_path_text(&dir), None::<&str>)
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Fingerprint quality
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn audit_fingerprint(profile_id: String) -> CommandResult<FingerprintAudit> {
    let data_root = data_root_path();
    let store = load_store(&data_root);
    let profile = find_profile(&store, &profile_id)?;
    Ok(audit_profile(profile))
}

// ---------------------------------------------------------------------------
// Proxy pool
// ---------------------------------------------------------------------------

fn compose_proxy_url(
    protocol: &str,
    host: &str,
    port: &str,
    username: &str,
    password: &str,
) -> String {
    let scheme = if protocol.trim().is_empty() {
        "socks5".to_string()
    } else {
        protocol.trim().to_lowercase()
    };
    let host = host.trim();
    let port = port.trim();
    if host.is_empty() || port.is_empty() {
        return String::new();
    }
    if username.trim().is_empty() {
        format!("{scheme}://{host}:{port}")
    } else {
        format!(
            "{scheme}://{}:{}@{host}:{port}",
            username.trim(),
            password.trim()
        )
    }
}

#[tauri::command]
pub fn save_proxy(mut entry: ProxyEntry) -> CommandResult<ProxyEntry> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    if entry.id.trim().is_empty() {
        entry.id = Uuid::new_v4().to_string();
    }
    if entry.created_at.trim().is_empty() {
        entry.created_at = now_iso();
    }
    if entry.name.trim().is_empty() {
        entry.name = format!("{}:{}", entry.host.trim(), entry.port.trim());
    }
    entry.url = compose_proxy_url(
        &entry.protocol,
        &entry.host,
        &entry.port,
        &entry.username,
        &entry.password,
    );
    if entry.url.is_empty() {
        return Err("代理缺少 host 或 port".to_string());
    }
    match store.proxies.iter_mut().find(|item| item.id == entry.id) {
        Some(existing) => *existing = entry.clone(),
        None => store.proxies.push(entry.clone()),
    }
    save_store(&data_root, &store)?;
    Ok(entry)
}

#[tauri::command]
pub fn delete_proxy(proxy_id: String) -> CommandResult<()> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    store.proxies.retain(|item| item.id != proxy_id);
    for profile in &mut store.profiles {
        if profile.proxy_pool_id == proxy_id {
            profile.proxy_pool_id.clear();
        }
    }
    save_store(&data_root, &store)
}

/// Bulk import proxies from free text, one per line. Accepts `ip:port`,
/// `ip:port:user:pass`, or full `scheme://user:pass@host:port` URLs.
#[tauri::command]
pub fn import_proxies(text: String, protocol: String) -> CommandResult<usize> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let default_protocol = if protocol.trim().is_empty() {
        default_proxy_protocol()
    } else {
        protocol.trim().to_string()
    };
    let mut count = 0usize;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(parsed) = parse_proxy_line(line, &default_protocol) else {
            continue;
        };
        let entry = ProxyEntry {
            id: Uuid::new_v4().to_string(),
            name: format!("{}:{}", parsed.host, parsed.port),
            protocol: parsed.protocol,
            host: parsed.host,
            port: parsed.port,
            username: parsed.username,
            password: parsed.password,
            url: parsed.proxy,
            last_ip: String::new(),
            last_country: String::new(),
            last_checked_at: String::new(),
            status: default_proxy_status(),
            created_at: now_iso(),
        };
        store.proxies.push(entry);
        count += 1;
    }
    save_store(&data_root, &store)?;
    Ok(count)
}

/// Check a single pool proxy's connectivity and persist the resulting IP/status.
#[tauri::command]
pub async fn check_proxy_entry(proxy_id: String) -> CommandResult<ProxyEntry> {
    tauri::async_runtime::spawn_blocking(move || {
        let data_root = data_root_path();
        let mut store = load_store(&data_root);
        let entry = store
            .proxies
            .iter()
            .find(|item| item.id == proxy_id)
            .cloned()
            .ok_or_else(|| "找不到该代理".to_string())?;
        let result = resolve_geoip(&entry.url, &entry.protocol);
        let target = store
            .proxies
            .iter_mut()
            .find(|item| item.id == proxy_id)
            .ok_or_else(|| "找不到该代理".to_string())?;
        target.last_checked_at = now_iso();
        match &result {
            Ok(geo) => {
                target.last_ip = geo.ip.clone();
                target.last_country = geo.country_code.clone();
                target.status = "ok".to_string();
            }
            Err(_) => {
                target.last_ip.clear();
                target.status = "fail".to_string();
            }
        }
        let updated = target.clone();
        save_store(&data_root, &store)?;
        result.map(|_| updated)
    })
    .await
    .map_err(|err| err.to_string())?
}

/// Bind a pool proxy to one or more profiles (copies the URL into each profile).
#[tauri::command]
pub fn assign_proxy_to_profiles(proxy_id: String, profile_ids: Vec<String>) -> CommandResult<usize> {
    let data_root = data_root_path();
    let mut store = load_store(&data_root);
    let entry = store
        .proxies
        .iter()
        .find(|item| item.id == proxy_id)
        .cloned()
        .ok_or_else(|| "找不到该代理".to_string())?;
    let selected: HashSet<String> = profile_ids.into_iter().collect();
    let mut count = 0usize;
    for profile in &mut store.profiles {
        if selected.contains(&profile.id) {
            profile.proxy_pool_id = entry.id.clone();
            profile.proxy = entry.url.clone();
            profile.proxy_protocol = entry.protocol.clone();
            profile.proxy_host = entry.host.clone();
            profile.proxy_port = entry.port.clone();
            profile.proxy_username = entry.username.clone();
            profile.proxy_password = entry.password.clone();
            profile.updated_at = now_iso();
            count += 1;
        }
    }
    save_store(&data_root, &store)?;
    Ok(count)
}

// ---------------------------------------------------------------------------
// Security: master password
// ---------------------------------------------------------------------------

/// Combined security snapshot the frontend reads before rendering the main UI.
#[tauri::command]
pub fn get_security_status() -> serde_json::Value {
    serde_json::json!({
        "hasMasterPassword": crate::crypto::has_master_password(),
        "locked": crate::crypto::is_locked(),
    })
}

#[tauri::command]
pub fn unlock(password: String) -> CommandResult<()> {
    crate::crypto::unlock(&password)
}

/// Set or change the master password, then re-encrypt the store with it.
#[tauri::command]
pub fn set_master_password(password: String) -> CommandResult<()> {
    if crate::crypto::is_locked() {
        return Err("请先解锁".to_string());
    }
    let data_root = data_root_path();
    let store = load_store(&data_root);
    crate::crypto::install_master_password(&password)?;
    save_store(&data_root, &store)
}

/// Remove the master password, reverting to the machine key.
#[tauri::command]
pub fn remove_master_password() -> CommandResult<()> {
    if crate::crypto::is_locked() {
        return Err("请先解锁".to_string());
    }
    let data_root = data_root_path();
    let store = load_store(&data_root);
    crate::crypto::clear_master_password()?;
    save_store(&data_root, &store)
}

/// Return the current automation-API base URL, token and per-profile endpoints.
#[tauri::command]
pub fn get_automation_info(state: State<'_, Arc<RuntimeState>>) -> serde_json::Value {
    let api_port = *state.api_port.lock().unwrap();
    let token = state.api_token.lock().unwrap().clone();
    serde_json::json!({
        "apiPort": api_port,
        "apiBase": if api_port > 0 { format!("http://127.0.0.1:{api_port}") } else { String::new() },
        "token": token,
        "endpoints": state.endpoint_snapshot(),
    })
}
