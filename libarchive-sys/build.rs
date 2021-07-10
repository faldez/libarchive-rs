use std::{env, path::PathBuf};

fn main() {
    let mode = if cfg!(feature = "static") || cfg!(target_env = "msvc") {
        "static"
    } else {
        "dylib"
    };

    if cfg!(feature = "vendored") {
        build_libarchive();
    }

    if mode == "static" || cfg!(all(target_env = "msvc", feature = "vendored")) {
        link_deps(mode);
    }

    link_libarchive(mode);

    generate_binding();
}

fn generate_binding() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .blocklist_type("_?P?IMAGE_TLS_DIRECTORY.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn build_libarchive() {
    let dst = cmake::Config::new("libarchive")
        .define("ENABLE_TEST", "OFF")
        .define("ENABLE_CPIO:BOOL", "OFF")
        .define("ENABLE_CPIO_SHARED:BOOL", "OFF")
        .define("ENABLE_TAR:BOOL", "OFF")
        .define("ENABLE_TAR_SHARED:BOOL:BOOL", "OFF")
        .define("ENABLE_OPENSSL:BOOL", "OFF")
        .define("LIBARCHIVE_STATIC", "ON")
        .build();

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
}

#[cfg(target_env = "msvc")]
fn link_libarchive(_mode: &str) {
    if cfg!(feature = "vendored") {
        println!("cargo:rustc-link-lib=static=archive_static");
    } else {
        vcpkg::Config::new()
            .find_package("libarchive")
            .expect("Unable to find libarchive");
    }
}

#[cfg(not(target_env = "msvc"))]
fn link_libarchive(mode: &str) {
    println!("cargo:rustc-link-lib={}=archive", mode);
}

#[cfg(target_env = "msvc")]
fn link_deps(_mode: &str) {
    println!("cargo:rustc-link-lib=dylib=gdi32");
    println!("cargo:rustc-link-lib=dylib=user32");
    println!("cargo:rustc-link-lib=dylib=crypt32");

    // vcpkg::Config::new().find_package("openssl").expect("Unable to find openssl");
    vcpkg::Config::new()
        .find_package("bzip2")
        .expect("Unable to find bzip2");
    vcpkg::Config::new()
        .find_package("libxml2")
        .expect("Unable to find libxml2");
    vcpkg::Config::new()
        .find_package("lz4")
        .expect("Unable to find lz4");
    // vcpkg::Config::new().find_package("lzma").expect("Unable to find lzma");
    vcpkg::Config::new()
        .find_package("zstd")
        .expect("Unable to find zstd");
}

#[cfg(not(target_env = "msvc"))]
fn link_deps(mode: &str) {
    let pc_path = pkg_config::get_variable("pkg-config", "pc_path").expect("failed to get pc_path");

    for path in pc_path.split(":") {
        println!(
            "cargo:rustc-link-search=native={}",
            path.replace("/pkgconfig", "")
        );
    }

    if let Ok(pkg_config_path) = std::env::var("PKG_CONFIG_PATH") {
        for path in pkg_config_path.split(":") {
            println!(
                "cargo:rustc-link-search=native={}",
                path.replace("/pkgconfig", "")
            );
        }
    }

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");

    let icu_uc = pkg_config::Config::new()
        .statik(mode == "static")
        .probe("icu-uc")
        .expect("Unable to find icu-uc");

    for link_path in icu_uc.link_paths {
        println!("cargo:rustc-link-search=native={}", link_path.display());
    }

    for lib in icu_uc.libs {
        if lib == "pthread" || lib == "m" {
            continue;
        }
        println!("cargo:rustc-link-lib={}={}", mode, lib);
    }

    let expat = pkg_config::Config::new()
        .statik(mode == "static")
        .probe("expat")
        .expect("Unable to find expat");

    for link_path in expat.link_paths {
        println!("cargo:rustc-link-search=native={}", link_path.display());
    }

    for lib in expat.libs {
        println!("cargo:rustc-link-lib={}={}", mode, lib);
    }

    let libarchive = pkg_config::Config::new()
        .statik(mode == "static")
        .probe("libarchive")
        .expect("Unable to find libarchive");

    for link_path in libarchive.link_paths {
        println!("cargo:rustc-link-search=native={}", link_path.display());
    }

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search=native=/usr/local/opt/bzip2/lib");

    for lib in libarchive.libs {
        println!("cargo:rustc-link-lib={}={}", mode, lib);
    }
}
