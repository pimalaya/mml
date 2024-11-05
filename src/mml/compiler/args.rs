use std::fs;

use clap::Parser;
use color_eyre::Result;
use pimalaya_tui::terminal::cli::arg::path_parser;
use tracing::warn;

use crate::mml::{format_stdin, format_str};

type MmlMessage = String;

/// Compile the given MML message to a valid MIME message
#[derive(Parser, Debug)]
pub struct CompileCommand {
    /// Read the message from the given file path.
    #[arg(value_parser = parse_mml)]
    mml: Option<MmlMessage>,
}

impl CompileCommand {
    /// Return the command-line provided message or read one from stdin.
    pub fn mml(self) -> MmlMessage {
        match self.mml {
            Some(mml) => mml,
            None => format_stdin(),
        }
    }
}

fn parse_mml(raw: &str) -> Result<MmlMessage, String> {
    match path_parser(raw) {
        Ok(path) => fs::read_to_string(path).map_err(|err| err.to_string()),
        Err(err) => {
            warn!(?err, "invalid path, processing arg as raw MML message");
            Ok(format_str(&raw))
        }
    }
}
