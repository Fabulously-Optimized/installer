use std::{env, path::PathBuf, process::Command};

fn main() {
    let verifier_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("verifier")
        .canonicalize()
        .unwrap();
    Command::new("go")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg("dist/libverifier.a")
        .current_dir(&verifier_dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    println!("cargo::rustc-link-lib=verifier");
    println!("cargo::rustc-link-search={}", verifier_dir.join("dist").display());
    tauri_build::build()
}
