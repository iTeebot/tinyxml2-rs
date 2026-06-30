fn main() {
    cc::Build::new()
        .cpp(true)
        .file("../../include/tinyxml2.cpp")
        .file("src/cpp_wrapper.cpp")
        .include("../../include")
        .compile("tinyxml2_cpp_bench");
    println!("cargo:rerun-if-changed=../../include/tinyxml2.cpp");
    println!("cargo:rerun-if-changed=../../include/tinyxml2.h");
    println!("cargo:rerun-if-changed=src/cpp_wrapper.cpp");
}
