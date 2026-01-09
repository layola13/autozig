fn main() -> anyhow::Result<()> {
    // 强制使用 MODULAR_BUILDZIG 模式避免文件重复
    std::env::set_var("AUTOZIG_MODE", "modular_buildzig");
    autozig_build::build("src")?;
    Ok(())
}
