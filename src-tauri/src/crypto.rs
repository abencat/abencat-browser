//! At-rest encryption for sensitive profile/proxy fields, with an optional
//! user master password.
//!
//! Two key modes:
//! - **Machine key (default, zero friction):** a random 256-bit key is stored in
//!   `.cloak_key` under the data root. Copying `profiles.json` elsewhere cannot
//!   decrypt it. No prompt on startup.
//! - **Master password (opt-in, compliance):** the key is derived from the user
//!   password via Argon2id with a stored salt. The app is *locked* on startup
//!   until [`unlock`] succeeds; the derived key lives only in memory.
//!
//! Values are stored as `enc:v1:<base64(nonce|ct)>`; legacy plaintext (no
//! marker) is read back unchanged and re-encrypted on the next save.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};

use crate::models::{BrowserProfile, ProxyEntry};
use crate::store::default_data_root;

const MARKER: &str = "enc:v1:";
const VERIFIER_PLAINTEXT: &[u8] = b"cloak-master-ok";

/// Active in-memory key. `None` means "not yet resolved"; combined with
/// [`has_master_password`] this tells us whether the app is locked.
static KEY_CACHE: Mutex<Option<[u8; 32]>> = Mutex::new(None);

#[derive(Serialize, Deserialize)]
struct MasterConfig {
    salt: String,
    verifier: String,
}

fn key_path() -> PathBuf {
    default_data_root().join(".cloak_key")
}

fn master_config_path() -> PathBuf {
    default_data_root().join("master.json")
}

/// True if a master password has been configured for this data root.
pub fn has_master_password() -> bool {
    master_config_path().is_file()
}

/// True if a master password is set and the app has not been unlocked yet.
pub fn is_locked() -> bool {
    has_master_password() && KEY_CACHE.lock().unwrap().is_none()
}

fn read_master_config() -> Option<MasterConfig> {
    fs::read(master_config_path())
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    use argon2::Argon2;
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|err| format!("密钥派生失败: {err}"))?;
    Ok(key)
}

fn cipher_with(key: &[u8; 32]) -> Aes256Gcm {
    Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key))
}

fn encrypt_with(key: &[u8; 32], plain: &[u8]) -> Option<String> {
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher_with(key).encrypt(nonce, plain).ok()?;
    let mut blob = nonce_bytes.to_vec();
    blob.extend_from_slice(&ciphertext);
    Some(STANDARD.encode(blob))
}

fn decrypt_with(key: &[u8; 32], encoded: &str) -> Option<Vec<u8>> {
    let blob = STANDARD.decode(encoded).ok()?;
    if blob.len() < 12 {
        return None;
    }
    let (nonce_bytes, ciphertext) = blob.split_at(12);
    cipher_with(key)
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .ok()
}

/// Load or create the machine key (used when no master password is set).
fn load_or_create_machine_key() -> [u8; 32] {
    let path = key_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|text| STANDARD.decode(text.trim()).ok())
        .and_then(|bytes| <[u8; 32]>::try_from(bytes.as_slice()).ok())
        .unwrap_or_else(|| {
            let mut fresh = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut fresh);
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&path, STANDARD.encode(fresh));
            fresh
        })
}

/// Resolve the active key. Returns `None` only when a master password is set
/// and the app has not been unlocked.
fn current_key() -> Option<[u8; 32]> {
    if let Some(key) = *KEY_CACHE.lock().unwrap() {
        return Some(key);
    }
    if has_master_password() {
        return None; // locked: caller must unlock() first
    }
    let key = load_or_create_machine_key();
    *KEY_CACHE.lock().unwrap() = Some(key);
    Some(key)
}

