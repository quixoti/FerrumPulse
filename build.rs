fn main() {
    let manifest = if cfg!(debug_assertions) {
        include_str!("app.manifest.dev")
    } else {
        include_str!("app.manifest")
    };
    let windows = tauri_build::WindowsAttributes::new().app_manifest(manifest);
    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("failed to run tauri-build");
}
