extern crate bindgen;
extern crate gcc;

use std::env;
use std::path::PathBuf;


fn main() {
    let res = bindgen::builder()
        .clang_arg("-Izydis-c/include")
        .clang_arg("-Izydis-c/src")
        .clang_arg("-Isrc")
        .clang_arg("-DZYDIS_ENABLE_FEATURE_DECODER")
        .clang_arg("-DZYDIS_ENABLE_FEATURE_ENCODER")
        .clang_arg("-DZYDIS_ENABLE_FEATURE_EVEX")
        .clang_arg("-DZYDIS_ENABLE_FEATURE_MVEX")
        .clang_arg("-DZYDIS_ENABLE_FEATURE_FLAGS")
        .clang_arg("zydis-c/include/Zydis/Zydis.h")
        .constified_enum("Zydis.*")
        .prepend_enum_name(false)
        .unstable_rust(false)
        .generate();

    let res = match res {
        Ok(r) => r,
        Err(e) => panic!("{:?}", e)
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    if let Err(e) = res.write_to_file(out_path.join("bindings.rs")) {
        panic!("{:?}", e);
    }

    gcc::Build::new()
        .define("ZYDIS_ENABLE_FEATURE_DECODER", None)
        .define("ZYDIS_ENABLE_FEATURE_ENCODER", None)
        .define("ZYDIS_ENABLE_FEATURE_EVEX", None)
        .define("ZYDIS_ENABLE_FEATURE_MVEX", None)
        .define("ZYDIS_ENABLE_FEATURE_FLAGS", None)
        .include("zydis-c/include")
        .include("zydis-c/src")
        .include("src")
        .file("zydis-c/src/Mnemonic.c")
        .file("zydis-c/src/Register.c")
        .file("zydis-c/src/SharedData.c")
        .file("zydis-c/src/Utils.c")
        .file("zydis-c/src/Zydis.c")
        .file("zydis-c/src/Decoder.c")
        .file("zydis-c/src/DecoderData.c")
        .file("zydis-c/src/Formatter.c")
        //.file("zydis-c/src/Encoder.c")
        //.file("zydis-c/src/EncoderData.c")
        .compile("libzydis.a");
}