/// Attempt to unlock with the master password, caching the derived key.
pub fn unlock(password: &str) -> Result<(), String> {
    let config = read_master_config().ok_or_else(|| "未设置主密码".to_string())?;
    let salt = STANDARD
        .decode(&config.salt)
        .map_err(|_| "主密码配置损坏".to_string())?;
    let key = derive_key(password, &salt)?;
    let plain = decrypt_with(&key, &config.verifier).ok_or_else(|| "主密码错误".to_string())?;
    if plain != VERIFIER_PLAINTEXT {
        return Err("主密码错误".to_string());
    }
    *KEY_CACHE.lock().unwrap() = Some(key);
    Ok(())
}

/// Install/replace the master password. Requires the store to currently be
/// decryptable (machine key or already unlocked). Swaps the active key; the
/// caller must re-save the store afterwards so data is re-encrypted.
pub fn install_master_password(password: &str) -> Result<(), String> {
    if password.trim().len() < 4 {
        return Err("主密码至少 4 位".to_string());
    }
    if is_locked() {
        return Err("请先解锁".to_string());
    }
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    let key = derive_key(password, &salt)?;
    let verifier = encrypt_with(&key, VERIFIER_PLAINTEXT).ok_or_else(|| "加密失败".to_string())?;
    let config = MasterConfig {
        salt: STANDARD.encode(salt),
        verifier,
    };
    let bytes = serde_json::to_vec_pretty(&config).map_err(|err| err.to_string())?;
    let path = master_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, bytes).map_err(|err| err.to_string())?;
    *KEY_CACHE.lock().unwrap() = Some(key);
    Ok(())
}

/// Remove the master password, reverting to the machine key. Requires unlocked.
/// The caller must re-save the store afterwards.
pub fn clear_master_password() -> Result<(), String> {
    if is_locked() {
        return Err("请先解锁".to_string());
    }
    let machine_key = load_or_create_machine_key();
    *KEY_CACHE.lock().unwrap() = Some(machine_key);
    let _ = fs::remove_file(master_config_path());
    Ok(())
}

/// Encrypt a value, returning the `enc:v1:` envelope. Empty/already-encrypted
/// inputs or a locked store return the input unchanged.
pub fn encrypt_text(plain: &str) -> String {
    if plain.is_empty() || plain.starts_with(MARKER) {
        return plain.to_string();
    }
    let Some(key) = current_key() else {
        return plain.to_string();
    };
    match encrypt_with(&key, plain.as_bytes()) {
        Some(encoded) => format!("{MARKER}{encoded}"),
        None => plain.to_string(),
    }
}

/// Decrypt an `enc:v1:` value. Plaintext (legacy) values pass through. A corrupt
/// or undecryptable envelope yields an empty string rather than leaking data.
pub fn decrypt_text(value: &str) -> String {
    let Some(encoded) = value.strip_prefix(MARKER) else {
        return value.to_string();
    };
    let Some(key) = current_key() else {
        return String::new();
    };
    decrypt_with(&key, encoded)
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}

fn encrypt_field(field: &mut String) {
    *field = encrypt_text(field);
}

fn decrypt_field(field: &mut String) {
    *field = decrypt_text(field);
}

pub fn encrypt_profile_secrets(profile: &mut BrowserProfile) {
    encrypt_field(&mut profile.proxy);
    encrypt_field(&mut profile.proxy_username);
    encrypt_field(&mut profile.proxy_password);
    encrypt_field(&mut profile.cookies_json);
}

pub fn decrypt_profile_secrets(profile: &mut BrowserProfile) {
    decrypt_field(&mut profile.proxy);
    decrypt_field(&mut profile.proxy_username);
    decrypt_field(&mut profile.proxy_password);
    decrypt_field(&mut profile.cookies_json);
}

pub fn encrypt_proxy_secrets(entry: &mut ProxyEntry) {
    encrypt_field(&mut entry.username);
    encrypt_field(&mut entry.password);
    encrypt_field(&mut entry.url);
}

pub fn decrypt_proxy_secrets(entry: &mut ProxyEntry) {
    decrypt_field(&mut entry.username);
    decrypt_field(&mut entry.password);
    decrypt_field(&mut entry.url);
}
