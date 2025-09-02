use std::{env, path::PathBuf};

fn main() {
    let dst = cmake::Config::new("bolt").build_target("bolt").build();

    println!(
        "cargo:rustc-link-search=native={}/build/bolt",
        dst.display()
    );
    println!("cargo:rustc-link-lib=static=bolt");

    let bindings = bindgen::Builder::default()
        .header("./bolt/bolt/bolt.h")
        .header("./bolt/bolt/bt_context.h")
        .header("./bolt/bolt/bt_value.h")
        .header("./bolt/bolt/bt_object.h")
        .header("./bolt/bolt/bt_type.h")
        .header("./bolt/bolt/bt_prelude.h")
        .header("./bolt/bolt/bt_buffer.h")
        .header("./bolt/bolt/bt_compiler.h")
        .header("./bolt/bolt/bt_parser.h")
        .header("./bolt/bolt/bt_tokenizer.h")
        .header("./bolt/bolt/bt_gc.h")
        .header("./bolt/bolt/bt_debug.h")
        .header("./bolt/bolt/bt_embedding.h")
        .header("./bolt/bolt/bt_userdata.h")
        .header("./bolt/bolt/boltstd/boltstd.h")
        .header("./bolt/bolt/boltstd/boltstd_core.h")
        .header("./bolt/bolt/boltstd/boltstd_arrays.h")
        .header("./bolt/bolt/boltstd/boltstd_strings.h")
        .header("./bolt/bolt/boltstd/boltstd_tables.h")
        .header("./bolt/bolt/boltstd/boltstd_math.h")
        .header("./bolt/bolt/boltstd/boltstd_io.h")
        .header("./bolt/bolt/boltstd/boltstd_meta.h")
        .header("./bolt/bolt/boltstd/boltstd_regex.h")
        .derive_debug(true)
        .derive_copy(true)
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
