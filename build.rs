fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    let has_fma = features.split(',').any(|f| f.trim() == "fma");

    if matches!(arch.as_str(), "x86" | "x86_64") && !has_fma {
        println!(
            "cargo:warning=metallic: compiling for {arch} without FMA. \
             Performance and accuracy are reduced. Add to .cargo/config.toml:\n\
             [build]\n\
             rustflags = [\"-Ctarget-cpu=native\"]"
        );
    }
}
