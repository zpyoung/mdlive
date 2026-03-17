fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap();
    let target_triple = std::env::var("TARGET").unwrap_or_else(|_| {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        match (arch, os) {
            ("aarch64", "macos") => "aarch64-apple-darwin".to_string(),
            ("x86_64", "macos") => "x86_64-apple-darwin".to_string(),
            ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_string(),
            _ => format!("{arch}-unknown-{os}"),
        }
    });

    let profile = if std::env::var("PROFILE").unwrap_or_default() == "release" {
        "release"
    } else {
        "debug"
    };

    let binaries_dir = manifest_dir.join("binaries");
    std::fs::create_dir_all(&binaries_dir).ok();

    let dest = binaries_dir.join(format!("mdlive-cli-{target_triple}"));
    let src = workspace_root.join("target").join(profile).join("mdlive");
    if src.exists() {
        std::fs::copy(&src, &dest)
            .unwrap_or_else(|_| panic!("failed to copy mdlive cli to binaries/"));
    } else if !dest.exists() {
        // placeholder so tauri build doesn't fail before cli is built
        std::fs::write(&dest, "").ok();
    }

    tauri_build::build()
}
