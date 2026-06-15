//! Proxy parsing, GeoIP resolution and proxy-API extraction.

use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use url::Url;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::models::*;
use crate::store::{load_store, save_store};
use crate::util::{now_iso, CommandResult, EmptyTextFallback};

#[cfg(windows)]
use crate::util::CREATE_NO_WINDOW;

#[derive(Clone, Debug)]
pub struct ProxyCandidate {
    pub display: String,
    pub reqwest_url: Option<String>,
    pub is_socks: bool,
}

#[derive(Clone, Debug)]
pub struct GeoIpResult {
    pub locale: String,
    pub timezone: String,
    pub ip: String,
    pub country_code: String,
}

#[derive(Clone, Debug)]
pub struct GeoIpLookupTarget {
    pub profile_id: String,
    pub proxy: String,
    pub protocol: String,
}

pub fn locale_for_country(country_code: &str) -> String {
    match country_code.to_uppercase().as_str() {
        "CN" => "zh-CN",
        "HK" | "MO" => "zh-HK",
        "TW" => "zh-TW",
        "US" => "en-US",
        "GB" => "en-GB",
        "AU" => "en-AU",
        "CA" => "en-CA",
        "NZ" => "en-NZ",
        "SG" => "en-SG",
        "JP" => "ja-JP",
        "KR" => "ko-KR",
        "FR" => "fr-FR",
        "DE" => "de-DE",
        "IT" => "it-IT",
        "ES" => "es-ES",
        "PT" => "pt-PT",
        "BR" => "pt-BR",
        "RU" => "ru-RU",
        "UA" => "uk-UA",
        "NL" => "nl-NL",
        "BE" => "nl-BE",
        "SE" => "sv-SE",
        "NO" => "nb-NO",
        "DK" => "da-DK",
        "FI" => "fi-FI",
        "PL" => "pl-PL",
        "TR" => "tr-TR",
        "TH" => "th-TH",
        "VN" => "vi-VN",
        "ID" => "id-ID",
        "MY" => "ms-MY",
        "PH" => "en-PH",
        "IN" => "hi-IN",
        "MX" => "es-MX",
        "AR" => "es-AR",
        "CL" => "es-CL",
        "CO" => "es-CO",
        "ZA" => "en-ZA",
        "AE" => "ar-AE",
        "SA" => "ar-SA",
        "IL" => "he-IL",
        _ => "en-US",
    }
    .to_string()
}

fn split_host_port_user_pass(value: &str) -> Option<String> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 4 {
        return None;
    }
    if parts.iter().any(|part| part.trim().is_empty()) {
        return None;
    }
    Some(format!(
        "{}:{}@{}:{}",
        parts[2].trim(),
        parts[3].trim(),
        parts[0].trim(),
        parts[1].trim()
    ))
}

pub fn proxy_candidates(proxy_text: &str, preferred_protocol: &str) -> Vec<ProxyCandidate> {
    let trimmed = proxy_text.trim();
    if trimmed.is_empty() {
        return vec![ProxyCandidate {
            display: String::new(),
            reqwest_url: None,
            is_socks: false,
        }];
    }

    let mut values = Vec::new();
    if Url::parse(trimmed).is_ok() {
        values.push(trimmed.to_string());
    } else {
        let normalized = split_host_port_user_pass(trimmed).unwrap_or_else(|| trimmed.to_string());
        let preferred = preferred_protocol.trim().to_lowercase();
        let ordered_protocols = match preferred.as_str() {
            "http" | "https" => vec!["http", "https", "socks5"],
            "socks4" => vec!["socks4", "socks5", "http"],
            _ => vec!["socks5", "http", "https"],
        };
        for protocol in ordered_protocols {
            let candidate = format!("{protocol}://{normalized}");
            if Url::parse(&candidate).is_ok() && !values.contains(&candidate) {
                values.push(candidate);
            }
        }
    }

    values
        .into_iter()
        .map(|value| ProxyCandidate {
            is_socks: value.to_lowercase().starts_with("socks"),
            display: value.clone(),
            reqwest_url: Some(value),
        })
        .collect()
}

