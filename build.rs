extern crate bindgen;
extern crate cmake;

use std::env;
use std::path::PathBuf;

use bindgen::{Builder, EnumVariation};

const ZYDIS_INCLUDE_PATH: &'static str = "zydis-c/include";
const ZYDIS_SRC_PATH: &'static str = "zydis-c/src";
const ZYDIS_ZYCORE_PATH: &'static str = "zydis-c/dependencies/zycore/include";

fn build_library() {
    let mut config = cmake::Config::new("zydis-c");

    config
        .define("ZYDIS_BUILD_EXAMPLES", "OFF")
        .define("ZYDIS_BUILD_TOOLS", "OFF");

    if cfg!(feature = "no_libc") {
        config.define("ZYDIS_NO_LIBC", "ON");
    }

    let dst = config.build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=zydis");
}

fn build_bindings(out_path: PathBuf) {
    let host = env::var("HOST").unwrap();
    let target = env::var("TARGET").unwrap();

    let mut builder = Builder::default()
        .header(format!("{}/Zydis/Zydis.h", ZYDIS_INCLUDE_PATH))
        .clang_arg(format!("-I{}", ZYDIS_INCLUDE_PATH))
        .clang_arg(format!("-I{}", ZYDIS_SRC_PATH))
        .clang_arg(format!("-I{}", ZYDIS_ZYCORE_PATH))
        .clang_arg("-Isrc")
        .default_enum_style(EnumVariation::Consts)
        .whitelist_type("Zydis.*")
        .whitelist_function("Zydis.*");

    if target != host {
        // For some reason we get strange problems with the sysroot if we always add this line and
        // we're not cross compiling. "stddef.h" is not being found in that case, for what ever
        // reason (at least on a linux host).
        //
        // Thus we condtionally set this argument if we're cross compiling.
        builder = builder.clang_arg(format!("--target={}", target.as_str()))
    }

    let bindings = builder
        .emit_builtins()
        .layout_tests(true)
        .prepend_enum_name(false)
        .generate()
        .expect("Could not generate bindings to zydis");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings");
}

fn main() {
    println!("cargo:rerun-if-changed=zydis-c");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    build_library();
    build_bindings(out_path);
}
