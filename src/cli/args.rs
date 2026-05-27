//! Reusable clap arg groups shared between the editor-driven
//! commands and their `template` counterparts.

use std::{
    fs,
    io::{IsTerminal, stdin},
};

use anyhow::bail;
use clap::Parser;
use log::debug;
use pimalaya_cli::clap::parsers::path_parser;

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

#[derive(Debug, Parser)]
pub struct MessageArg {
    /// Can be a path to a file, raw message contents or nothing if piped via
    /// standard input.
    #[arg(name = "message-raw", value_name = "MESSAGE", trailing_var_arg = true)]
    pub raw: Vec<String>,
}

impl MessageArg {
    pub fn parse(&self) -> anyhow::Result<String> {
        if !self.raw.is_empty() {
            let mime = self.raw.join(" ").replace("\\r", "").replace("\\n", "\r\n");

            let Ok(path) = path_parser(&mime) else {
                return Ok(mime);
            };

            let Ok(mime) = fs::read_to_string(path) else {
                return Ok(mime);
            };

            return Ok(mime);
        }

        if !stdin().is_terminal() {
            let lines: Vec<_> = stdin().lines().map_while(Result::ok).collect();
            return Ok(lines.join("\r\n"));
        }

        bail!("Message cannot be empty");
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
