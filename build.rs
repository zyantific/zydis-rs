extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

use bindgen::Builder;
use cc::Build;

const ZYDIS_INCLUDE_PATH: &'static str = "zydis-c/include";
const ZYDIS_SRC_PATH: &'static str = "zydis-c/src";


fn build_library() {
    Build::new()
        .include(ZYDIS_INCLUDE_PATH)
        .include(ZYDIS_SRC_PATH)
        .include("src")
        .files(vec![
            format!("{}/MetaInfo.c", ZYDIS_SRC_PATH),
            format!("{}/Mnemonic.c", ZYDIS_SRC_PATH),
            format!("{}/Register.c", ZYDIS_SRC_PATH),
            format!("{}/SharedData.c", ZYDIS_SRC_PATH),
            format!("{}/String.c", ZYDIS_SRC_PATH),
            format!("{}/Utils.c", ZYDIS_SRC_PATH),
            format!("{}/Zydis.c", ZYDIS_SRC_PATH),
            format!("{}/Decoder.c", ZYDIS_SRC_PATH),
            format!("{}/DecoderData.c", ZYDIS_SRC_PATH),
            format!("{}/Formatter.c", ZYDIS_SRC_PATH),
        ])
        .compile("libzydis.a");
}

fn build_bindings(out_path: PathBuf) {
    let bindings = Builder::default()
        //.rust_target()
        .header(format!("{}/Zydis/Zydis.h", ZYDIS_INCLUDE_PATH))
        .clang_arg(format!("-I{}", ZYDIS_INCLUDE_PATH))
        .clang_arg(format!("-I{}", ZYDIS_SRC_PATH))
        .clang_arg("-Isrc")
        .emit_builtins()
        //.constified_enum_module("Zydis.*")
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
