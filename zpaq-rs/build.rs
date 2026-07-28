fn main() {
    cc::Build::new()
        .cpp(true)
        .file("../libzpaq.cpp")
        .file("../zpaq_ffi.cpp")
        .include("..")
        .flag("-march=native")
        .flag("-O3")
        .std("c++17")
        .define("unix", None)
        .compile("zpaq_ffi");

    println!("cargo:rerun-if-changed=../libzpaq.cpp");
    println!("cargo:rerun-if-changed=../libzpaq.h");
    println!("cargo:rerun-if-changed=../zpaq_ffi.cpp");
    println!("cargo:rerun-if-changed=../zpaq_ffi.h");
    println!("cargo:rerun-if-changed=build.rs");
}
