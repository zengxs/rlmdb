use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    let lmdb_dir = PathBuf::from(&manifest_dir).join("lmdb/libraries/liblmdb");

    println!("cargo:rerun-if-changed=wrapper.h");

    let mut builder = cc::Build::new();
    builder
        .include(&lmdb_dir)
        .flag("-std=c11")
        .flag_if_supported("-Wno-unused-parameter")
        .file(lmdb_dir.join("mdb.c"))
        .file(lmdb_dir.join("midl.c"));
    if target_os == "android" {
        builder.define("ANDROID", "1");
    }
    builder.compile("lmdb");

    bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", lmdb_dir.display()))
        .rustified_enum("MDB_cursor_op")
        .blocklist_item("__glibc_c99_flexarr_available")
        .blocklist_item("__have_pthread_attr_t")
        .blocklist_item("__clock_t_defined")
        .blocklist_item("__clockid_t_defined")
        .blocklist_item("__time_t_defined")
        .blocklist_item("__timer_t_defined")
        .blocklist_item("__sigset_t_defined")
        .blocklist_item("__timeval_defined")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(PathBuf::from(&out_dir).join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