pub fn resolve_geoip(proxy_text: &str, preferred_protocol: &str) -> CommandResult<GeoIpResult> {
    let candidates = proxy_candidates(proxy_text, preferred_protocol);
    let mut last_error = String::new();
    for candidate in candidates {
        match resolve_geoip_candidate(&candidate) {
            Ok(result) => return Ok(result),
            Err(err) => {
                if !candidate.display.is_empty() {
                    last_error = format!("{}: {}", candidate.display, err);
                } else {
                    last_error = err;
                }
            }
        }
    }
    Err(if last_error.is_empty() {
        "代理地址为空或格式不支持".to_string()
    } else {
        last_error
    })
}

pub fn apply_geoip_to_profile(profile: &mut BrowserProfile, geo: GeoIpResult) {
    profile.proxy_ip = geo.ip.clone();
    profile.proxy_country = geo.country_code.clone();
    profile.location = geo.country_code.clone();
    profile.webrtc_ip = geo.ip;
    if profile.auto_locale {
        profile.locale = geo.locale;
    }
    if profile.auto_timezone {
        profile.timezone = geo.timezone;
    }
    profile.auto_locale_timezone = profile.auto_locale && profile.auto_timezone;
    profile.updated_at = now_iso();
}

fn geoip_lookup_key(proxy: &str, protocol: &str) -> String {
    let proxy = proxy.trim();
    if proxy.is_empty() {
        return "direct".to_string();
    }
    let normalized_proxy = proxy.to_lowercase();
    if Url::parse(proxy).is_ok() {
        normalized_proxy
    } else {
        format!("{}|{}", protocol.trim().to_lowercase(), normalized_proxy)
    }
}

fn apply_geoip_lookup_result_to_store(
    store: &mut StoreFile,
    profile_ids: &[String],
    result: &CommandResult<GeoIpResult>,
) {
    let selected: HashSet<&str> = profile_ids.iter().map(String::as_str).collect();
    for profile in &mut store.profiles {
        if !selected.contains(profile.id.as_str()) {
            continue;
        }
        match result {
            Ok(geo) => apply_geoip_to_profile(profile, geo.clone()),
            Err(_) => {
                profile.proxy_ip = "获取失败".to_string();
                profile.proxy_country.clear();
                profile.location.clear();
                profile.webrtc_ip.clear();
                profile.updated_at = now_iso();
            }
        }
    }
}

pub fn update_profiles_geoip_async(data_root: PathBuf, targets: Vec<GeoIpLookupTarget>) {
    if targets.is_empty() {
        return;
    }
    std::thread::spawn(move || {
        let mut grouped: HashMap<String, (String, String, Vec<String>)> = HashMap::new();
        for target in targets {
            let key = geoip_lookup_key(&target.proxy, &target.protocol);
            let entry = grouped
                .entry(key)
                .or_insert((target.proxy, target.protocol, Vec::new()));
            entry.2.push(target.profile_id);
        }

        for (_, (proxy, protocol, profile_ids)) in grouped {
            let result = resolve_geoip(&proxy, &protocol);
            let mut store = load_store(&data_root);
            apply_geoip_lookup_result_to_store(&mut store, &profile_ids, &result);
            let _ = save_store(&data_root, &store);
        }
    });
}

fn resolve_geoip_candidate(candidate: &ProxyCandidate) -> CommandResult<GeoIpResult> {
    if candidate.is_socks || candidate.reqwest_url.is_none() {
        return resolve_geoip_with_curl(candidate);
    }
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(6))
        .user_agent("CloakFingerprintControllerRust/0.1");
    if let Some(proxy_url) = &candidate.reqwest_url {
        let proxy = reqwest::Proxy::all(proxy_url).map_err(|err| err.to_string())?;
        builder = builder.proxy(proxy);
    }
    let client = builder.build().map_err(|err| err.to_string())?;
    let mut last_error = String::new();
    for url in [
        "https://ipinfo.io/json",
        "http://ip-api.com/json/?fields=status,message,query,countryCode,timezone",
        "https://ipwho.is/?fields=success,message,ip,country_code,timezone",
    ] {
        match resolve_geoip_from_url(&client, url) {
            Ok(result) => return Ok(result),
            Err(err) => last_error = err,
        }
    }
    Err(last_error)
}

