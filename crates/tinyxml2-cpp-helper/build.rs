fn main() {
    cc::Build::new()
        .cpp(true)
        .file("../../include/tinyxml2.cpp")
        .file("src/main.cpp")
        .include("../../include")
        .compile("tinyxml2_cpp");
    println!("cargo:rerun-if-changed=../../include/tinyxml2.cpp");
    println!("cargo:rerun-if-changed=../../include/tinyxml2.h");
    println!("cargo:rerun-if-changed=src/main.cpp");
}
