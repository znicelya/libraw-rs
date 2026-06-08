use std::env;
use std::path::PathBuf;

fn main() {
    // Link pre-built static library based on target OS
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-search=native=static_lib/win");
        println!("cargo:rustc-link-lib=static=libraw_static");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=ws2_32");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=native=static_lib/mac");
        println!("cargo:rustc-link-lib=static=raw_r");
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=z");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-search=native=static_lib/linux");
        println!("cargo:rustc-link-lib=static=raw");
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=z");
    }

    // Generate bindings from LibRaw C API headers
    let bindings = bindgen::Builder::default()
        .header("vendor/LibRaw/libraw/libraw.h")
        .clang_arg("-Ivendor/LibRaw")
        .allowlist_function("libraw_.*")
        .allowlist_type("libraw_.*")
        .allowlist_type("LibRaw_.*")
        .allowlist_var("LIBRAW_.*")
        .opaque_type("std::.*")
        .size_t_is_usize(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=vendor/LibRaw/libraw/");
    println!("cargo:rerun-if-changed=static_lib/");
}
