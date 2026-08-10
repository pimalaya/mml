//! Command output sink honouring both `--json` and `--output`.
//!
//! The two are independent: `--json` picks the payload format,
//! `--output` picks where it lands. Pretty JSON is reserved for a
//! terminal sink, like [`pimalaya_cli::printer::StdoutPrinter`].

use std::{
    fmt,
    fs::File,
    io::{IsTerminal, Write, stdout},
};

use anyhow::{Context, Result};
use pimalaya_cli::{clap::args::JsonFlag, printer::Printer};
use serde::Serialize;

use crate::cli::args::OutputArg;

/// A [`Printer`] writing to stdout or to the `--output` file.
pub struct MmlPrinter {
    sink: Box<dyn Write>,
    json: bool,
    terminal: bool,
}

impl MmlPrinter {
    /// Opens the `--output` sink, falling back to stdout.
    ///
    /// NOTE: truncated eagerly, so a bad path fails before any work.
    pub fn new(json: &JsonFlag, output: &OutputArg) -> Result<Self> {
        let (sink, terminal): (Box<dyn Write>, bool) = match output.path() {
            Some(path) => {
                let file = File::create(path)
                    .with_context(|| format!("Cannot open output file {}", path.display()))?;
                (Box::new(file), false)
            }
            None => (Box::new(stdout()), stdout().is_terminal()),
        };

        Ok(Self {
            sink,
            json: json.enabled,
            terminal,
        })
    }

    /// Builds a stdout printer, whatever `--output` says.
    ///
    /// NOTE: used for the error report, so a failure never lands in a
    /// half-written artifact.
    pub fn stdout(json: &JsonFlag) -> Self {
        Self {
            sink: Box::new(stdout()),
            json: json.enabled,
            terminal: stdout().is_terminal(),
        }
    }
}

impl Printer for MmlPrinter {
    fn out<T: fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
        if self.json {
            if self.terminal {
                serde_json::to_writer_pretty(&mut self.sink, &data)
                    .context("Cannot write pretty JSON output")?;
                writeln!(self.sink)?;
            } else {
                serde_json::to_writer(&mut self.sink, &data).context("Cannot write JSON output")?;
            }
        } else {
            write!(self.sink, "{data}")?;
        }

        self.sink.flush()?;

        Ok(())
    }

    fn is_json(&self) -> bool {
        self.json
    }
}
