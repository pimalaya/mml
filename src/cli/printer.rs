//! Command output sink honouring both `--json` and `--output`.
//!
//! The two are independent: `--json` picks the payload format,
//! `--output` picks where it lands. Pretty JSON is reserved for a
//! terminal sink, like [`pimalaya_cli::printer::StdoutPrinter`], and
//! piped JSON is one value per line.
//!
//! Text written to a terminal is defused (see [`defuse`]): a message
//! rendered there could otherwise drive it with escape sequences. A
//! file or a pipe gets the artifact byte for byte.

use std::{
    borrow::Cow,
    fmt,
    fs::File,
    io::{IsTerminal, Write, stdout},
};

use anyhow::{Context, Result};
use pimalaya_cli::{clap::args::JsonFlag, output::Bytes, printer::Printer, sanitize};
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
    /// The file is truncated eagerly, so a bad path fails before any work.
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

    /// Builds a stdout printer, whatever `--output` says, for the error
    /// report, so a failure never lands in a half-written artifact.
    pub fn stdout(json: &JsonFlag) -> Self {
        Self {
            sink: Box::new(stdout()),
            json: json.enabled,
            terminal: stdout().is_terminal(),
        }
    }

    fn write_json<T: Serialize>(&mut self, data: &T) -> Result<()> {
        if self.terminal {
            serde_json::to_writer_pretty(&mut self.sink, data)
                .context("Cannot write pretty JSON output")?;
        } else {
            serde_json::to_writer(&mut self.sink, data).context("Cannot write JSON output")?;
        }

        writeln!(self.sink)?;
        self.sink.flush()?;
        Ok(())
    }

    fn write_text(&mut self, text: &str) -> Result<()> {
        if self.terminal {
            write!(self.sink, "{}", defuse(text))?;
        } else {
            write!(self.sink, "{text}")?;
        }

        self.sink.flush()?;
        Ok(())
    }
}

impl Printer for MmlPrinter {
    fn out<T: fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
        if self.json {
            self.write_json(&data)
        } else {
            self.write_text(&data.to_string())
        }
    }

    fn event<T: fmt::Display + Serialize>(&mut self, event: T) -> Result<()> {
        if self.json {
            return self.write_json(&event);
        }

        if self.terminal {
            self.write_text(&event.to_string())?;
        }

        Ok(())
    }

    fn out_bytes(&mut self, bytes: Bytes) -> Result<()> {
        if self.json {
            return self.write_json(&bytes);
        }

        if self.terminal {
            return self.write_text(&String::from_utf8_lossy(&bytes.data));
        }

        self.sink
            .write_all(&bytes.data)
            .context("Cannot write output")?;
        self.sink.flush()?;
        Ok(())
    }

    fn is_json(&self) -> bool {
        self.json
    }
}

/// A compiled MIME message as the bytes a command prints.
pub fn rfc822(data: Vec<u8>) -> Bytes {
    let mime = String::from("message/rfc822");
    Bytes { data, mime }
}

/// Defuses text bound for a terminal: [`sanitize::text`] on every
/// stretch between line breaks and tabs, which are kept, a CRLF
/// included.
pub fn defuse(text: &str) -> Cow<'_, str> {
    let mut defused = String::with_capacity(text.len());

    for line in text.split_inclusive('\n') {
        let (line, end) = match line.strip_suffix("\r\n") {
            Some(line) => (line, "\r\n"),
            None => match line.strip_suffix('\n') {
                Some(line) => (line, "\n"),
                None => (line, ""),
            },
        };

        for (i, cell) in line.split('\t').enumerate() {
            if i > 0 {
                defused.push('\t');
            }

            defused.push_str(&sanitize::text(cell));
        }

        defused.push_str(end);
    }

    if defused == text {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(defused)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::cli::printer::defuse;

    #[test]
    fn clean_text_is_borrowed() {
        let text = "Subject: Hi\r\n\r\n\tcafé ☕\n";
        assert!(matches!(defuse(text), Cow::Borrowed(t) if t == text));
    }

    #[test]
    fn escapes_and_bidi_are_defused_breaks_and_tabs_kept() {
        let text = "a\x1b]52;c;x\x07b\r\n\tinvoice\u{202e}fdp.exe\rz\n";
        let expected = "a\u{fffd}]52;c;x\u{fffd}b\r\n\tinvoice\u{fffd}fdp.exe\u{fffd}z\n";
        assert_eq!(defuse(text), expected);
    }
}
