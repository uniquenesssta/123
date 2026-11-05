fn main() {
    tauri_build::build();
    println!("cargo:rerun-if-env-changed=FOOTBALL_NATIVE_LOG_ACCEPTANCE_BUILD");
    println!("cargo:rustc-check-cfg=cfg(football_native_log_acceptance)");
    // The lib test host also links native dialogs; activate the same Common Controls v6
    // dependency as Tauri's application manifest. No new privileges or dependencies.
    if std::env::var("FOOTBALL_NATIVE_LOG_ACCEPTANCE_BUILD").as_deref() == Ok("1")
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        println!("cargo:rustc-cfg=football_native_log_acceptance");
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTUAC:NO");
        println!("cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'");
    }
}
