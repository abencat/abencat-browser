//! Profile creation, cloning, randomization and on-disk directory helpers.

use rand::{seq::SliceRandom, Rng};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::models::*;
use crate::util::{clean_list, clean_path_text, now_iso, CommandResult};

pub fn rand_seed() -> String {
    rand::thread_rng().gen_range(10000..99999).to_string()
}

pub fn random_device_name() -> String {
    let suffix = Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(8)
        .collect::<String>()
        .to_uppercase();
    format!("DESKTOP-{suffix}")
}

pub fn random_mac_address() -> String {
    let mut rng = rand::thread_rng();
    let first = rng.gen_range(0..=255) & 0b1111_1110;
    let mut parts = vec![first];
    parts.extend((0..5).map(|_| rng.gen_range(0..=255)));
    parts
        .into_iter()
        .map(|part| format!("{part:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}

pub fn random_cpu() -> u32 {
    *[4, 6, 8, 10, 12, 16]
        .choose(&mut rand::thread_rng())
        .unwrap_or(&8)
}

pub fn random_memory() -> u32 {
    *[4, 8, 12, 16, 24, 32]
        .choose(&mut rand::thread_rng())
        .unwrap_or(&8)
}

pub fn random_screen_size() -> (u32, u32) {
    *[
        (1366, 768),
        (1440, 900),
        (1536, 864),
        (1600, 900),
        (1920, 1080),
        (2560, 1440),
    ]
    .choose(&mut rand::thread_rng())
    .unwrap_or(&(1920, 1080))
}

pub fn new_profile(project_id: String, name: Option<String>) -> BrowserProfile {
    let mut profile = BrowserProfile {
        id: Uuid::new_v4().to_string(),
        project_id,
        name: name.unwrap_or_else(|| "新环境".to_string()),
        note: String::new(),
        proxy: String::new(),
        proxy_ip: String::new(),
        proxy_country: String::new(),
        locale: default_locale(),
        timezone: default_timezone(),
        platform: default_platform(),
        seed: rand_seed(),
        gpu_vendor: String::new(),
        gpu_renderer: String::new(),
        brand: default_brand(),
        brand_version: String::new(),
        platform_version: String::new(),
        location: String::new(),
        webrtc_ip: String::new(),
        user_agent: String::new(),
        webrtc_policy: default_webrtc_policy(),
        geolocation: String::new(),
        canvas_noise: default_noise_mode(),
        webgl_noise: default_noise_mode(),
        audio_noise: default_noise_mode(),
        fonts: String::new(),
        speech_voices: String::new(),
        do_not_track: false,
        cookies_json: String::new(),
        chrome_version: String::new(),
        ua_full_version: String::new(),
        sec_ch_ua: String::new(),
        homepage: String::new(),
        proxy_mode: default_proxy_mode(),
        proxy_protocol: default_proxy_protocol(),
        proxy_host: String::new(),
        proxy_port: String::new(),
        proxy_username: String::new(),
        proxy_password: String::new(),
        proxy_api: String::new(),
        proxy_pool_id: String::new(),
        location_policy: default_location_policy(),
        geolocation_precision: default_geolocation_precision(),
        screen_mode: default_screen_mode(),
        font_mode: default_font_mode(),
        webgl_image: default_noise_mode(),
        client_rects: default_noise_mode(),
        device_name: random_device_name(),
        mac_address: random_mac_address(),
        ssl_mode: default_ssl_mode(),
        ssl_disabled: String::new(),
        port_scan_mode: default_port_scan_mode(),
        port_whitelist: String::new(),
        gpu_enabled: true,
        media_devices: default_media_devices(),
        injection_scripts: Vec::new(),
        one_shot_injection_scripts: Vec::new(),
        extra_args: Vec::new(),
        tags: Vec::new(),
        created_at: now_iso(),
        updated_at: now_iso(),
        last_launched_at: String::new(),
        hardware_concurrency: random_cpu(),
        device_memory: random_memory(),
        screen_width: 1920,
        screen_height: 1080,
        storage_quota: 0,
        taskbar_height: 48,
        debug_port: 0,
        noise_enabled: true,
        auto_locale: false,
        auto_timezone: false,
        auto_locale_timezone: false,
    };
    randomize_clone_only_fields(&mut profile);
    profile
}

pub fn starter_profiles() -> Vec<BrowserProfile> {
    let mut default = new_profile("default".to_string(), Some("默认".to_string()));
    default.id = "demo-default".to_string();
    default.note = "主环境".to_string();
    default.seed = "74100".to_string();
    default.screen_width = 1920;
    default.screen_height = 1080;
    default.hardware_concurrency = 4;
    default.device_memory = 4;

    let mut bili_one = new_profile("default".to_string(), Some("b站小号1-test".to_string()));
    bili_one.id = "demo-bili-1".to_string();
    bili_one.note = "测试账号 1".to_string();
    bili_one.seed = "79459".to_string();
    bili_one.hardware_concurrency = 6;
    bili_one.device_memory = 24;

    let mut bili_two = new_profile("default".to_string(), Some("b站小号2-test".to_string()));
    bili_two.id = "demo-bili-2".to_string();
    bili_two.note = "测试账号 2".to_string();
    bili_two.seed = "52560".to_string();
    bili_two.hardware_concurrency = 6;
    bili_two.device_memory = 24;

    vec![default, bili_one, bili_two]
}

pub fn next_clone_name(original_name: &str, existing_names: &HashSet<String>) -> String {
    let trimmed = original_name.trim();
    let base = if trimmed.is_empty() {
        "新浏览器"
    } else {
        trimmed
    };
    let (prefix, start) = if let Some((head, tail)) = base.rsplit_once('-') {
        match tail.parse::<u32>() {
            Ok(value) => (head.trim_end().to_string(), value + 1),
            Err(_) => (base.to_string(), 1),
        }
    } else {
        (base.to_string(), 1)
    };
    for number in start..=9999 {
        let candidate = format!("{prefix}-{number}");
        if !existing_names.contains(&candidate.to_lowercase()) {
            return candidate;
        }
    }
    format!(
        "{prefix}-{}",
        Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(4)
            .collect::<String>()
    )
}

pub fn randomize_clone_only_fields(profile: &mut BrowserProfile) {
    let mut rng = rand::thread_rng();
    profile.id = Uuid::new_v4().to_string();
    profile.seed = rand_seed();
    profile.created_at = now_iso();
    profile.updated_at = profile.created_at.clone();
    profile.last_launched_at.clear();
    profile.hardware_concurrency = random_cpu();
    profile.device_memory = random_memory();
    profile.storage_quota = *[0, 60, 80, 120, 160, 240].choose(&mut rng).unwrap_or(&0);
    profile.taskbar_height = *[40, 44, 48, 52].choose(&mut rng).unwrap_or(&48);
    profile.debug_port = 0;
    profile.noise_enabled = true;
    profile.brand = "Chrome".to_string();
    profile.brand_version = String::new();
    profile.platform = "windows".to_string();
    profile.platform_version = ["10.0.0", "10.0.19045", "10.0.22631", ""]
        .choose(&mut rng)
        .unwrap_or(&"")
        .to_string();
    profile.gpu_vendor = ["Google Inc.", "NVIDIA Corporation", "Intel Inc.", "AMD", ""]
        .choose(&mut rng)
        .unwrap_or(&"")
        .to_string();
    profile.gpu_renderer = [
        "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0)",
        "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0)",
        "ANGLE (AMD, AMD Radeon Graphics Direct3D11 vs_5_0 ps_5_0)",
        "",
    ]
    .choose(&mut rng)
    .unwrap_or(&"")
    .to_string();
    profile.location.clear();
    profile.note.clear();
    profile.proxy_ip.clear();
    profile.proxy_country.clear();
    profile.webrtc_ip.clear();
    profile.user_agent.clear();
    profile.webrtc_policy = default_webrtc_policy();
    profile.geolocation.clear();
    profile.canvas_noise = default_noise_mode();
    profile.webgl_noise = default_noise_mode();
    profile.audio_noise = default_noise_mode();
    profile.fonts.clear();
    profile.speech_voices.clear();
    profile.do_not_track = false;
    profile.cookies_json.clear();
    profile.chrome_version.clear();
    profile.ua_full_version.clear();
    profile.sec_ch_ua.clear();
    profile.homepage.clear();
    profile.location_policy = default_location_policy();
    profile.geolocation_precision = default_geolocation_precision();
    profile.screen_mode = default_screen_mode();
    profile.font_mode = default_font_mode();
    profile.webgl_image = "random".to_string();
    profile.client_rects = default_noise_mode();
    profile.device_name = random_device_name();
    profile.mac_address = random_mac_address();
    profile.ssl_mode = default_ssl_mode();
    profile.ssl_disabled.clear();
    profile.port_scan_mode = default_port_scan_mode();
    profile.port_whitelist.clear();
    profile.gpu_enabled = true;
    profile.media_devices = default_media_devices();
    profile.extra_args.clear();
    if profile.screen_mode == "random" {
        let (width, height) = random_screen_size();
        profile.screen_width = width;
        profile.screen_height = height;
    }
}

pub fn normalize_profile(mut profile: BrowserProfile) -> BrowserProfile {
    normalize_auto_locale_timezone(&mut profile);
    if profile.id.trim().is_empty() {
        profile.id = Uuid::new_v4().to_string();
    }
    if profile.project_id.trim().is_empty() {
        profile.project_id = "default".to_string();
    }
    if profile.seed.trim().is_empty() {
        profile.seed = rand_seed();
    }
    if profile.created_at.trim().is_empty() {
        profile.created_at = now_iso();
    }
    profile.updated_at = now_iso();
    profile.injection_scripts = clean_list(profile.injection_scripts);
    profile.one_shot_injection_scripts = clean_list(profile.one_shot_injection_scripts);
    profile.extra_args = clean_list(profile.extra_args);
    profile.tags = clean_list(profile.tags);
    profile
}

pub fn normalize_auto_locale_timezone(profile: &mut BrowserProfile) {
    if profile.auto_locale_timezone && !profile.auto_locale && !profile.auto_timezone {
        profile.auto_locale = true;
        profile.auto_timezone = true;
    }
    profile.auto_locale_timezone = profile.auto_locale && profile.auto_timezone;
}

pub fn profile_dir(data_root: &Path, project_id: &str, profile_id: &str) -> PathBuf {
    let project_path = data_root
        .join("user-data")
        .join("projects")
        .join(project_id)
        .join(profile_id);
    if project_path.exists() {
        return project_path;
    }

    let legacy_path = data_root.join("user-data").join(profile_id);
    if legacy_path.exists() {
        return legacy_path;
    }

    let projects_root = data_root.join("user-data").join("projects");
    if let Ok(projects) = fs::read_dir(projects_root) {
        for entry in projects.flatten() {
            let candidate = entry.path().join(profile_id);
            if candidate.exists() {
                return candidate;
            }
        }
    }

    project_path
}

pub fn move_profile_data(
    data_root: &Path,
    profile_id: &str,
    from_project_id: &str,
    to_project_id: &str,
) -> CommandResult<()> {
    let from = profile_dir(data_root, from_project_id, profile_id);
    let to = data_root
        .join("user-data")
        .join("projects")
        .join(to_project_id)
        .join(profile_id);
    if from == to || !from.exists() {
        return Ok(());
    }
    if to.exists() {
        return Err(format!("目标用户数据目录已存在: {}", clean_path_text(&to)));
    }
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::rename(from, to).map_err(|err| err.to_string())
}
