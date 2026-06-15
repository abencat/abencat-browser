//! Download + install the CloakBrowser fingerprint kernel (© CloakHQ).
//!
//! The kernel is not bundled with the controller; this module fetches it from
//! the configured mirror (default: the official CloakBrowser GitHub release,
//! overridable via `CLOAKBROWSER_DOWNLOAD_URL`) and installs it under the data
//! root, returning the path to the `chrome` executable. Used by the desktop
//! "Download browser" button and by the headless server's auto-download.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use crate::store::{data_root_path, load_settings, save_settings_file};
use crate::util::{clean_path_text, CommandResult, EmptyTextFallback};

const VERSION: &str = "chromium-v146.0.7680.177.5";

fn asset_name() -> &'static str {
    #[cfg(windows)]
    {
        "cloakbrowser-windows-x64.zip"
    }
    #[cfg(target_os = "linux")]
    {
        "cloakbrowser-linux-x64.tar.gz"
    }
    #[cfg(target_os = "macos")]
    {
        "cloakbrowser-macos.tar.gz"
    }
}

fn bundle_dir_name() -> &'static str {
    #[cfg(windows)]
    {
        "cloakbrowser-windows-x64"
    }
    #[cfg(target_os = "macos")]
    {
        "cloakbrowser-macos"
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        "cloakbrowser-linux-x64"
    }
}

fn chrome_exe_name() -> &'static str {
    #[cfg(windows)]
    {
        "chrome.exe"
    }
    #[cfg(not(windows))]
    {
        "chrome"
    }
}

/// Candidate download URLs, tried in order. Honors `CLOAKBROWSER_DOWNLOAD_URL`
/// (full archive URL, or a base ending in `/`), then the b.abencat.com mirror,
/// then the official CloakBrowser GitHub release.
pub fn download_urls() -> Vec<String> {
    let mut urls = Vec::new();
    if let Ok(custom) = std::env::var("CLOAKBROWSER_DOWNLOAD_URL") {
        let custom = custom.trim().to_string();
        if !custom.is_empty() {
            urls.push(if custom.ends_with('/') {
                format!("{custom}{}", asset_name())
            } else {
                custom
            });
        }
    }
    urls.push(format!("https://b.abencat.com/downloads/{}", asset_name()));
    urls.push(format!(
        "https://github.com/CloakHQ/CloakBrowser/releases/download/{VERSION}/{}",
        asset_name()
    ));
    urls
}

/// First (preferred) download URL, for display.
pub fn download_url() -> String {
    download_urls().into_iter().next().unwrap_or_default()
}

/// Recursively locate the chrome executable under `root` (handles both flat and
/// single-folder archive layouts).
fn find_chrome(root: &Path) -> Option<PathBuf> {
    let direct = root.join(chrome_exe_name());
    if direct.is_file() {
        return Some(direct);
    }
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_chrome(&path) {
                return Some(found);
            }
        }
    }
    None
}

fn install_dir() -> PathBuf {
    data_root_path().join(bundle_dir_name())
}

/// Path to an already-installed kernel, if present (settings path or our dir).
pub fn installed_chrome() -> Option<String> {
    let configured = PathBuf::from(load_settings().browser_path);
    if configured.is_file() {
        return Some(clean_path_text(&configured));
    }
    find_chrome(&install_dir()).map(|p| clean_path_text(&p))
}

fn download_to(url: &str, dest: &Path) -> CommandResult<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(1800))
        .user_agent("AbencatBrowser/0.1")
        .build()
        .map_err(|err| err.to_string())?;
    let mut resp = client
        .get(url)
        .send()
        .map_err(|err| format!("下载失败: {err}"))?;
    if !resp.status().is_success() {
        return Err(format!("下载失败: HTTP {}", resp.status()));
    }
    let mut file = fs::File::create(dest).map_err(|err| err.to_string())?;
    std::io::copy(&mut resp, &mut file).map_err(|err| format!("写入失败: {err}"))?;
    Ok(())
}

fn extract(archive: &Path, dest: &Path) -> CommandResult<()> {
    fs::create_dir_all(dest).map_err(|err| err.to_string())?;
    // bsdtar (Windows 10+) and GNU tar both auto-detect .zip / .tar.gz with -xf.
    let tar_bin = if cfg!(windows) { "tar.exe" } else { "tar" };
    let status = Command::new(tar_bin)
        .arg("-xf")
        .arg(archive)
        .arg("-C")
        .arg(dest)
        .status()
        .map_err(|err| format!("解压失败 ({tar_bin}): {err}"))?;
    if !status.success() {
        return Err("解压失败".to_string());
    }
    Ok(())
}

/// Ensure the kernel is installed; download it if missing (or `force`). Updates
/// settings.browser_path and returns the chrome path.
pub fn ensure_browser(force: bool) -> CommandResult<String> {
    if !force {
        if let Some(existing) = installed_chrome() {
            return Ok(existing);
        }
    }
    let dest = install_dir();
    fs::create_dir_all(&dest).map_err(|err| err.to_string())?;

    let archive = std::env::temp_dir().join(asset_name());
    let mut last_err = String::new();
    let mut ok = false;
    for url in download_urls() {
        match download_to(&url, &archive) {
            Ok(()) => {
                ok = true;
                break;
            }
            Err(err) => last_err = format!("{url}: {err}"),
        }
    }
    if !ok {
        return Err(last_err.if_empty("下载失败"));
    }
    extract(&archive, &dest)?;
    let _ = fs::remove_file(&archive);

    let chrome = find_chrome(&dest)
        .ok_or_else(|| format!("下载完成但未找到 {}", chrome_exe_name()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&chrome, fs::Permissions::from_mode(0o755));
        // chromedriver / crashpad handler too, best-effort.
        if let Some(dir) = chrome.parent() {
            for sib in ["chromedriver", "chrome_crashpad_handler"] {
                let p = dir.join(sib);
                if p.is_file() {
                    let _ = fs::set_permissions(&p, fs::Permissions::from_mode(0o755));
                }
            }
        }
    }

    // Persist the path so launches use it without re-detection.
    let mut settings = load_settings();
    settings.browser_path = clean_path_text(&chrome);
    let _ = save_settings_file(&settings);

    Ok(clean_path_text(&chrome))
}
