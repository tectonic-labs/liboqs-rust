use std::path::{Path, PathBuf};
use std::process::Command;

/// Tries to parse the .gitmodules file to extract the URL and branch for the liboqs submodule.
/// Returns Some((url, branch)) if successful, None otherwise.
fn try_parse_gitmodules() -> Option<(String, String)> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let gitmodules_path = manifest_dir.parent()?.join(".gitmodules");

    let contents = std::fs::read_to_string(&gitmodules_path).ok()?;

    let mut url = None;
    let mut branch = None;
    let mut in_liboqs_section = false;

    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with("[submodule") && line.contains("liboqs") {
            in_liboqs_section = true;
            continue;
        }
        if line.starts_with('[') {
            in_liboqs_section = false;
            continue;
        }
        if in_liboqs_section {
            if let Some(value) = line.strip_prefix("url = ").or_else(|| line.strip_prefix("url=")) {
                url = Some(value.trim().to_string());
            } else if let Some(value) =
                line.strip_prefix("branch = ").or_else(|| line.strip_prefix("branch="))
            {
                branch = Some(value.trim().to_string());
            }
        }
    }

    let url = url?;
    let branch = branch.unwrap_or_else(|| "main".to_string());

    Some((url, branch))
}

/// Parses [package.metadata.liboqs] from Cargo.toml to get URL and branch.
/// This is used when .gitmodules is not available (e.g., crates.io builds).
fn parse_cargo_metadata() -> (String, String) {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let cargo_toml_path = manifest_dir.join("Cargo.toml");

    let contents = std::fs::read_to_string(&cargo_toml_path)
        .expect("Failed to read Cargo.toml");

    let mut url = None;
    let mut branch = None;
    let mut in_metadata_section = false;

    for line in contents.lines() {
        let line = line.trim();
        if line == "[package.metadata.liboqs]" {
            in_metadata_section = true;
            continue;
        }
        if line.starts_with('[') {
            in_metadata_section = false;
            continue;
        }
        if in_metadata_section {
            if let Some(value) = line.strip_prefix("url = ").or_else(|| line.strip_prefix("url=")) {
                url = Some(value.trim().trim_matches('"').to_string());
            } else if let Some(value) =
                line.strip_prefix("branch = ").or_else(|| line.strip_prefix("branch="))
            {
                branch = Some(value.trim().trim_matches('"').to_string());
            }
        }
    }

    let url = url.expect("Could not find 'url' in [package.metadata.liboqs] in Cargo.toml");
    let branch = branch.expect("Could not find 'branch' in [package.metadata.liboqs] in Cargo.toml");

    (url, branch)
}

/// Gets the liboqs repository URL and branch.
/// First tries .gitmodules (local development), then falls back to Cargo.toml metadata (crates.io).
fn get_liboqs_repo_info() -> (String, String) {
    try_parse_gitmodules().unwrap_or_else(parse_cargo_metadata)
}

/// Returns the path to liboqs source, downloading it to OUT_DIR if necessary.
/// When building from crates.io, the submodule isn't included so we download it.
/// For local development with the submodule, we use the local copy.
fn get_liboqs_source_dir() -> PathBuf {
    let local_liboqs = Path::new("liboqs");

    // If local submodule exists and has content, use it
    if local_liboqs.join("CMakeLists.txt").exists() {
        return local_liboqs.to_path_buf();
    }

    // Otherwise, download to OUT_DIR
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let liboqs_dir = out_dir.join("liboqs-src");

    // Check if already downloaded
    if liboqs_dir.join("CMakeLists.txt").exists() {
        return liboqs_dir;
    }

    println!("cargo:warning=liboqs source not found, downloading from git...");

    // Remove the directory if it exists but is empty/incomplete
    if liboqs_dir.exists() {
        std::fs::remove_dir_all(&liboqs_dir).ok();
    }

    // Get URL and branch from .gitmodules or Cargo.toml metadata
    let (repo_url, branch) = get_liboqs_repo_info();

    // Clone the repository
    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            &branch,
            &repo_url,
            liboqs_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute git clone. Make sure git is installed.");

    if !status.success() {
        panic!("Failed to clone liboqs repository");
    }

    println!("cargo:warning=liboqs source downloaded successfully");
    liboqs_dir
}

