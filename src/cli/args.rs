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

use crate::cli::account::Account;

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

/// Who the message comes from, shared by every command building a
/// template.
///
/// The signature style stays per command: compose and forward accept
/// `inlined`, reply accepts the quote-relative placements instead.
#[derive(Debug, Parser)]
pub struct IdentityArgs {
    /// Email address used as the `From:` header. Overrides the
    /// value from `[accounts.<name>]`.
    #[arg(long, short)]
    pub from: Option<String>,
    /// Display name placed before the `From:` address.
    #[arg(long, short = 'F')]
    pub from_name: Option<String>,
    /// Signature body, or path to a file containing it. Overrides
    /// `[accounts.<name>].signature`.
    #[arg(long, short)]
    pub signature: Option<String>,
}

impl IdentityArgs {
    /// Overrides the account's identity with whatever was passed.
    pub fn merge_into(self, account: Account) -> Account {
        account
            .with_from(self.from)
            .with_from_name(self.from_name)
            .with_signature(self.signature.map(resolve_signature))
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

/// Resolves a `--signature` value: a readable path becomes the file's
/// contents, anything else stays the raw signature.
fn resolve_signature(raw: String) -> String {
    let path = match path_parser(&raw) {
        Ok(path) => path,
        Err(err) => {
            debug!("invalid signature path ({err}), using value as raw signature");
            return raw;
        }
    };

    match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            debug!("cannot read signature file {path:?} ({err}), using value as raw signature");
            raw
        }
    }
}
