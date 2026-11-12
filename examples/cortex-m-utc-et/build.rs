fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("arm") {
        let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        std::fs::copy("memory.x", out.join("memory.x")).unwrap();
        println!("cargo:rustc-link-search={}", out.display());
        println!("cargo:rustc-link-arg-bin=cortex-m-utc-et=-Tlink.x");
        println!("cargo:rerun-if-changed=memory.x");
    }
}
