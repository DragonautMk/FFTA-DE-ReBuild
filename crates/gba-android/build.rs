use std::{fs,path::{Path,PathBuf}};
fn collect(path:&Path,files:&mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let path=entry.unwrap().path();
        if path.is_dir(){collect(&path,files)}
        else if path.extension().is_some_and(|ext|ext=="rs"){files.push(path)}
    }
}
fn main() {
    let directory=PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let mut files=vec![directory.join("build.rs"),directory.join("../../scripts/android/build_android.py")];
    for path in [directory.join("src"),directory.join("../gba-core/src"),directory.join("../recomp-core/src"),directory.join("../armv4t/src")] {
        collect(&path,&mut files);
    }
    files.sort();
    let mut hash=0xcbf29ce484222325u64;
    for file in files {
        println!("cargo:rerun-if-changed={}",file.display());
        for byte in fs::read(file).unwrap() {hash=(hash^byte as u64).wrapping_mul(0x100000001b3)}
    }
    for byte in std::env::var("TARGET").unwrap().bytes(){hash=(hash^byte as u64).wrapping_mul(0x100000001b3)}
    println!("cargo:rustc-env=FFTA_ANDROID_CACHE_TAG={hash:016x}");
}
