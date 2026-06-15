#![cfg_attr(all(not(debug_assertions), feature = "gui"), windows_subsystem = "windows")]

fn main() {
    #[cfg(feature = "gui")]
    cloak_fingerprint_controller_lib::run();

    // Without the GUI feature this binary does nothing useful; the headless
    // server lives in the `cloak-headless` binary.
    #[cfg(not(feature = "gui"))]
    eprintln!("Built without the `gui` feature. Use the `cloak-headless` binary instead.");
}