fn curl_proxy_url(proxy_url: &str) -> String {
    let lower = proxy_url.to_lowercase();
    if lower.starts_with("socks5://") {
        format!("socks5h://{}", &proxy_url[9..])
    } else {
        proxy_url.to_string()
    }
}

fn run_curl_geo_request(
    url: &str,
    proxy_url: Option<&str>,
    timeout_secs: u64,
) -> CommandResult<String> {
    let curl_bin = if cfg!(windows) { "curl.exe" } else { "curl" };
    let mut command = Command::new(curl_bin);
    command
        .arg("--silent")
        .arg("--show-error")
        .arg("--max-time")
        .arg(timeout_secs.max(1).to_string());
    if let Some(proxy_url) = proxy_url {
        command.arg("--proxy").arg(curl_proxy_url(proxy_url));
    }
    command
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command
        .spawn()
        .map_err(|err| format!("{curl_bin} 启动失败: {err}"))?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(mut pipe) = child.stdout.take() {
                let _ = pipe.read_to_string(&mut stdout);
            }
            if let Some(mut pipe) = child.stderr.take() {
                let _ = pipe.read_to_string(&mut stderr);
            }
            return if status.success() {
                Ok(stdout)
            } else {
                Err(stderr
                    .trim()
                    .to_string()
                    .if_empty("curl GeoIP request failed"))
            };
        }
        if started.elapsed() > Duration::from_secs(timeout_secs + 3) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("curl GeoIP request timeout".to_string());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn resolve_geoip_with_curl(candidate: &ProxyCandidate) -> CommandResult<GeoIpResult> {
    let proxy_url = candidate.reqwest_url.as_deref();
    let mut last_error = String::new();
    let urls: &[&str] = if proxy_url.is_some() {
        &[
            "https://ipinfo.io/json",
            "http://ip-api.com/json/?fields=status,message,query,countryCode,timezone",
        ]
    } else {
        &["https://ipinfo.io/json"]
    };
    for url in urls {
        match run_curl_geo_request(url, proxy_url, 6)
            .and_then(|body| resolve_geoip_from_body(&body, url))
        {
            Ok(result) => return Ok(result),
            Err(err) => last_error = err,
        }
    }
    Err(last_error.if_empty("Get proxy network location failed."))
}

fn resolve_geoip_from_url(
    client: &reqwest::blocking::Client,
    url: &str,
) -> CommandResult<GeoIpResult> {
    let payload: Value = client
        .get(url)
        .send()
        .map_err(|err| format!("{url}: {err}"))?
        .json()
        .map_err(|err| format!("{url}: {err}"))?;
    geoip_result_from_payload(&payload)
}

fn resolve_geoip_from_body(body: &str, url: &str) -> CommandResult<GeoIpResult> {
    if body.trim().is_empty() {
        return Err(format!("{url}: empty response"));
    }
    let payload: Value = serde_json::from_str(body).map_err(|err| format!("{url}: {err}"))?;
    geoip_result_from_payload(&payload)
}

fn geoip_result_from_payload(payload: &Value) -> CommandResult<GeoIpResult> {
    if payload.get("status").is_some()
        && payload.get("status").and_then(Value::as_str) != Some("success")
    {
        return Err(value_text(payload, &["message"]).if_empty("GeoIP lookup failed"));
    }
    if payload.get("success").is_some()
        && payload.get("success").and_then(Value::as_bool) == Some(false)
    {
        return Err(value_text(payload, &["message", "reason"]).if_empty("GeoIP lookup failed"));
    }

    let country = value_text(payload, &["countryCode", "country_code", "country"]);
    let timezone = value_text(payload, &["timezone"]);
    let ip = value_text(payload, &["query", "ip"]);
    if timezone.is_empty() {
        return Err("GeoIP 响应缺少 timezone".to_string());
    }
    if ip.is_empty() {
        return Err("GeoIP 响应缺少 IP".to_string());
    }
    Ok(GeoIpResult {
        locale: locale_for_country(&country),
        timezone,
        country_code: country.to_uppercase(),
        ip,
    })
}

