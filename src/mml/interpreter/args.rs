use anyhow::Result;
use clap::Parser;
use log::warn;
use shellexpand_utils::try_shellexpand_path;
use std::{fs, path::PathBuf};

use crate::mml::{format_stdin, format_str};

type MimeMessage = String;

/// Interpret the given MIME message as a MML message
#[derive(Parser, Debug)]
pub struct InterpretCommand {
    /// Read the mssage from the given file path.
    #[arg(value_parser = parse_mime)]
    mime: Option<MimeMessage>,
}

impl InterpretCommand {
    /// Return the command-line provided message or read one from stdin.
    pub fn mime(self) -> MimeMessage {
        match self.mime {
            Some(mime) => mime,
            None => format_stdin(),
        }
    }
}

fn parse_mime(raw: &str) -> Result<MimeMessage, String> {
    let mime = match try_shellexpand_path(raw) {
        Ok(path) => fs::read_to_string(PathBuf::from(path)).map_err(|e| e.to_string())?,
        Err(err) => {
            warn!("{err}");
            warn!("invalid path, processing it as raw MIME message");
            format_str(raw)
        }
    };
    Ok(mime)
}
