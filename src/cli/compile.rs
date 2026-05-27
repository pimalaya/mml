//! `mml compile`: MML to MIME pipeline. Reads MML from the
//! positional argument or stdin and writes the resulting MIME
//! message to stdout. Parse errors are rendered as Ariadne
//! diagnostics on stderr and the process still exits successfully so
//! the caller can pipe stdout unconditionally.

use anyhow::{Context, Result};
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{cli::args::MessageArg, compiler::message::MmlCompilerBuilder, error::MmlError};

/// Compile an MML message into a MIME message.
#[derive(Debug, Parser)]
pub struct CompileCommand {
    /// Path to an MML file, a raw MML string, or nothing when piped
    /// on stdin.
    #[command(flatten)]
    pub mml: MessageArg,
}

impl CompileCommand {
    pub fn execute(self, _printer: &mut impl Printer) -> Result<()> {
        let mml = self.mml.parse()?;
        let compiler = MmlCompilerBuilder::new()
            .build(&mml)
            .context("cannot build MML compiler")?;

        match compiler.compile() {
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
            Ok(res) => {
                let mime = res.into_string()?;
                print!("{mime}");
                Ok(())
            }
        }
    }
}
