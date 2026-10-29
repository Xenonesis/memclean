fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    // Only embed the elevated app manifest for actual binary targets. When Cargo
    // builds test executables, the [test]/lib harness, or example/bench binaries,
    // CARGO_BIN_NAME is unset — embedding `requireAdministrator` there causes OS
    // error 740 ("The requested operation requires elevation") so the tests cannot
    // be launched unelevated.
    #[cfg(windows)]
    if std::env::var_os("CARGO_BIN_NAME").is_some() {
        let mut res = winresource::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        let _ = res.compile();
    }
}
