extern crate cmake;

use std::env;

fn build_library() {
    let mut config = cmake::Config::new("zydis-c");

    config
        .define("ZYDIS_BUILD_EXAMPLES", "OFF")
        .define("ZYDIS_BUILD_TOOLS", "OFF");

    let dst = config.build();

    let target = env::var("TARGET").unwrap_or("(unknown)".to_string());
    let profile = env::var("PROFILE").unwrap_or("(unknown)".to_string());
    let is_msvc = target.ends_with("windows-msvc");

    let relative_build_dir = if is_msvc {
        // ref: https://docs.rs/cmake/0.1.24/src/cmake/lib.rs.html#323
        match &profile[..] {
            "bench" | "release" => "Release",
            _ => "Debug",
        }
    } else { "" };

    println!(
        "cargo:rustc-link-search=native={}/build/{}",
        dst.display(),
        relative_build_dir);
    println!(
        "cargo:rustc-link-search=native={}/build/dependencies/zycore/{}",
        dst.display(),
        relative_build_dir
    );

    println!("cargo:rustc-link-lib=static=Zydis");
    println!("cargo:rustc-link-lib=static=Zycore");
}

fn main() {
    println!("cargo:rerun-if-changed=zydis-c");
    println!("cargo:rerun-if-changed=src/ZycoreExportConfig.h");
    println!("cargo:rerun-if-changed=src/ZydisExportConfig.h");

    build_library();
}
