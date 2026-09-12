use std::env;

const WINDOWS_ICON_PATH: &str = "assets/windows/segs2.ico";

/// Embeds the application icon and version metadata in Windows executables.
///
/// Returns an error when the target's resource compiler cannot generate or link
/// the resource file.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed={WINDOWS_ICON_PATH}");

    // Skip resource compilation for non-Windows targets
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }

    // Embed the icon in the executable for Explorer and installed shortcuts
    winresource::WindowsResource::new()
        .set_icon(WINDOWS_ICON_PATH)
        .compile()?;

    Ok(())
}
