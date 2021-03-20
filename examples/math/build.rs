use std::{env, ffi::OsString, path::{Path, PathBuf}, str::FromStr};

use dyn_bindgen::*;

fn main() {
    let library_name = OsString::from_str("math").unwrap();
    let library_filename = libloading::library_filename(library_name);
    let library_filename = library_filename
        .to_str()
        .expect("library file name was invalid");
    let library_path = &format!("target/{}", library_filename);

    // CC doesn't allow us to generate a .so/.dll/.dylib so we do it outselves.
    let mut command = cc::Build::new()
        .file("src/math.c")
        .shared_flag(true)
        .get_compiler()
        .to_command();
    command
        .arg("src/math.c")
        .arg("-o")
        .arg(library_path)
        .output()
        .expect("could not compile math shared library");

    let builder = bindgen::builder().header("./src/math.h");
    let bundle = Bundle::File(Path::new(library_path).into());
    let config = Config {
        loading_strategy: LoadingStrategy::ImplicitlyLoadedBundle(bundle),
        use_rust_fmt: false,
    };
    let code = generate(builder, config).unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    std::fs::write(out_path.join("bindings.rs"), code).unwrap();
}
