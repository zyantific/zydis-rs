extern crate cmake;

use std::env;

/// Determine the CMAKE_BUILD_TYPE profile that will be used given the current
/// build options.
///
/// Note: this implementation comes from the `cmake-rs` crate and should stay
/// in sync.
///
/// Via that project: the profile is automatically inferred from Rust's
/// compilation profile as follows:
///
/// * if `opt-level=0` then `CMAKE_BUILD_TYPE=Debug`,
/// * if `opt-level={1,2,3}` and:
///   * `debug=false` then `CMAKE_BUILD_TYPE=Release`
///   * otherwise `CMAKE_BUILD_TYPE=RelWithDebInfo`
/// * if `opt-level={s,z}` then `CMAKE_BUILD_TYPE=MinSizeRel`
fn get_cmake_profile() -> &'static str {
    // via: https://github.com/alexcrichton/cmake-rs/blob/master/src/lib.rs#L427

    // Determine Rust's profile, optimization level, and debug info:
    #[derive(PartialEq)]
    enum RustProfile {
        Debug,
        Release,
    }
    #[derive(PartialEq, Debug)]
    enum OptLevel {
        Debug,
        Release,
        Size,
    }

    let rust_profile = match &env::var("PROFILE").unwrap()[..] {
        "debug" => RustProfile::Debug,
        "release" | "bench" => RustProfile::Release,
        unknown => {
            eprintln!(
                "Warning: unknown Rust profile={}; defaulting to a release build.",
                unknown
            );
            RustProfile::Release
        }
    };

    let opt_level = match &env::var("OPT_LEVEL").unwrap()[..] {
        "0" => OptLevel::Debug,
        "1" | "2" | "3" => OptLevel::Release,
        "s" | "z" => OptLevel::Size,
        unknown => {
            let default_opt_level = match rust_profile {
                RustProfile::Debug => OptLevel::Debug,
                RustProfile::Release => OptLevel::Release,
            };
            eprintln!(
                "Warning: unknown opt-level={}; defaulting to a {:?} build.",
                unknown, default_opt_level
            );
            default_opt_level
        }
    };

    let debug_info: bool = match &env::var("DEBUG").unwrap()[..] {
        "false" => false,
        "true" => true,
        unknown => {
            eprintln!("Warning: unknown debug={}; defaulting to `true`.", unknown);
            true
        }
    };

    match (opt_level, debug_info) {
        (OptLevel::Debug, _) => "Debug",
        (OptLevel::Release, false) => "Release",
        (OptLevel::Release, true) => "RelWithDebInfo",
        (OptLevel::Size, _) => "MinSizeRel",
    }
}

fn build_library() {
    let mut config = cmake::Config::new("zydis-c");

    config
        .define("ZYDIS_BUILD_EXAMPLES", "OFF")
        .define("ZYDIS_BUILD_TOOLS", "OFF");

    if env::var("CARGO_FEATURE_MINIMAL").is_ok() {
        config.define("ZYDIS_MINIMAL_MODE", "ON");
    }

    let dst = config.build();

    let target = env::var("TARGET").unwrap_or("(unknown)".to_string());
    let is_msvc = target.ends_with("windows-msvc");

    let relative_build_dir = if is_msvc { get_cmake_profile() } else { "" };

    println!(
        "cargo:rustc-link-search=native={}/build/{}",
        dst.display(),
        relative_build_dir
    );
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
