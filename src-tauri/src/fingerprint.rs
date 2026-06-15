//! Local fingerprint self-check: scores a profile for internal consistency and
//! flags settings that commonly leak or look automated to anti-bot systems.

use crate::models::{BrowserProfile, FingerprintAudit};

/// Audit a profile and return a 0-100 quality score plus issues/warnings.
/// Issues cost more than warnings; the score is purely heuristic and meant to
/// guide the operator, not to guarantee undetectability.
pub fn audit_profile(profile: &BrowserProfile) -> FingerprintAudit {
    let mut issues: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Fingerprint seed is the backbone of per-profile entropy.
    if profile.seed.trim().is_empty() {
        issues.push("缺少指纹种子 (seed)，多个环境会共享同一指纹".to_string());
    }

    // Noise off makes canvas/webgl/audio trivially correlatable.
    if !profile.noise_enabled {
        warnings.push("指纹噪声已关闭，Canvas/WebGL/Audio 更易被关联".to_string());
    }

    // WebRTC leaks the real IP behind a proxy.
    if profile.webrtc_policy != "protect" && profile.webrtc_policy != "proxy" {
        warnings.push("WebRTC 策略未设为保护，可能泄露真实 IP".to_string());
    }

    // Proxy present but timezone/locale not following it is a classic mismatch.
    let has_proxy = !profile.proxy.trim().is_empty() || !profile.proxy_pool_id.trim().is_empty();
    if has_proxy {
        let country = profile
            .proxy_country
            .trim()
            .to_uppercase();
        if !country.is_empty() {
            let expected = crate::proxy::locale_for_country(&country);
            let lang = expected.split('-').next().unwrap_or("");
            if !profile.auto_locale
                && !lang.is_empty()
                && !profile.locale.to_lowercase().starts_with(lang)
            {
                warnings.push(format!(
                    "代理出口为 {country}，但语言为 {}，建议开启自动同步",
                    profile.locale
                ));
            }
            if !profile.auto_timezone && profile.timezone.trim().is_empty() {
                warnings.push("代理已设置但时区为空".to_string());
            }
        }
    }

    // Hardware values out of realistic desktop range look synthetic.
    if profile.hardware_concurrency == 0 || profile.hardware_concurrency > 64 {
        issues.push(format!(
            "CPU 核心数 {} 不在常见范围 (1-64)",
            profile.hardware_concurrency
        ));
    }
    if profile.device_memory != 0
        && ![1, 2, 4, 8, 16, 32, 64].contains(&profile.device_memory)
    {
        warnings.push(format!(
            "内存 {}GB 非 2 的幂，navigator.deviceMemory 只暴露 1/2/4/8",
            profile.device_memory
        ));
    }

    // Unrealistic screen sizes.
    if profile.screen_width < 800 || profile.screen_height < 600 {
        warnings.push(format!(
            "屏幕分辨率 {}x{} 偏小，移动端尺寸用于桌面 UA 会矛盾",
            profile.screen_width, profile.screen_height
        ));
    }

    // GPU vendor/renderer mismatch (e.g. NVIDIA vendor + AMD renderer).
    let vendor = profile.gpu_vendor.to_lowercase();
    let renderer = profile.gpu_renderer.to_lowercase();
    if !vendor.is_empty() && !renderer.is_empty() {
        let vendor_brand = if vendor.contains("nvidia") {
            "nvidia"
        } else if vendor.contains("amd") || vendor.contains("ati") {
            "amd"
        } else if vendor.contains("intel") {
            "intel"
        } else {
            ""
        };
        if !vendor_brand.is_empty()
            && !renderer.contains(vendor_brand)
            && !renderer.contains("angle")
        {
            warnings.push("GPU 厂商与渲染器字符串不匹配".to_string());
        }
    }

    let score = 100i32
        .saturating_sub(issues.len() as i32 * 25)
        .saturating_sub(warnings.len() as i32 * 8)
        .max(0) as u32;

    FingerprintAudit {
        score,
        issues,
        warnings,
    }
}
