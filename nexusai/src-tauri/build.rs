fn main() {
    tauri_build::build();

    // TODO: Phase 1 — Build llama.cpp with TurboQuant support
    // This will use cc::Build to compile the llama.cpp fork with TQ types.
    // For now, we stub the inference layer in pure Rust.
    //
    // let mut build = cc::Build::new();
    // build
    //     .cpp(true)
    //     .file("turboquant-sys/src/llama_bridge.cpp")
    //     .include("turboquant-sys/include")
    //     .flag_if_supported("-std=c++17")
    //     .flag_if_supported("-O3");
    //
    // #[cfg(feature = "metal")]
    // {
    //     println!("cargo:rustc-link-lib=framework=Metal");
    //     println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
    //     build.file("turboquant-sys/kernels/metal/tq_dequant.metal");
    // }
    //
    // #[cfg(feature = "cuda")]
    // {
    //     println!("cargo:rustc-link-lib=cuda");
    //     println!("cargo:rustc-link-lib=cudart");
    // }
    //
    // build.compile("turboquant_sys");
}
