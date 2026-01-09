fn main() -> anyhow::Result<()> {
    // Check for WASM target
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("wasm") {
        println!(
            "cargo:warning=Skipping compilation of autozig-wasm-filter for non-WASM target: {}",
            target
        );
        return Ok(());
    }

    autozig_build::build("src")?;
    Ok(())
}
