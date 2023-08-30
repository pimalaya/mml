use anyhow::Result;
use clap::Parser;
use log::warn;
use shellexpand_utils::try_shellexpand_path;
use std::{fs, path::PathBuf};

use crate::mml::{format_stdin, format_str};

type MmlMessage = String;

/// Compile the given MML message to a valid MIME message
#[derive(Parser, Debug)]
pub struct CompileCommand {
    /// Read the mssage from the given file path.
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
    let mml = match try_shellexpand_path(raw) {
        Ok(path) => fs::read_to_string(PathBuf::from(path)).map_err(|e| e.to_string())?,
        Err(err) => {
            warn!("{err}");
            warn!("invalid path, processing it as raw MML message");
            format_str(raw)
        }
    };

    Ok(mml)
}
