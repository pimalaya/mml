use anyhow::{Context, Result};
use mml::MimeInterpreter;

pub async fn interpret(mime: String) -> Result<()> {
    let mml = MimeInterpreter::new()
        .interpret_bytes(mime.as_bytes())
        .await
        .context("cannot interpreter mime message")?;
    print!("{mml}");
    Ok(())
}
