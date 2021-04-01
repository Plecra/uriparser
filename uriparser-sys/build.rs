use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=static=lib/uriparser");

    println!(
        "cargo:rustc-link-search=native={}",
        cmake::Config::new("uriparser")
            .define("URIPARSER_BUILD_DOCS", "OFF")
            .define("URIPARSER_BUILD_TESTS", "OFF")
            .define("URIPARSER_BUILD_TOOLS", "OFF")
            .define("URIPARSER_BUILD_WCHAR_T", "OFF")
            .define("BUILD_SHARED_LIBS", "OFF")
            .build()
            .display()
    );

    println!("cargo:rerun-if-changed=uriparser/include/uriparser/Uri.h");
    bindgen::Builder::default()
        .header("uriparser/include/uriparser/Uri.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(true)
        .derive_default(true)
        .whitelist_type("Uri.*A")
        .whitelist_function("uri.*A")
        .whitelist_var("URI_.*")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
