//! `mml compile`: MML to MIME pipeline. Reads MML from `--input` or
//! stdin and prints the resulting MIME message.
//!
//! A source that does not parse fails coded `mml-invalid`, each parse
//! error drawn on stderr against the source unless `--json` is set.

use anyhow::{Context, Result};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{args::InputArg, crypto, error::mml_invalid, printer::rfc822},
    compiler::message::MmlCompileOptions,
    error::MmlError,
};

/// Compile an MML message into a MIME message.
#[derive(Debug, Parser)]
pub struct CompileCommand {
    #[command(flatten)]
    pub input: InputArg,
}

impl CompileCommand {
    /// Compiles the source and prints the MIME message.
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        let mml = self.input.read()?;

        match crypto::compile(&mml, MmlCompileOptions::default()) {
            Ok(mime) => printer.out_bytes(rfc822(mime)),
            Err(MmlError::ParseMmlError(errs, body)) => {
                Err(mml_invalid(&errs, &body, !printer.is_json()))
            }
            Err(err) => Err(err).context("cannot compile MML message"),
        }
    }
}