fn generate_bindings(includedir: &Path, headerfile: &str, allow_filter: &str, block_filter: &str) {
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let mut builder = bindgen::Builder::default()
        .clang_arg(format!("-I{}", includedir.display()))
        .header(
            includedir
                .join("oqs")
                .join(format!("{headerfile}.h"))
                .to_str()
                .unwrap(),
        );

    // Add Emscripten system headers for WASM targets
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.starts_with("wasm32") {
        if let Ok(emsdk) = std::env::var("EMSDK") {
            // Add Emscripten system include paths
            let emsdk_include = format!("{}/upstream/emscripten/cache/sysroot/include", emsdk);
            builder = builder.clang_arg(format!("-I{}", emsdk_include));

            // Set the target for clang
            builder = builder.clang_arg("--target=wasm32-unknown-emscripten");
        }
    }

    builder
        // Options
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .size_t_is_usize(true)
        // Don't generate docs unless enabled
        // Otherwise it breaks tests
        .generate_comments(cfg!(feature = "docs"))
        // Allowlist/blocklist OQS stuff
        .allowlist_recursively(false)
        .allowlist_type(allow_filter)
        .allowlist_function(allow_filter)
        .allowlist_var(allow_filter)
        .blocklist_type(block_filter)
        .blocklist_function(block_filter)
        .blocklist_var(block_filter)
        // Use core and libc
        .use_core()
        .ctypes_prefix("::libc")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join(format!("{headerfile}_bindings.rs")))
        .expect("Couldn't write bindings!");
}

fn build_from_source(liboqs_src: &Path) -> PathBuf {
    let mut config = cmake::Config::new(liboqs_src);
    config.profile("Release");
    config.define("OQS_BUILD_ONLY_LIB", "Yes");

    // Detect WASM target and configure for Emscripten
    let target = std::env::var("TARGET").unwrap_or_default();
    let is_wasm = target.starts_with("wasm32");

    if is_wasm {
        // Use Ninja generator as recommended for Emscripten
        // (emcmake cmake -GNinja ...)
        config.generator("Ninja");

        // Set Emscripten toolchain file if EMSDK is available
        if let Ok(emsdk) = std::env::var("EMSDK") {
            let toolchain = format!(
                "{}/upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake",
                emsdk
            );
            config.define("CMAKE_TOOLCHAIN_FILE", &toolchain);
        }

        // Force OpenSSL OFF for WASM (as per liboqs issue #1199)
        config.define("OQS_USE_OPENSSL", "OFF");

        // Permit unsupported architecture for WASM
        config.define("OQS_PERMIT_UNSUPPORTED_ARCHITECTURE", "ON");

        println!("cargo:warning=Building for WASM with Emscripten");
    }

    if cfg!(feature = "non_portable") {
        // Build with CPU feature detection or just enable whatever is available for this CPU
        config.define("OQS_DIST_BUILD", "No");
    } else {
        config.define("OQS_DIST_BUILD", "Yes");
    }

    macro_rules! algorithm_feature {
        ($typ:literal, $feat: literal) => {
            let configflag = format!("OQS_ENABLE_{}_{}", $typ, $feat.to_ascii_uppercase());
            let value = if cfg!(feature = $feat) { "Yes" } else { "No" };
            config.define(&configflag, value);
        };
    }

    // KEMs
    // BIKE is not supported on Windows or Arm32, so if either is in the mix,
    // have it be opt-in explicitly except through the default kems feature.
    if cfg!(feature = "kems") && !(cfg!(windows) || cfg!(target_arch = "arm")) {
        println!("cargo:rustc-cfg=feature=\"bike\"");
        config.define("OQS_ENABLE_KEM_BIKE", "Yes");
    } else {
        algorithm_feature!("KEM", "bike");
    }
    algorithm_feature!("KEM", "classic_mceliece");
    algorithm_feature!("KEM", "frodokem");
    algorithm_feature!("KEM", "hqc");
    algorithm_feature!("KEM", "kyber");
    algorithm_feature!("KEM", "ml_kem");
    algorithm_feature!("KEM", "ntruprime");

    // signature schemes
    algorithm_feature!("SIG", "cross");
    algorithm_feature!("SIG", "dilithium");
    algorithm_feature!("SIG", "falcon");
    algorithm_feature!("SIG", "mayo");
    algorithm_feature!("SIG", "ml_dsa");
    algorithm_feature!("SIG", "sphincs");
    algorithm_feature!("SIG", "uov");

    if cfg!(windows) {
        // Select the latest available Windows SDK
        // SDK version 10.0.17763.0 seems broken
        config.define("CMAKE_SYSTEM_VERSION", "10.0");
    }

    // link the openssl libcrypto
    // Skip OpenSSL for WASM builds as it's not compatible
    if !is_wasm && cfg!(any(feature = "openssl", feature = "vendored_openssl")) {
        config.define("OQS_USE_OPENSSL", "Yes");
        if cfg!(windows) {
            // Windows doesn't prefix with lib
            println!("cargo:rustc-link-lib=libcrypto");
        } else {
            println!("cargo:rustc-link-lib=crypto");
        }
    } else {
        config.define("OQS_USE_OPENSSL", "No");
    }

    // let the linker know where to search for openssl libcrypto
    // Skip OpenSSL configuration for WASM builds
    if !is_wasm && cfg!(feature = "vendored_openssl") {
        // DEP_OPENSSL_ROOT is set by openssl-sys if a vendored build was used.
        // We point CMake towards this so that the vendored openssl is preferred
        // over the system openssl.
        let vendored_openssl_root = std::env::var("DEP_OPENSSL_ROOT")
            .expect("The `vendored_openssl` feature was enabled, but DEP_OPENSSL_ROOT was not set");
        config.define("OPENSSL_ROOT_DIR", vendored_openssl_root);
    } else if !is_wasm && cfg!(feature = "openssl") {
        println!("cargo:rerun-if-env-changed=OPENSSL_ROOT_DIR");
        if let Ok(dir) = std::env::var("OPENSSL_ROOT_DIR") {
            let dir = Path::new(&dir).join("lib");
            println!("cargo:rustc-link-search={}", dir.display());
        } else if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
            println!("cargo:warning=You may need to specify OPENSSL_ROOT_DIR or disable the default `openssl` feature.");
        }
    }

    let permit_unsupported = "OQS_PERMIT_UNSUPPORTED_ARCHITECTURE";
    if let Ok(str) = std::env::var(permit_unsupported) {
        config.define(permit_unsupported, str);
    }

    // build the default (install) target.
    let outdir = config.build();

    // remove the build folder
    let temp_build = outdir.join("build");
    if let Err(e) = std::fs::remove_dir_all(temp_build) {
        println!(
            "cargo:warning=unexpected error while cleaning build files:{}",
            e
        );
    }

    // lib is installed to $outdir/lib or lib64, depending on CMake conventions
    let libdir = outdir.join("lib");
    let libdir64 = outdir.join("lib64");

    if cfg!(windows) {
        // Static linking doesn't work on Windows
        println!("cargo:rustc-link-lib=oqs");
    } else {
        // Statically linking makes it easier to use the sys crate
        println!("cargo:rustc-link-lib=static=oqs");
    }

    if cfg!(windows) {
        // Explicitly link against advapi32. See https://github.com/rust-lang/rust/issues/140555
        println!("cargo:rustc-link-lib=advapi32");
    }

    println!("cargo:rustc-link-search=native={}", libdir.display());
    println!("cargo:rustc-link-search=native={}", libdir64.display());

    outdir
}