pub fn value_text(value: &Value, keys: &[&str]) -> String {
    for key in keys {
        if let Some(item) = value.get(*key) {
            if let Some(text) = item.as_str() {
                if !text.trim().is_empty() {
                    return text.trim().to_string();
                }
            } else if item.is_number() || item.is_boolean() {
                return item.to_string();
            }
        }
    }
    String::new()
}

fn proxy_result(
    protocol: &str,
    host: &str,
    port: &str,
    username: &str,
    password: &str,
) -> CommandResult<ProxyApiResult> {
    let protocol = if protocol.trim().is_empty() {
        default_proxy_protocol()
    } else {
        protocol.trim().to_uppercase()
    };
    let host = host.trim();
    let port = port.trim();
    if host.is_empty() || port.is_empty() {
        return Err("代理缺少 host 或 port".to_string());
    }
    let username = username.trim().to_string();
    let password = password.trim().to_string();
    let scheme = protocol.to_lowercase();
    let proxy = if username.is_empty() {
        format!("{scheme}://{host}:{port}")
    } else {
        format!("{scheme}://{username}:{password}@{host}:{port}")
    };
    Ok(ProxyApiResult {
        proxy,
        protocol,
        host: host.to_string(),
        port: port.to_string(),
        username,
        password,
    })
}

pub fn parse_proxy_line(line: &str, default_protocol: &str) -> CommandResult<ProxyApiResult> {
    let raw = line.trim().trim_matches('"').trim_matches('\'');
    if raw.is_empty() {
        return Err("API 没有返回代理地址".to_string());
    }
    let normalized = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("{}://{raw}", default_protocol.to_lowercase())
    };
    if let Ok(url) = Url::parse(&normalized) {
        if let (Some(host), Some(port)) = (url.host_str(), url.port()) {
            return proxy_result(
                url.scheme(),
                host,
                &port.to_string(),
                url.username(),
                url.password().unwrap_or(""),
            );
        }
    }

    let parts: Vec<&str> = raw.split(':').collect();
    match parts.as_slice() {
        [host, port] => proxy_result(default_protocol, host, port, "", ""),
        [host, port, username, password] => {
            proxy_result(default_protocol, host, port, username, password)
        }
        _ => Err("无法解析代理，支持 ip:port、ip:port:user:pass 或完整 URL".to_string()),
    }
}

pub fn parse_proxy_payload(payload: &str, default_protocol: &str) -> CommandResult<ProxyApiResult> {
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return Err("API 响应为空".to_string());
    }
    if let Ok(json_value) = serde_json::from_str::<Value>(trimmed) {
        let item = if let Some(first) = json_value.as_array().and_then(|items| items.first()) {
            first
        } else if let Some(data) = json_value.get("data") {
            data
        } else if let Some(result) = json_value.get("result") {
            result
        } else {
            &json_value
        };
        if let Some(text) = item.as_str() {
            return parse_proxy_line(text, default_protocol);
        }
        let proxy_text = value_text(item, &["proxy", "http", "https", "socks5", "url"]);
        if !proxy_text.is_empty() {
            return parse_proxy_line(&proxy_text, default_protocol);
        }
        let protocol = value_text(item, &["protocol", "type", "scheme"]);
        return proxy_result(
            if protocol.is_empty() {
                default_protocol
            } else {
                &protocol
            },
            &value_text(item, &["host", "ip", "server", "address"]),
            &value_text(item, &["port"]),
            &value_text(item, &["username", "user", "account"]),
            &value_text(item, &["password", "pass", "pwd"]),
        );
    }
    parse_proxy_line(trimmed.lines().next().unwrap_or(trimmed), default_protocol)
}
