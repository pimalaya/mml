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
use edit::Builder as EditBuilder;

use crate::{
    cli::{
        crypto,
        error::mml_invalid,
        utils::choice::{PostEditChoice, post_edit},
    },
    compiler::{
        body::{MmlBodyCompiler, MmlBodyFormat},
        message::MmlCompileOptions,
    },
    error::MmlError,
    template::MmlTemplate,
};

/// Builds the compile options of a draft from its configuration
/// section, unset values falling back to the compiler defaults.
pub fn compile_options(
    format: Option<MmlBodyFormat>,
    flowed_column: Option<usize>,
) -> MmlCompileOptions {
    let defaults = MmlBodyCompiler::default();

    MmlCompileOptions {
        body: MmlBodyCompiler {
            format: format.unwrap_or(defaults.format),
            flowed_column: flowed_column.unwrap_or(defaults.flowed_column),
        },
    }
}

/// Compose loop: edit → compile → prompt → loop.
///
/// `has_output_path` toggles the save-choice label between
/// "Save to file" and "Save to stdout"; the actual write happens at
/// the caller. Returns `Ok(Some(mime))` when the user saves,
/// `Ok(None)` on abort.
pub fn edit_loop(
    initial: MmlTemplate,
    opts: MmlCompileOptions,
    has_output_path: bool,
) -> Result<Option<Vec<u8>>> {
    let mut buffer = initial.content;

    loop {
        buffer = edit::edit_with_builder(&buffer, EditBuilder::new().suffix(".eml"))
            .context("cannot spawn editor")?;

        let compile_result = compile_buffer(&buffer, &opts);
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

fn compile_buffer(buffer: &str, opts: &MmlCompileOptions) -> Result<Vec<u8>> {
    match crypto::compile(buffer, opts.clone()) {
        Ok(mime) => Ok(mime),
        Err(MmlError::ParseMmlError(errs, body)) => Err(mml_invalid(&errs, &body, true)),
        Err(err) => Err(anyhow::Error::from(err)).context("cannot compile MML message"),
    }
}
