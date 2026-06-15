//! Headless automation server binary for Ubuntu/Linux (or any OS) without a GUI.
//!
//! Build: `cargo build --release --bin cloak-headless --no-default-features`
//! Run:   `./cloak-headless`  (configure the browser path/data root via env or
//!        a desktop install's settings.json under the data root).

fn main() {
    cloak_fingerprint_controller_lib::headless_main();
}
