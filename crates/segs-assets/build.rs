//! `build.rs`

#[cfg(windows)]
const HOST_FAMILY: &str = "windows";

#[cfg(unix)]
const HOST_FAMILY: &str = "unix";

fn main() {
    println!("cargo::rustc-check-cfg=cfg(host_family, values(\"windows\"))");

    #[cfg(any(windows, unix))]
    println!("cargo:rust-cfg=host_family={}", HOST_FAMILY);
}
