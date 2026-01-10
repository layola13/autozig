fn main() -> anyhow::Result<()> {
    // Check for WASM target
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("wasm") {
        println!(
            "cargo:warning=Skipping compilation of autozig-rust-export for non-WASM target: {}",
            target
        );
        return Ok(());
    }

    // 强制使用 MODULAR_BUILDZIG 模式避免文件重复
    std::env::set_var("AUTOZIG_MODE", "modular_buildzig");
    
    // 🎯 一行搞定！对于 WASM 目标，build() 会自动：
    // 1. 编译 Zig 代码（如果有 autozig! 宏）
    // 2. 生成 TypeScript 绑定（对于 #[autozig_export] 函数）
    autozig_build::build("src")?;
    
    Ok(())
}