//! Build script for tinyxml2-capi.
//!
//! Uses cbindgen to generate a C header file at `include/tinyxml2.h`.

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_dir = std::path::Path::new(&crate_dir).join("include");

    // Ensure the output directory exists.
    std::fs::create_dir_all(&output_dir).expect("Failed to create include/ directory");

    let config =
        cbindgen::Config::from_file(std::path::Path::new(&crate_dir).join("cbindgen.toml"))
            .expect("Failed to read cbindgen.toml");

    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
        .expect("Failed to generate C bindings")
        .write_to_file(output_dir.join("tinyxml2.h"));
}
