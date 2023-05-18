use std::env;

fn build_library() {
    let mut config = cmake::Config::new("zydis-c");

    config
        .define("ZYDIS_BUILD_EXAMPLES", "OFF")
        .define("ZYDIS_BUILD_TOOLS", "OFF");

    if env::var("CARGO_FEATURE_MINIMAL").is_ok() {
        config.define("ZYDIS_MINIMAL_MODE", "ON");
        config.define("ZYDIS_FEATURE_ENCODER", "OFF");
    }

    if env::var("CARGO_FEATURE_NOLIBC").is_ok() {
        config.define("ZYAN_NO_LIBC", "ON");
    }

    let dst = config.build();

    let target = env::var("TARGET").unwrap_or("(unknown)".to_string());
    let is_msvc = target.ends_with("windows-msvc");

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
