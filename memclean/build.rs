fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=memclean.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("memclean.ico");
        res.set_manifest_file("app.manifest");
        res.set("FileDescription", "MemClean - High-Performance Windows Memory & Latency Optimizer");
        res.set("ProductName", "MemClean");
        res.set("OriginalFilename", "memclean.exe");
        res.set("LegalCopyright", "Copyright (C) 2026 Xenonesis. All rights reserved.");
        res.set("CompanyName", "MemClean Open Source Project");
        res.set("ProductVersion", "1.1.0");
        res.set("FileVersion", "1.1.0.0");
        let _ = res.compile();
    }
}