fn includedir_from_source() -> PathBuf {
    let liboqs_src = get_liboqs_source_dir();
    let outdir = build_from_source(&liboqs_src);
    outdir.join("include")
}

fn probe_includedir() -> PathBuf {
    if cfg!(feature = "vendored") {
        return includedir_from_source();
    }

    println!("cargo:rerun-if-env-changed=LIBOQS_NO_VENDOR");
    let force_no_vendor = std::env::var_os("LIBOQS_NO_VENDOR").is_some_and(|v| v != "0");

    let version = env!("CARGO_PKG_VERSION");
    let (_, liboqs_version) = version.split_once("+liboqs-").unwrap();
    let &[major_version, minor_version, _] =
        liboqs_version.split('.').collect::<Vec<_>>().as_slice()
    else {
        panic!("Failed to parse target liboqs version");
    };
    let minor_num: usize = minor_version.parse().unwrap();
    let upper_bound = format!("{}.{}.0", major_version, minor_num + 1);
    let config = pkg_config::Config::new()
        .range_version(liboqs_version..upper_bound.as_str())
        .probe("liboqs");

    match config {
        Ok(lib) => lib.include_paths.first().cloned().unwrap(),
        _ => {
            if force_no_vendor {
                panic!("The env variable LIBOQS_NO_VENDOR has been set but a suitable system liboqs could not be found.");
            }

            includedir_from_source()
        }
    }
}

fn main() {
    // Check if clang is available before compiling anything.
    bindgen::clang_version();

    let includedir = probe_includedir();
    let gen_bindings = |file, allow_filter, block_filter| {
        generate_bindings(&includedir, file, allow_filter, block_filter)
    };

    gen_bindings("common", "OQS_.*", "");
    gen_bindings("rand", "OQS_(randombytes|RAND).*", "");
    gen_bindings("kem", "OQS_KEM.*", "");
    gen_bindings("sig", "OQS_SIG.*", "OQS_SIG_STFL.*");

    // Only watch for changes if local submodule exists (not when downloaded)
    if Path::new("liboqs/CMakeLists.txt").exists() {
        build_deps::rerun_if_changed_paths("liboqs/src/**/*").unwrap();
        build_deps::rerun_if_changed_paths("liboqs/src").unwrap();
        build_deps::rerun_if_changed_paths("liboqs/src/*").unwrap();
    }
}
