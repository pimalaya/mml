#![cfg_attr(
    not(all(feature = "compiler", feature = "interpreter")),
    allow(dead_code, unused_imports)
)]

use anyhow::Result;
use clap::Parser;
use log::warn;
use shellexpand_utils::try_shellexpand_path;
use std::{
    fs,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

pub type MmlMessage = String;
pub type MimeMessage = String;

/// Compile the given MML message to a valid MIME message
#[cfg(feature = "compiler")]
#[derive(Parser, Debug)]
pub struct CompileCommand {
    /// Read the mssage from the given file path.
    #[arg(value_parser = parse_mml)]
    mml: Option<MmlMessage>,
}

#[cfg(feature = "compiler")]
impl CompileCommand {
    /// Return the command-line provided message or read one from stdin.
    pub fn mml(self) -> MmlMessage {
        match self.mml {
            Some(mml) => mml,
            None => format_stdin(),
        }
    }
}

/// Interpret the given MIME message as a MML message
#[cfg(feature = "interpreter")]
#[derive(Parser, Debug)]
pub struct InterpreterCommand {
    /// Read the mssage from the given file path.
    #[arg(value_parser = parse_mime)]
    mime: Option<MimeMessage>,
}

#[cfg(feature = "interpreter")]
impl InterpreterCommand {
    /// Return the command-line provided message or read one from stdin.
    pub fn mime(self) -> MimeMessage {
        match self.mime {
            Some(mime) => mime,
            None => format_stdin(),
        }
    }
}

#[cfg(feature = "compiler")]
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

#[cfg(feature = "interpreter")]
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

fn format_str(input: &str) -> String {
    let input = input.replace("\\r", "").replace("\\n", "\n");
    let mut lines = input.lines();
    let mut output = String::new();

    while let Some(ref line) = lines.next() {
        output.push_str(line);
        output.push('\r');
        output.push('\n');
    }

    output
}

fn format_stdin() -> String {
    let mut lines = BufReader::new(io::stdin()).lines();
    let mut output = String::new();

    while let Some(Ok(ref line)) = lines.next() {
        output.push_str(line);
        output.push('\r');
        output.push('\n');
    }

    output
}
