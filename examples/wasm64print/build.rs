fn main() -> anyhow::Result<()> {
    // Check for WASM target
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("wasm") {
        println!(
            "cargo:warning=Skipping compilation of autozig-wasm64print for non-WASM target: {}",
            target
        );
        return Ok(());
    }

    // 强制使用 MODULAR_BUILDZIG 模式避免文件重复
    std::env::set_var("AUTOZIG_MODE", "modular_buildzig");

    // 🎯 编译 Zig 代码并生成 TypeScript 绑定
    autozig_build::build("src")?;

    Ok(())
}
