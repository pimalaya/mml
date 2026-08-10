//! Reusable clap arg groups shared between the editor-driven
//! commands and their `template` counterparts.
//!
//! Commands read from stdin and write to stdout by default.
//! [`InputArg`] and [`OutputArg`] redirect either end to a file, `-`
//! naming the stream.

use std::{
    fs,
    io::{IsTerminal, Read, stdin, stdout},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use log::debug;
use pimalaya_cli::clap::parsers::path_parser;

/// Where a command reads its source, stdin by default.
#[derive(Debug, Parser)]
pub struct InputArg {
    /// Read the source from PATH instead of stdin.
    ///
    /// Pass `-` for stdin, `./-` for a file named `-`.
    #[arg(long, short = 'i', value_name = "PATH")]
    pub input: Option<PathBuf>,
}

impl InputArg {
    /// Reads the source from the given path, or from stdin.
    ///
    /// NOTE: an implicit stdin that is a terminal would just hang, so
    /// it is rejected. An explicit `-i -` still reads the terminal.
    pub fn read(&self) -> Result<String> {
        if let Some(path) = self.input.as_deref().filter(|path| *path != Path::new("-")) {
            return fs::read_to_string(path)
                .with_context(|| format!("Cannot read input from {}", path.display()));
        }

        if self.input.is_none() && stdin().is_terminal() {
            bail!("Cannot read the source: stdin is a terminal, pass --input <PATH>");
        }

        let mut buffer = String::new();
        stdin()
            .read_to_string(&mut buffer)
            .context("Cannot read input from stdin")?;

        Ok(buffer)
    }
}

/// Where a command writes its artifact, stdout by default.
#[derive(Debug, Parser)]
pub struct OutputArg {
    /// Write the result to PATH instead of stdout.
    ///
    /// Pass `-` for stdout, `./-` for a file named `-`.
    #[arg(long, short = 'o', global = true, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

impl OutputArg {
    /// Resolves the destination file, `None` meaning stdout.
    pub fn path(&self) -> Option<&Path> {
        self.output
            .as_deref()
            .filter(|path| *path != Path::new("-"))
    }

    /// Rejects an editor-driven run whose stdout is redirected.
    ///
    /// NOTE: the editor inherits stdout, so a pipe there swallows its
    /// interface. A path, `>(consumer)` included, keeps stdout free.
    pub fn check_editor_can_run(&self) -> Result<()> {
        if self.path().is_none() && !stdout().is_terminal() {
            bail!("Cannot open the editor: stdout is redirected, pass --output <PATH>");
        }

        Ok(())
    }
}

/// Reusable `-H KEY:VAL` repeatable flag, shared by template
/// builders and the editor-driven commands.
#[derive(Debug, Parser)]
pub struct HeaderRawArgs {
    /// Prefill the template with custom headers.
    ///
    /// Repeatable. Each value follows `KEY:VAL`.
    #[arg(long = "header", short = 'H', required = false)]
    #[arg(name = "header-raw", value_name = "KEY:VAL", value_parser = raw_header_parser)]
    pub raw: Vec<(String, String)>,
}

fn raw_header_parser(raw_header: &str) -> Result<(String, String), String> {
    if let Some((key, val)) = raw_header.split_once(':') {
        Ok((key.trim().to_owned(), val.trim().to_owned()))
    } else {
        Err(format!("cannot parse raw header {raw_header:?}"))
    }
}

/// Resolve a `--signature` CLI value: when the input shell-expands
/// to a readable file, replace it with the file's contents;
/// otherwise keep the raw value.
pub fn resolve_signature(raw: Option<String>) -> Option<String> {
    let raw = raw?;

    let path = match path_parser(&raw) {
        Ok(path) => path,
        Err(err) => {
            debug!("invalid signature path ({err}), using value as raw signature");
            return Some(raw);
        }
    };

    match fs::read_to_string(&path) {
        Ok(contents) => Some(contents),
        Err(err) => {
            debug!("cannot read signature file {path:?} ({err}), using value as raw signature");
            Some(raw)
        }
    }
}
