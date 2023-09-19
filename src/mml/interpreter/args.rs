use anyhow::Result;
use clap::Parser;
use log::warn;
use mml::message::ShowHeadersStrategy;
use shellexpand_utils::try_shellexpand_path;
use std::{fs, path::PathBuf};

use crate::mml::{format_stdin, format_str};

type MimeMessage = String;

/// Interpret the given MIME message as a MML message.
#[derive(Parser, Debug)]
pub struct InterpretCommand {
    /// Read the MIME message from the given file path.
    #[arg(value_parser = parse_mime)]
    mime: Option<MimeMessage>,

    /// Transfer given header from the MIME message to the interpreted
    /// MML message.
    #[arg(long, value_name = "HEADER")]
    show_header: Option<Vec<String>>,

    /// Skip headers and interpret only the body of the MIME message.
    #[arg(long)]
    hide_headers: bool,
}

impl InterpretCommand {
    /// Return the command-line provided message or read one from stdin.
    pub fn mime(self) -> MimeMessage {
        match self.mime {
            Some(mime) => mime,
            None => format_stdin(),
        }
    }

    pub fn show_headers(&self) -> ShowHeadersStrategy {
        if self.hide_headers {
            ShowHeadersStrategy::Only(Vec::new())
        } else {
            match &self.show_header {
                Some(headers) => ShowHeadersStrategy::Only(headers.clone()),
                None => ShowHeadersStrategy::All,
            }
        }
    }
}

fn parse_mime(raw: &str) -> Result<MimeMessage, String> {
    match try_shellexpand_path(raw) {
        Ok(path) => fs::read_to_string(PathBuf::from(path)).map_err(|e| e.to_string()),
        Err(err) => {
            warn!("{err}");
            warn!("invalid path, processing it as raw MIME message");
            Ok(format_str(raw))
        }
    }
}
