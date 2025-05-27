// Advanced build customizations
use std::process::Command;

fn main() {
    // Embed Git commit hash
    let git_output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap();

    let git_hash = String::from_utf8(git_output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash.trim());

    // Embed build timestamp
    let build_time = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
}
