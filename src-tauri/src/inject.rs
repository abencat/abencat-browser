//! Userscript parsing, MV3 injection-extension generation and cookie seeding.

use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::models::BrowserProfile;
use crate::util::{clean_list, clean_path_text, CommandResult};

pub struct ScriptMeta {
    pub matches: Vec<String>,
    pub run_at: String,
}

pub fn combined_scripts(profile: &BrowserProfile) -> Vec<String> {
    clean_list(
        profile
            .injection_scripts
            .iter()
            .chain(profile.one_shot_injection_scripts.iter())
            .cloned()
            .collect(),
    )
}

pub fn enabled_scripts(script_paths: &[String]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    script_paths
        .iter()
        .map(|path| path.trim())
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .filter(|path| seen.insert(path.to_string_lossy().to_lowercase()))
        .collect()
}

fn chrome_run_at(userscript_run_at: &str) -> String {
    match userscript_run_at.trim().to_lowercase().as_str() {
        "document-start" => "document_start".to_string(),
        "document-idle" => "document_idle".to_string(),
        _ => "document_end".to_string(),
    }
}

pub fn parse_script_meta(source: &str) -> ScriptMeta {
    let mut matches = Vec::new();
    let mut run_at = "document_end".to_string();
    let mut in_block = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == "// ==UserScript==" {
            in_block = true;
            continue;
        }
        if trimmed == "// ==/UserScript==" {
            break;
        }
        if !in_block || !trimmed.starts_with("//") {
            continue;
        }
        let meta = trimmed.trim_start_matches('/').trim();
        let Some(meta) = meta.strip_prefix('@') else {
            continue;
        };
        let mut parts = meta.splitn(2, char::is_whitespace);
        let key = parts.next().unwrap_or("").trim().to_lowercase();
        let value = parts.next().unwrap_or("").trim();
        if value.is_empty() {
            continue;
        }
        match key.as_str() {
            "match" | "include" => {
                matches.push(if value == "*" {
                    "<all_urls>".to_string()
                } else {
                    value.to_string()
                });
            }
            "run-at" => run_at = chrome_run_at(value),
            _ => {}
        }
    }

    if matches.is_empty() {
        matches.push("<all_urls>".to_string());
    }
    ScriptMeta { matches, run_at }
}

fn safe_script_name(path: &Path, index: usize) -> String {
    let base = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("script.js");
    let mut clean = String::new();
    for ch in base.chars() {
        clean.push(
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            },
        );
    }
    if !clean.to_lowercase().ends_with(".js") {
        clean.push_str(".js");
    }
    format!("{index:03}_{clean}")
}

fn wrap_script(file_name: &str, source: &str) -> String {
    let script_name = serde_json::to_string(file_name).unwrap_or_else(|_| "\"script\"".to_string());
    format!(
        "(() => {{\n  const scriptName = {script_name};\n  try {{\n{source}\n  }} catch (error) {{\n    console.error('[CloakInjector]', scriptName, error);\n  }}\n}})();\n"
    )
}

fn remove_dir_contents(path: &Path) -> CommandResult<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let child = entry.path();
        if child.is_dir() {
            fs::remove_dir_all(&child).map_err(|err| err.to_string())?;
        } else {
            fs::remove_file(&child).map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

pub fn build_injection_extension(
    script_paths: &[String],
    extension_dir: &Path,
) -> CommandResult<()> {
    let scripts = enabled_scripts(script_paths);
    if scripts.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(extension_dir).map_err(|err| err.to_string())?;
    remove_dir_contents(extension_dir)?;

    let mut content_scripts = Vec::new();
    for (index, script_path) in scripts.iter().enumerate() {
        let source = fs::read_to_string(script_path)
            .map_err(|err| format!("读取脚本失败 {}: {err}", clean_path_text(script_path)))?;
        let meta = parse_script_meta(&source);
        let output_name = safe_script_name(script_path, index + 1);
        let file_name = script_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("script.js");
        fs::write(
            extension_dir.join(&output_name),
            wrap_script(file_name, &source),
        )
        .map_err(|err| err.to_string())?;
        content_scripts.push(json!({
            "matches": meta.matches,
            "js": [output_name],
            "run_at": meta.run_at,
            "all_frames": true
        }));
    }

    let manifest = json!({
        "manifest_version": 3,
        "name": "Abencat Local JS Injector",
        "version": "1.0.0",
        "description": "Generated by Abencat Browser.",
        "host_permissions": ["<all_urls>"],
        "content_scripts": content_scripts
    });
    fs::write(
        extension_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

pub fn write_profile_cookies(profile: &BrowserProfile, dir: &Path) -> CommandResult<()> {
    let raw = profile.cookies_json.trim();
    if raw.is_empty() {
        return Ok(());
    }
    let parsed: Value =
        serde_json::from_str(raw).map_err(|err| format!("Cookie JSON 格式错误: {err}"))?;
    let bytes = serde_json::to_vec_pretty(&parsed).map_err(|err| err.to_string())?;
    fs::write(dir.join("CloakCookies.json"), bytes).map_err(|err| err.to_string())
}
