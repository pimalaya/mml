#![cfg_attr(
    not(all(feature = "compiler", feature = "interpreter")),
    allow(unused_imports)
)]

use anyhow::{Context, Result};
#[cfg(feature = "interpreter")]
use mml::MimeInterpreter;
#[cfg(feature = "compiler")]
use mml::MmlCompiler;

#[cfg(feature = "compiler")]
pub async fn compile(mml: String) -> Result<()> {
    let mime = MmlCompiler::new()
        .compile(mml)
        .await
        .context("cannot compile mml message")?
        .write_to_string()?;
    print!("{mime}");
    Ok(())
}

#[cfg(feature = "interpreter")]
pub async fn interpret(mime: String) -> Result<()> {
    let mml = MimeInterpreter::new()
        .interpret_bytes(mime.as_bytes())
        .await
        .context("cannot interpreter mime message")?;
    print!("{mml}");
    Ok(())
}
