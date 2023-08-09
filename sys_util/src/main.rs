// extern crate bindgen;
// extern crate cc;

// use std::env;
// use std::path::PathBuf;

fn main() {
    // const EXPERIMENTAL_API: &str = "REDISMODULE_EXPERIMENTAL_API";

    // // Determine if the `experimental-api` feature is enabled
    // fn experimental_api() -> bool {
    //     std::env::var_os("CARGO_FEATURE_EXPERIMENTAL_API").is_some()
    // }

    // let mut build = bindgen::Builder::default();

    // if experimental_api() {
    //     build = build.clang_arg(format!("-D{}", EXPERIMENTAL_API).as_str());
    // }

    // let bindings = build
    //     .header("src/include/bindings.h")
    //     .generate()
    //     .expect("error generating bindings");

    // let out_path = PathBuf::from(env::var("TOASTATE_TOASTAINER_BINDINGS_OUTPUT_DIR").unwrap());
    // bindings
    //     .write_to_file(out_path.join("mod.rs"))
    //     .expect("failed to write bindings to file");
}