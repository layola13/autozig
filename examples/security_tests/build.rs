fn main() -> anyhow::Result<()> {
    // 扫描 src 目录中的 autozig! 宏
    autozig_build::build("src")?;
    Ok(())
}