//! Settings + profile/proxy store persistence and path resolution.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::crypto;
use crate::models::*;
use crate::profile::{normalize_auto_locale_timezone, rand_seed, starter_profiles};
use crate::util::{clean_path_text, current_dir, now_iso, CommandResult};

pub fn default_data_root() -> PathBuf {
    let cwd = current_dir();
    let legacy = cwd
        .parent()
        .unwrap_or(cwd.as_path())
        .join("FingerprintController")
        .join("build-msvc2019")
        .join("release")
        .join("controller-data");
    if legacy.exists() {
        legacy
    } else {
        cwd.join("controller-data")
    }
}

pub fn settings_path() -> PathBuf {
    default_data_root().join("settings.json")
}

pub fn profiles_path(data_root: &Path) -> PathBuf {
    data_root.join("profiles.json")
}

/// (bundle subdir, executable name) for the bundled Cloak browser per platform.
#[cfg(windows)]
const BUNDLE_DIRS: &[(&str, &str)] = &[("cloakbrowser-windows-x64", "chrome.exe")];
#[cfg(target_os = "linux")]
const BUNDLE_DIRS: &[(&str, &str)] = &[
    ("cloakbrowser-linux-x64", "chrome"),
    ("cloakbrowser-linux-x64", "chrome-wrapper"),
];
#[cfg(target_os = "macos")]
const BUNDLE_DIRS: &[(&str, &str)] = &[("cloakbrowser-macos", "Chromium.app/Contents/MacOS/Chromium")];

/// System Chrome/Chromium fallbacks (Linux/macOS).
#[cfg(target_os = "linux")]
const SYSTEM_BROWSERS: &[&str] = &[
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/snap/bin/chromium",
];
#[cfg(target_os = "macos")]
const SYSTEM_BROWSERS: &[&str] = &[
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
];
#[cfg(windows)]
const SYSTEM_BROWSERS: &[&str] = &[];

pub fn detect_browser_path() -> String {
    if let Ok(path) = std::env::var("CLOAKBROWSER_BINARY_PATH") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return clean_path_text(&candidate);
        }
    }

    let cwd = current_dir();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| cwd.clone());
    let mut roots = vec![cwd.clone(), exe_dir.clone()];
    roots.extend(cwd.ancestors().map(Path::to_path_buf));
    roots.extend(exe_dir.ancestors().map(Path::to_path_buf));

    for root in &roots {
        for (subdir, exe) in BUNDLE_DIRS {
            let candidate = root.join(subdir).join(exe);
            if candidate.is_file() {
                return clean_path_text(&candidate);
            }
        }
    }

    for system in SYSTEM_BROWSERS {
        let candidate = PathBuf::from(system);
        if candidate.is_file() {
            return clean_path_text(&candidate);
        }
    }
    String::new()
}

pub fn load_settings() -> AppSettings {
    let mut settings = AppSettings {
        browser_path: detect_browser_path(),
        data_root: clean_path_text(&default_data_root()),
    };
    let path = settings_path();
    if let Ok(bytes) = fs::read(path) {
        if let Ok(file_settings) = serde_json::from_slice::<AppSettings>(&bytes) {
            if !file_settings.browser_path.trim().is_empty() {
                settings.browser_path = file_settings.browser_path;
            }
            if !file_settings.data_root.trim().is_empty() {
                settings.data_root = file_settings.data_root;
            }
        }
    }
    settings
}

pub fn save_settings_file(settings: &AppSettings) -> CommandResult<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(settings).map_err(|err| err.to_string())?;
    fs::write(path, bytes).map_err(|err| err.to_string())
}

pub fn data_root_path() -> PathBuf {
    PathBuf::from(load_settings().data_root)
}

pub fn load_store(data_root: &Path) -> StoreFile {
    let _ = fs::create_dir_all(data_root.join("user-data"));
    let loaded_store = fs::read(profiles_path(data_root))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<StoreFile>(&bytes).ok());
    let mut store = loaded_store.unwrap_or_else(|| StoreFile {
        seeded: false,
        projects: default_projects(),
        active_project_id: default_active_project_id(),
        profiles: Vec::new(),
        proxies: Vec::new(),
    });

    if store.projects.is_empty() {
        store.projects = default_projects();
    }
    if !store.seeded {
        for project in default_projects() {
            if !store.projects.iter().any(|item| item.id == project.id) {
                store.projects.push(project);
            }
        }
        if store.profiles.is_empty() {
            store.profiles = starter_profiles();
        }
        store.seeded = true;
    }
    if !store.projects.iter().any(|project| project.id == "default") {
        store.projects.insert(
            0,
            ProjectInfo {
                id: "default".to_string(),
                name: "默认项目".to_string(),
            },
        );
    }
    let project_ids: HashSet<String> = store.projects.iter().map(|p| p.id.clone()).collect();
    for profile in &mut store.profiles {
        normalize_auto_locale_timezone(profile);
        if profile.id.trim().is_empty() {
            profile.id = uuid::Uuid::new_v4().to_string();
        }
        if profile.project_id.trim().is_empty() || !project_ids.contains(&profile.project_id) {
            profile.project_id = "default".to_string();
        }
        if profile.seed.trim().is_empty() {
            profile.seed = rand_seed();
        }
        if profile.created_at.trim().is_empty() {
            profile.created_at = now_iso();
        }
        // Decrypt sensitive fields that were stored encrypted at rest.
        crypto::decrypt_profile_secrets(profile);
    }
    for proxy in &mut store.proxies {
        crypto::decrypt_proxy_secrets(proxy);
    }
    if !store
        .projects
        .iter()
        .any(|project| project.id == store.active_project_id)
    {
        store.active_project_id = store
            .projects
            .first()
            .map(|project| project.id.clone())
            .unwrap_or_else(default_active_project_id);
    }
    store
}

pub fn save_store(data_root: &Path, store: &StoreFile) -> CommandResult<()> {
    fs::create_dir_all(data_root).map_err(|err| err.to_string())?;
    // Encrypt sensitive fields just before serializing to disk; the in-memory
    // store keeps plaintext so callers are unaffected.
    let mut on_disk = store.clone();
    for profile in &mut on_disk.profiles {
        crypto::encrypt_profile_secrets(profile);
    }
    for proxy in &mut on_disk.proxies {
        crypto::encrypt_proxy_secrets(proxy);
    }
    let bytes = serde_json::to_vec_pretty(&on_disk).map_err(|err| err.to_string())?;
    fs::write(profiles_path(data_root), bytes).map_err(|err| err.to_string())
}

pub fn find_profile_mut<'a>(
    store: &'a mut StoreFile,
    profile_id: &str,
) -> CommandResult<&'a mut BrowserProfile> {
    store
        .profiles
        .iter_mut()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| "找不到该环境".to_string())
}

pub fn find_profile<'a>(store: &'a StoreFile, profile_id: &str) -> CommandResult<&'a BrowserProfile> {
    store
        .profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| "找不到该环境".to_string())
}
