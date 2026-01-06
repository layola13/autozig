fn main() -> anyhow::Result<()> {
    autozig_build::build("src")?;
    Ok(())
}