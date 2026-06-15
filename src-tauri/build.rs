fn main() {
    // Only run the Tauri build step for the GUI build; the headless binary
    // (built with --no-default-features) has no `tauri` dependency.
    if std::env::var_os("CARGO_FEATURE_GUI").is_some() {
        tauri_build::build();
    }
}
