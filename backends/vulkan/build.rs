// The `ngx` feature links the DLSS SDK's static loader (`libnvsdk_ngx.a`, C++) from outside the
// repository: `NGX_SDK_DIR` is the SDK's root (NVIDIA/DLSS 310.9.16), with the library under
// `lib/Linux_x86_64`. Nothing of NVIDIA's is kept in the repository.
fn main() {
    println!("cargo:rerun-if-env-changed=NGX_SDK_DIR");
    if std::env::var_os("CARGO_FEATURE_NGX").is_none() {
        return;
    }
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if os != "linux" {
        println!("cargo:warning=the ngx feature links only on Linux (Streamline is the Windows route)");
        return;
    }
    let Some(dir) = std::env::var_os("NGX_SDK_DIR") else {
        // A `cargo check` needs no library; a build then fails to link and says why here.
        println!("cargo:warning=the ngx feature needs NGX_SDK_DIR (the DLSS SDK's root) to link");
        return;
    };
    let lib = std::path::Path::new(&dir).join("lib/Linux_x86_64");
    println!("cargo:rustc-link-search=native={}", lib.display());
    println!("cargo:rustc-link-lib=static=nvsdk_ngx");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=dl");
}
