//! `$EDITOR` flow shared by `compose` / `reply` / `forward`.
//!
//! The editor is whatever the `edit` crate resolves: `$VISUAL`, then
//! `$EDITOR`, then an OS default. mml does not expose a config
//! override on top; set `VISUAL` / `EDITOR` in your shell.
//!
//! The flow is: open the editor on the template, try to compile MML
//! into MIME, surface compile errors via Ariadne on stderr, then
//! prompt the user with [`crate::cli::utils::choice::post_edit`]
//! (save / edit again / view MML / view MIME / abort). The save
//! choice label tracks the caller's output destination (file vs
//! stdout). The loop only exits on a save choice (returns
//! `Some(mime)`) or `Abort` (returns `None`).

use std::io::{Write, stderr};

use anyhow::{Context, Result};
use ariadne::{Color, Label, Report, ReportKind, Source};
use edit::Builder as EditBuilder;

use crate::{
    cli::utils::choice::{PostEditChoice, post_edit},
    compiler::message::MmlCompilerBuilder,
    error::MmlError,
    template::types::Template,
};

/// Compose loop: edit → compile → prompt → loop.
///
/// `has_output_path` toggles the save-choice label between
/// "Save to file" and "Save to stdout"; the actual write happens at
/// the caller. Returns `Ok(Some(mime))` when the user saves,
/// `Ok(None)` on abort.
pub fn edit_loop(initial: Template, has_output_path: bool) -> Result<Option<Vec<u8>>> {
    let mut buffer = initial.content;

    loop {
        buffer = edit::edit_with_builder(&buffer, EditBuilder::new().suffix(".eml"))
            .context("cannot spawn editor")?;

        let compile_result = compile_buffer(&buffer);
        let compiled = match &compile_result {
            Ok(mime) => Some(mime.clone()),
            Err(err) => {
                let _ = writeln!(stderr(), "{err:#}");
                None
            }
        };

        loop {
            let choice = post_edit(compiled.is_some(), has_output_path)?;
            match choice {
                PostEditChoice::SaveToFile | PostEditChoice::SaveToStdout => {
                    if let Some(mime) = compiled {
                        return Ok(Some(mime));
                    }
                }
                PostEditChoice::Edit => break,
                PostEditChoice::ViewTemplate => {
                    let _ = writeln!(stderr(), "{buffer}");
                }
                PostEditChoice::ViewMime => {
                    if let Some(mime) = &compiled {
                        let _ = stderr().write_all(mime);
                        let _ = writeln!(stderr());
                    }
                }
                PostEditChoice::Abort => return Ok(None),
            }
        }
    }
}

fn compile_buffer(buffer: &str) -> Result<Vec<u8>> {
    let compiler = MmlCompilerBuilder::new()
        .build(buffer)
        .context("cannot build MML compiler")?;

    match compiler.compile() {
        Ok(res) => Ok(res.into_vec()?),
        Err(MmlError::ParseMmlError(errs, body)) => {
            for err in &errs {
                Report::build(ReportKind::Error, err.span().into_range())
                    .with_message("cannot parse MML message")
                    .with_label(
                        Label::new(err.span().into_range())
                            .with_message(err.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(&body))
                    .unwrap();
            }
            anyhow::bail!("MML parse error");
        }
        Err(err) => Err(anyhow::Error::from(err)).context("cannot compile MML message"),
    }
}
