use anyhow::{Context, Result};
use mml::{
    message::{FilterParts, ShowHeadersStrategy},
    MimeInterpreter,
};

pub async fn interpret(show_headers: ShowHeadersStrategy, mime: String) -> Result<()> {
    let interpreter = MimeInterpreter::new()
        // TODO: with_pgp(…)
        .with_show_headers(show_headers)
        // TODO: .with_show_additional_headers(…);
        .with_filter_parts(FilterParts::All)
        .with_show_multiparts(true)
        .with_save_attachments(true)
        .with_save_attachments_dir("/tmp")
        .with_show_inline_attachments(true)
        .with_show_plain_texts_signature(true);

    let mml = interpreter
        .interpret_bytes(mime.as_bytes())
        .await
        .context("cannot interpreter MIME message")?;

    print!("{mml}");

    Ok(())
}
