use anyhow::{Context, Result};
use mml::MmlCompiler;

pub async fn compile(mml: String) -> Result<()> {
    let mime = MmlCompiler::new()
        .compile(mml)
        .await
        .context("cannot compile mml message")?
        .write_to_string()?;
    print!("{mime}");
    Ok(())
}
