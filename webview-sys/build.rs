extern crate bindgen;
extern crate cc;
extern crate pkg_config;

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

// TODO: Remove binding generation
// and rework this build into something saner
fn main() {
    let outdir = PathBuf::from("src");
    let target = env::var("TARGET").unwrap();
    let mut build = cc::Build::new();
    let mut bindings = bindgen::Builder::default()
        .header("webview/webview.h")
        .whitelist_type("webview_priv")
        .opaque_type("webview_priv");
    let binding_fpath;

    let webview_path: PathBuf = match env::var("WEBVIEW_DIR") {
        Ok(path) => path.into(),
        Err(_) => {
            // Initialize webview submodule if user forgot to clone parent repository with --recursive.
            if !Path::new("webview/.git").exists() {
                let _ = Command::new("git")
                    .args(&["submodule", "update", "--init"])
                    .status();
            }
            "webview".into()
        }
    };

    build
        .include(&webview_path)
        .file("webview.c")
        .flag_if_supported("-std=c11")
        .flag_if_supported("-w");


    if env::var("DEBUG").is_err() {
        build.define("NDEBUG", None);
    } else {
        build.define("DEBUG", None);
    }

    if target.contains("windows") {
        build.define("WEBVIEW_WINAPI", None);

        for &lib in &["ole32", "comctl32", "oleaut32", "uuid", "gdi32"] {
            println!("cargo:rustc-link-lib={}", lib);
        }

        bindings = bindings.clang_arg("-D WEBVIEW_WINAPI");
        binding_fpath = "webview_bind_win.rs";
    } else if target.contains("linux") || target.contains("bsd") {
        let webkit = pkg_config::Config::new()
            .atleast_version("2.8")
            .probe("webkit2gtk-4.0")
            .unwrap();

        for path in webkit.include_paths {
            bindings = bindings.clang_arg(
                format!("-I{}",
                    path
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap()
                )
            );

            build.include(path);
        }

        bindings = bindings.clang_arg("-D WEBVIEW_GTK");
        binding_fpath = "webview_bind.rs";
        build.define("WEBVIEW_GTK", None);
    } else if target.contains("apple") {
        build
            .define("WEBVIEW_COCOA", None)
            .flag("-x")
            .flag("objective-c");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");

        bindings = bindings.clang_arg("-D WEBVIEW_COCOA");
        binding_fpath = "webview_bind_osx.rs";
    } else {
        panic!("unsupported target");
    }

    let path = outdir.join(binding_fpath);

    if !path.exists() {
        bindings
            .generate()
            .unwrap()
            .write_to_file(path)
            .expect("Unable to generate webview bindings!");
    }

    build.compile("webview");
}
