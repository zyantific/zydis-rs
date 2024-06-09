use std::env;

fn bool2cmake(x: bool) -> &'static str {
    if x {
        "ON"
    } else {
        "OFF"
    }
}

fn build_library() {
    let mut config = cmake::Config::new("zydis-c");

    config
        .define("ZYDIS_BUILD_EXAMPLES", "OFF")
        .define("ZYDIS_BUILD_TOOLS", "OFF")
        .define("ZYDIS_BUILD_TESTS", "OFF")
        .define("ZYDIS_BUILD_DOXYGEN", "OFF")
        .define("ZYDIS_FEATURE_DECODER", "ON");

    config.define(
        "ZYDIS_MINIMAL_MODE",
        bool2cmake(!env::var("CARGO_FEATURE_FULL_DECODER").is_ok()),
    );
    config.define(
        "ZYDIS_FEATURE_FORMATTER",
        bool2cmake(env::var("CARGO_FEATURE_FORMATTER").is_ok()),
    );
    config.define(
        "ZYDIS_FEATURE_ENCODER",
        bool2cmake(env::var("CARGO_FEATURE_ENCODER").is_ok()),
    );
    config.define(
        "ZYAN_NO_LIBC",
        bool2cmake(env::var("CARGO_FEATURE_NOLIBC").is_ok()),
    );

    let target = env::var("TARGET").unwrap_or("(unknown)".to_string());
    let is_msvc = target.ends_with("windows-msvc");

    if env::var("CARGO_FEATURE_NO_STACK_PROTECTOR").is_ok() {
        if is_msvc {
            config.cflag("/GS-");
        } else {
            config.cflag("-fno-stack-protector");
        }
    }

    let dst = config.build();
    let relative_build_dir = if is_msvc { config.get_profile() } else { "" };

    println!(
        "cargo:rustc-link-search=native={}/build/{}",
        dst.display(),
        relative_build_dir
    );
    println!(
        "cargo:rustc-link-search=native={}/build/zycore/{}",
        dst.display(),
        relative_build_dir
    );

    println!("cargo:rustc-link-lib=static=Zydis");
    println!("cargo:rustc-link-lib=static=Zycore");
}

fn main() {
    println!("cargo:rerun-if-changed=zydis-c");

    build_library();
}
