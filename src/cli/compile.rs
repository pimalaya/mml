//! `mml compile`: MML to MIME pipeline. Reads MML from `--input` or
//! stdin and prints the resulting MIME message.
//!
//! NOTE: parse errors are rendered as Ariadne diagnostics on stderr
//! and the process still exits zero, so the caller can pipe stdout
//! unconditionally.

use anyhow::{Context, Result};
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{cli::args::InputArg, compiler::message::MmlCompiler, error::MmlError};

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
        match MmlCompiler::default().compile(&mml) {
            Err(MmlError::ParseMmlError(errs, body)) => {
                for err in errs {
                    Report::build(ReportKind::Error, err.span().into_range())
                        .with_message("cannot parse MML message")
                        .with_label(
                            Label::new(err.span().into_range())
                                .with_message(err.reason().to_string())
                                .with_color(Color::Red),
                        )
                        .finish()
                        .print(Source::from(&body))
                        .unwrap();
                }
                Ok(())
            }
            Err(err) => Err(err).context("cannot compile MML message"),
            Ok(mime) => printer.out(String::from_utf8(mime)?),
        }
    }
}
