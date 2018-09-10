extern crate cmake;

fn build_library() {
    let mut config = cmake::Config::new("zydis-c");

    config
        .define("ZYDIS_BUILD_EXAMPLES", "OFF")
        .define("ZYDIS_BUILD_TOOLS", "OFF");

    let dst = config.build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!(
        "cargo:rustc-link-search=native={}/build/dependencies/zycore",
        dst.display()
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
