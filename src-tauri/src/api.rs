//! Embedded local automation HTTP API.
//!
//! Mirrors the "local API" pattern of commercial fingerprint browsers
//! (AdsPower/BitBrowser): automation scripts hit `http://127.0.0.1:<port>` to
//! start/stop profiles and receive the DevTools WebSocket endpoint that
//! Puppeteer/Playwright/Selenium connect to. Tauri-agnostic: the desktop GUI
//! and the headless server both start it with an `Arc<RuntimeState>`.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use tiny_http::{Header, Method, Response, Server};
use uuid::Uuid;

use crate::launch::{collect_running_ids, launch_profiles_batch, stop_running, RuntimeState};
use crate::profile::new_profile;
use crate::store::{data_root_path, load_store, save_store};

/// Preferred fixed port (AdsPower-style), falling back to an ephemeral port.
const PREFERRED_PORT: u16 = 50327;

/// Start the API server in a background thread. Returns the bound port.
pub fn start(runtime: Arc<RuntimeState>) -> Option<u16> {
    let server = Server::http(format!("127.0.0.1:{PREFERRED_PORT}"))
        .or_else(|_| Server::http("127.0.0.1:0"))
        .ok()?;
    let port = server.server_addr().to_ip().map(|a| a.port()).unwrap_or(0);
    let token = Uuid::new_v4().simple().to_string();

    *runtime.api_port.lock().unwrap() = port;
    *runtime.api_token.lock().unwrap() = token.clone();

    thread::spawn(move || {
        for request in server.incoming_requests() {
            handle_request(&runtime, &token, request);
        }
    });
    Some(port)
}

fn json_response(status: u16, body: serde_json::Value) -> Response<std::io::Cursor<Vec<u8>>> {
    let data = body.to_string().into_bytes();
    let mut response = Response::from_data(data).with_status_code(status);
    if let Ok(header) = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]) {
        response.add_header(header);
    }
    if let Ok(header) = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]) {
        response.add_header(header);
    }
    response
}

fn parse_query(url: &str) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    let (path, query) = match url.split_once('?') {
        Some((p, q)) => (p.to_string(), q),
        None => (url.to_string(), ""),
    };
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        params.insert(urldecode(key), urldecode(value));
    }
    (path, params)
}

fn urldecode(input: &str) -> String {
    let bytes = input.replace('+', " ");
    let bytes = bytes.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn handle_request(runtime: &RuntimeState, token: &str, request: tiny_http::Request) {
    let (path, params) = parse_query(request.url());
    let method = request.method().clone();

    if method == Method::Options {
        let _ = request.respond(json_response(200, serde_json::json!({"ok": true})));
        return;
    }

    let response = route(runtime, token, &path, &params);
    let _ = request.respond(response);
}

fn route(
    runtime: &RuntimeState,
    token: &str,
    path: &str,
    params: &HashMap<String, String>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    match path {
        "/status" | "/api/v1/status" => {
            let running = collect_running_ids(runtime);
            json_response(
                200,
                serde_json::json!({
                    "code": 0,
                    "ok": true,
                    "version": env!("CARGO_PKG_VERSION"),
                    "headless": runtime.headless,
                    "running": running,
                }),
            )
        }
        "/api/v1/profiles" => {
            let store = load_store(&data_root_path());
            let running: std::collections::HashSet<String> =
                collect_running_ids(runtime).into_iter().collect();
            let list: Vec<serde_json::Value> = store
                .profiles
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "projectId": p.project_id,
                        "running": running.contains(&p.id),
                    })
                })
                .collect();
            json_response(200, serde_json::json!({"code": 0, "profiles": list}))
        }
        "/api/v1/profile/create" => {
            if !authorized(token, params) {
                return unauthorized();
            }
            let data_root = data_root_path();
            let mut store = load_store(&data_root);
            let project_id = params
                .get("projectId")
                .cloned()
                .filter(|id| store.projects.iter().any(|p| &p.id == id))
                .unwrap_or_else(|| store.active_project_id.clone());
            let profile = new_profile(project_id, params.get("name").cloned());
            let id = profile.id.clone();
            store.profiles.push(profile);
            match save_store(&data_root, &store) {
                Ok(()) => json_response(200, serde_json::json!({"code": 0, "data": {"id": id}})),
                Err(err) => json_response(500, serde_json::json!({"code": 1, "msg": err})),
            }
        }
        "/api/v1/profile/delete" => {
            if !authorized(token, params) {
                return unauthorized();
            }
            let Some(id) = params.get("id").cloned() else {
                return json_response(400, serde_json::json!({"code": 1, "msg": "missing id"}));
            };
            stop_running(runtime, &id);
            let data_root = data_root_path();
            let mut store = load_store(&data_root);
            store.profiles.retain(|p| p.id != id);
            let _ = save_store(&data_root, &store);
            json_response(200, serde_json::json!({"code": 0, "data": {"id": id}}))
        }
        "/api/v1/browser/start" => {
            if !authorized(token, params) {
                return unauthorized();
            }
            let Some(id) = params.get("id").cloned() else {
                return json_response(400, serde_json::json!({"code": 1, "msg": "missing id"}));
            };
            let url_override = params.get("url").cloned();
            // Per-request headless override: ?headless=0|1 (default = server mode).
            let headless_override = params.get("headless").map(|v| {
                matches!(v.as_str(), "1" | "true" | "yes" | "on")
            });
            match launch_profiles_batch(runtime, vec![id.clone()], true, url_override, headless_override) {
                Ok(results) => {
                    let endpoint = runtime.endpoint_snapshot().get(&id).cloned();
                    let preview = results.first().map(|r| r.command_preview.clone());
                    json_response(
                        200,
                        serde_json::json!({
                            "code": 0,
                            "data": {
                                "id": id,
                                "ws": endpoint.as_ref().map(|e| e.ws_endpoint.clone()),
                                "debugPort": endpoint.as_ref().map(|e| e.debug_port),
                                "http": endpoint.as_ref().map(|e| e.http_endpoint.clone()),
                                "debuggerAddress": endpoint.as_ref().map(|e| format!("127.0.0.1:{}", e.debug_port)),
                            },
                            "preview": preview,
                        }),
                    )
                }
                Err(err) => json_response(500, serde_json::json!({"code": 1, "msg": err})),
            }
        }
        "/api/v1/browser/stop" => {
            if !authorized(token, params) {
                return unauthorized();
            }
            let Some(id) = params.get("id").cloned() else {
                return json_response(400, serde_json::json!({"code": 1, "msg": "missing id"}));
            };
            stop_running(runtime, &id);
            json_response(200, serde_json::json!({"code": 0, "data": {"id": id}}))
        }
        "/api/v1/browser/active" => json_response(
            200,
            serde_json::json!({"code": 0, "data": runtime.endpoint_snapshot()}),
        ),
        _ => json_response(404, serde_json::json!({"code": 1, "msg": "not found"})),
    }
}

fn authorized(token: &str, params: &HashMap<String, String>) -> bool {
    params.get("token").map(String::as_str) == Some(token)
}

fn unauthorized() -> Response<std::io::Cursor<Vec<u8>>> {
    json_response(401, serde_json::json!({"code": 1, "msg": "invalid token"}))
}
