
use std::env;
use std::path::PathBuf;

fn main() {
    let oodle_dir = PathBuf::from("oodle/src");
    let out_dir = PathBuf::from(
        env::var("OUT_DIR").expect("OUT_DIR environment variable not set"),
    );

    let target_os = env::var("CARGO_CFG_TARGET_OS")
        .expect("CARGO_CFG_TARGET_OS environment variable not set");

    let target_env = env::var("CARGO_CFG_TARGET_ENV")
        .unwrap_or_default();

    let include_dirs = [
        oodle_dir.join("base"),
        oodle_dir.join("core/public"),
        oodle_dir.join("core/templates"),
        oodle_dir.join("core"),
    ];

    let mut cpp_files: Vec<PathBuf> = include_dirs
        .iter()
        .flat_map(|dir| {
            std::fs::read_dir(dir)
                .expect("Failed to read directory")
        })
        .map(|entry| {
            entry
                .expect("Failed to read directory entry")
                .path()
        })
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "cpp")
        })
        .collect();

    // Apply AVX2 compiler settings to the required source files.
    for path in &mut cpp_files {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_owned();

        if matches!(
            file_name.as_str(),
            "newlz_offsets_avx2.cpp" | "ctmf_avx2.cpp"
        ) {
            let source = std::fs::read_to_string(&*path)
                .expect("Failed to read AVX2 source file");

            let patched_source = format!(
                "#if defined(__GNUC__) || defined(__clang__)\n\
                #pragma GCC target(\"avx2\")\n\
                #endif\n\n\
                {}",
                source
            );

            let patched_path = out_dir.join(&file_name);

            std::fs::write(&patched_path, patched_source)
                .expect("Failed to write patched AVX2 source file");

            *path = patched_path;
        }
    }

    println!("cargo:rerun-if-changed=oodle/src");

    let mut build = cc::Build::new();

    build
        .cpp(true)
        .std("c++20")
        .opt_level(3)
        .includes(&include_dirs)
        .files(&cpp_files)
        .define("OODLE_BUILDING_LIB", None);

    // Suppress compiler warnings when supported.
    build
        .flag_if_supported("-Wno-class-memaccess")
        .flag_if_supported("-Wno-comment")
        .flag_if_supported("-Wno-deprecated-enum-float-conversion")
        .flag_if_supported("-Wno-empty-body")
        .flag_if_supported("-Wno-extra")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-missing-field-initializers")
        .flag_if_supported("-Wno-type-limits")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-unused-local-typedefs")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-value")
        .flag_if_supported("-Wno-unused-variable");

    build.compile("oodle2");

    let bindgen = bindgen::Builder::default()
        .clang_args(
            include_dirs
                .iter()
                .map(|dir| format!("-I{}", dir.display())),
        )
        .header(oodle_dir.join("core/newlz.cpp").to_string_lossy())
        .header(oodle_dir.join("core/oodlelzpub.h").to_string_lossy())
        .header(
            oodle_dir
                .join("core/oodlelzcompressors.cpp")
                .to_string_lossy(),
        )
        .enable_cxx_namespaces()
        .rustified_enum(".*")
        .allowlist_recursively(true)
        .allowlist_function("oo2::OodleLZ_Compress")
        .allowlist_function("oo2::Mermaid_DecodeOneQuantum")
        .allowlist_function("oo2::Leviathan_DecodeOneQuantum")
        .allowlist_function("oo2::newLZ_decode_chunk_phase1")
        .allowlist_function("oo2::newLZ_decode_chunk_phase2")
        .clang_arg("-DOODLE_BUILDING_LIB")
        .clang_args(["-x", "c++"])
        .clang_arg("-std=c++20");

    bindgen
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/bindings.rs")
        .expect("Unable to write bindings");

    // Link the platform C++ standard library.
    match (target_os.as_str(), target_env.as_str()) {
        ("macos", _) => {
            println!("cargo:rustc-link-lib=dylib=c++");
        }
        (_, "gnu") => {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        ("windows", "msvc") => {
            // MSVC links the C++ runtime automatically.
        }
        _ => {
            println!(
                "cargo:warning=The C++ standard library linkage for target OS '{}' and environment '{}' has not been explicitly configured.",
                target_os,
                target_env
            );
        }
    }
}
