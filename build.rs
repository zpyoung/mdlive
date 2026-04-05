use std::process::Command;

fn main() {
    minijinja_embed::embed_templates!("templates", &[".html"]);

    let version = env!("CARGO_PKG_VERSION").to_string();
    let version = if std::env::var("MDLIVE_DEV").is_ok() {
        let sha = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        format!("{}-dev.{}", version, sha.trim())
    } else {
        version
    };
    println!("cargo:rustc-env=MDLIVE_VERSION={version}");
    println!("cargo:rerun-if-env-changed=MDLIVE_DEV");
}
