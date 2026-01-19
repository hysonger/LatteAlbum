use std::env;
//use std::path::PathBuf;

fn main() {
    // Check if we should build from project's vendor directory
    let project_build = env::var("CARGO_FEATURE_VENDOR_BUILD").is_ok();

    if project_build {
        // Build from vendor directory
        build_from_vendor();
    } else {
        // Link against system libraries
        link_system_libraries();
    }

    // Link against C++ standard library in Linux, or there would be compile error
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");
}

/// Link against system libraries using pkg-config
fn link_system_libraries() {
    use pkg_config::Config;

    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=LIBHEIF_NO_PKG_CONFIG");

    let mut pkg_config = Config::new();
    pkg_config.cargo_metadata(true);

    // Find libheif
    match pkg_config.probe("libheif") {
        Ok(heif_info) => {
            println!("Found libheif: {}", heif_info.version);
            // libheif is found, no need to do anything else as pkg-config will handle it
        }
        Err(e) => {
            eprintln!("Failed to find libheif: {}", e);
            eprintln!("Try installing libheif-dev or libheif-devel package");
            eprintln!("Or use --features vendor-build to build from vendor directory");
            std::process::exit(1);
        }
    }
}

/// Build libheif from vendor directory using cmake
fn build_from_vendor() {
    #[cfg(feature = "vendor-build")]
    {
        use cmake::Config;
        use std::fs;

        println!("cargo:rerun-if-changed=vendor/libheif");

        let out_dir = env::var("OUT_DIR").unwrap();
        let vendor_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("vendor");
        let install_dir = PathBuf::from(&out_dir).join("install");

        // Create install directory
        fs::create_dir_all(&install_dir).unwrap();

        // Build libheif
        println!("Building libheif from vendor directory...");
        let libheif_dir = vendor_dir.join("libheif");
        let libheif_build_dir = PathBuf::from(&out_dir).join("libheif-build");

        let mut libheif_config = Config::new(libheif_dir);
        libheif_config
            .out_dir(&libheif_build_dir)
            .define("CMAKE_INSTALL_PREFIX", &install_dir)
            .define("CMAKE_BUILD_TYPE", "Release")
            .define("BUILD_SHARED_LIBS", "ON") // 动态库
            .define("WITH_EXAMPLES", "OFF")
            .define("WITH_GDK_PIXBUF", "OFF")
            .define("WITH_GNOME", "OFF")
            .define("BUILD_TESTING", "OFF")
            .define("BUILD_DOCUMENTATION", "OFF")
            .define("WITH_FUZZERS", "OFF")
            .define("WITH_WEBCODECS", "OFF")
            .define("WITH_UNCOMPRESSED_CODEC", "OFF");

        let _libheif_install = libheif_config.build();
        println!("libheif built successfully");

        // Add the install directory to pkg-config path for downstream crates
        println!("cargo:pkgconfigpath={}", install_dir.join("lib").join("pkgconfig").display());

        // Add library directories
        println!("cargo:rustc-link-search=native={}", install_dir.join("lib").display());

        // Link against the built dynamic library
        // 前面构建环境变量用的是动态库，这里就要对应上
        println!("cargo:rustc-link-lib=dylib=heif");

        // Add include directories for any crates that need them
        println!("cargo:include={}", install_dir.join("include").display());
    }
}
