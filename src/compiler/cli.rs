use std::fs;

use anyhow::{Context, Result};
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use log::debug;
use pimalaya_cli::{clap::parsers::path_parser, printer::Printer};

use crate::{
    cli::stdin::{format_stdin, format_str},
    compiler::message::MmlCompilerBuilder,
    error::MmlError,
};

type MmlMessage = String;

/// Compile an MML message into a MIME message.
///
/// Reads MML from the positional argument (file path or raw text)
/// or, when absent, from stdin. Writes the resulting MIME message to
/// stdout. Parse errors are rendered as Ariadne diagnostics on stderr
/// and the process exits successfully — same behavior as before.
#[derive(Debug, Parser)]
pub struct CompileCommand {
    /// Path to an MML file, or a raw MML string.
    #[arg(value_parser = parse_mml)]
    pub mml: Option<MmlMessage>,
}

impl CompileCommand {
    pub fn execute(self, _printer: &mut impl Printer) -> Result<()> {
        let mml = self.mml.unwrap_or_else(format_stdin);

        let compiler = MmlCompilerBuilder::new()
            .build(&mml)
            .context("cannot build MML compiler")?;

        match compiler.compile() {
            Err(MmlError::ParseMmlError(errs, body)) => {
                for err in errs {
                    Report::build(ReportKind::Error, (), err.span().start)
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

fn parse_mml(raw: &str) -> Result<MmlMessage, String> {
    match path_parser(raw) {
        Ok(path) => fs::read_to_string(path).map_err(|err| err.to_string()),
        Err(err) => {
            debug!("invalid path ({err}), processing arg as raw MML message");
            Ok(format_str(raw))
        }
    }
}
