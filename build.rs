extern crate bindgen;
extern crate gcc;

use std::env;
use std::path::PathBuf;

use bindgen::Builder;
use gcc::Build;

const ZYDIS_INCLUDE_PATH: &'static str = "zydis-c/include";
const ZYDIS_SRC_PATH: &'static str = "zydis-c/src";
const ZYDIS_DEFINES: &'static [&'static str] = &[
    "ZYDIS_ENABLE_FEATURE_EVEX",
    "ZYDIS_ENABLE_FEATURE_MVEX",
    "ZYDIS_ENABLE_FEATURE_FLAGS",
    "ZYDIS_ENABLE_FEATURE_DECODER",
    "ZYDIS_ENABLE_FEATURE_ENCODER",
];

fn build_library() {
    let mut build = Build::new();

    for cur_define in ZYDIS_DEFINES {
        build.define(cur_define, None);
    }

    build
        .include(ZYDIS_INCLUDE_PATH)
        .include(ZYDIS_SRC_PATH)
        .include("src")
        .files(vec![
            format!("{}/Decoder.c", ZYDIS_SRC_PATH),
            format!("{}/DecoderData.c", ZYDIS_SRC_PATH),
            format!("{}/MetaInfo.c", ZYDIS_SRC_PATH),
            format!("{}/Encoder.c", ZYDIS_SRC_PATH),
            format!("{}/EncoderData.c", ZYDIS_SRC_PATH),
            format!("{}/Formatter.c", ZYDIS_SRC_PATH),
            format!("{}/Mnemonic.c", ZYDIS_SRC_PATH),
            format!("{}/Register.c", ZYDIS_SRC_PATH),
            format!("{}/SharedData.c", ZYDIS_SRC_PATH),
            format!("{}/Utils.c", ZYDIS_SRC_PATH),
            format!("{}/Zydis.c", ZYDIS_SRC_PATH),
        ])
        .compile("libzydis.a");
}

fn build_bindings(out_path: PathBuf) {
    let mut bindings = Builder::default()
        .unstable_rust(false)
        .header(format!("{}/Zydis/Zydis.h", ZYDIS_INCLUDE_PATH))
        //.header(format!("{}/Zydis/Encoder.h", ZYDIS_INCLUDE_PATH))
        .clang_arg(format!("-I{}", ZYDIS_INCLUDE_PATH))
        .clang_arg(format!("-I{}", ZYDIS_SRC_PATH))
        .clang_arg("-Isrc")
        .emit_builtins()
        .constified_enum("Zydis.*")
        .layout_tests(true)
        .prepend_enum_name(false);

    for cur_define in ZYDIS_DEFINES {
        bindings = bindings.clang_arg(format!("-D{}", cur_define));
    }

    let bindings = bindings
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
