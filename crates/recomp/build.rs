fn git(args: &[&str]) -> String {
    std::process::Command::new("git").args(args).output().ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned()).unwrap_or_else(|| "unknown".into())
}

fn main() {
    // Build-time provenance, never inferred from the checkout at report time.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs");
    println!("cargo:rerun-if-changed=../");
    println!("cargo:rerun-if-changed=../../Cargo.lock");
    println!("cargo:rustc-env=FFTA_BUILD_REVISION={}", git(&["rev-parse", "HEAD"]));
    let status = git(&["status", "--porcelain", "--untracked-files=normal"]);
    let dirty = if status == "unknown" { "unknown" } else if status.is_empty() { "false" } else { "true" };
    println!("cargo:rustc-env=FFTA_BUILD_DIRTY={dirty}");
    println!("cargo:rustc-env=FFTA_BUILD_TARGET={}", std::env::var("TARGET").unwrap());
    println!("cargo:rustc-env=FFTA_BUILD_PROFILE={}", std::env::var("PROFILE").unwrap());
    println!("cargo:rerun-if-changed=../installer/assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../installer/assets/icon.ico");
        res.compile().unwrap();
    }
}
