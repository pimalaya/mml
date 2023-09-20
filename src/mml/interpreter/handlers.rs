use anyhow::{Context, Result};
use mml::{
    message::{FilterHeaders, FilterParts},
    MimeInterpreter,
};
use std::path::PathBuf;

pub async fn interpret(
    filter_headers: FilterHeaders,
    filter_parts: FilterParts,
    show_multiparts: bool,
    save_attachments: bool,
    save_attachments_dir: Option<PathBuf>,
    show_attachments: bool,
    show_inline_attachments: bool,
    show_plain_texts_signature: bool,
    mime: String,
) -> Result<()> {
    let interpreter = MimeInterpreter::new()
        // TODO:
        // .with_pgp(…)
        .with_show_headers(filter_headers)
        // TODO:
        // .with_show_additional_headers(…);
        .with_filter_parts(filter_parts)
        .with_show_multiparts(show_multiparts)
        .with_save_attachments(save_attachments)
        .with_save_some_attachments_dir(save_attachments_dir)
        .with_show_attachments(show_attachments)
        .with_show_inline_attachments(show_inline_attachments)
        .with_show_plain_texts_signature(show_plain_texts_signature);

    let mml = interpreter
        .interpret_bytes(mime.as_bytes())
        .await
        .context("cannot interpreter MIME message")?;

    print!("{mml}");

    Ok(())
}
