//! Small shared helpers used across modules.

use chrono::Utc;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// Result type used by every Tauri command and the helpers behind them.
pub type CommandResult<T> = Result<T, String>;

#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

pub trait EmptyTextFallback {
    fn if_empty(self, fallback: &str) -> String;
}

impl EmptyTextFallback for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn clean_path_text(path: &Path) -> String {
    // Normalize to backslashes on Windows; leave POSIX paths untouched.
    #[cfg(windows)]
    {
        path.to_string_lossy().replace('/', "\\")
    }
    #[cfg(not(windows))]
    {
        path.to_string_lossy().to_string()
    }
}

pub fn current_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Trim, drop empties, and de-duplicate case-insensitively while keeping order.
pub fn clean_list(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert(value.to_lowercase()))
        .collect()
}

pub fn quote_preview(value: &str) -> String {
    if value.contains(' ') || value.contains('\t') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}
