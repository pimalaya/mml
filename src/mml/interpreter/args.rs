use std::{fs, path::PathBuf};

use clap::Parser;
use color_eyre::Result;
use mml::message::{FilterHeaders, FilterParts};
use pimalaya_tui::terminal::cli::arg::path_parser;
use tracing::warn;

use crate::mml::{format_stdin, format_str};

type MimeMessage = String;

/// Interpret the given MIME message as a MML message.
#[derive(Parser, Debug)]
pub struct InterpretCommand {
    /// Read the MIME message from the given file path.
    #[arg(value_parser = parse_mime)]
    mime: Option<MimeMessage>,

    /// Include header to the interpreted message.
    #[arg(
        long,
        value_name = "HEADER",
        value_delimiter = ',',
        conflicts_with = "exclude_header"
    )]
    include_header: Option<Vec<String>>,

    /// Exclude header from the interpreted message.
    #[arg(
        long,
        value_name = "HEADER",
        value_delimiter = ',',
        conflicts_with = "include_header"
    )]
    exclude_header: Option<Vec<String>>,

    /// Include parts to intepret by their MIME type.
    #[arg(
        long,
        value_name = "MIME",
        value_delimiter = ',',
        conflicts_with = "exclude_part"
    )]
    include_part: Option<Vec<String>>,

    /// Exclude parts to interpret by their MIME type.
    #[arg(
        long,
        value_name = "MIME",
        value_delimiter = ',',
        conflicts_with = "include_part"
    )]
    exclude_part: Option<Vec<String>>,

    /// Enable interpretation of multiparts.
    #[arg(long)]
    show_multiparts: bool,

    /// Save automatically attachments found in the original MIME
    /// message to the `save_attachments_dir` directory.
    #[arg(long)]
    save_attachments: bool,

    /// Define the directory attachments should point to.
    ///
    /// If `save_attachments` is true, attachments are automatically
    /// downloaded to this directory.
    #[arg(long, value_name = "DIR")]
    save_attachments_dir: Option<PathBuf>,

    /// If true, disable interpretation of all attachments.
    #[arg(long)]
    hide_attachments: bool,

    /// If true, disable interpretation of attachments with a content
    /// disposition set to inline.
    #[arg(long)]
    hide_inline_attachments: bool,

    /// If true, trim out signatures from text bodies.
    ///
    /// Only standard signatures can be trimmed out. Plain text found
    /// after a `-- \n` is considered a standard signature.
    #[arg(long)]
    hide_plain_texts_signature: bool,
}

impl InterpretCommand {
    pub fn mime(self) -> MimeMessage {
        match self.mime {
            Some(mime) => mime,
            None => format_stdin(),
        }
    }

    pub fn filter_headers(&self) -> FilterHeaders {
        if let Some(mime_types) = &self.exclude_header {
            FilterHeaders::Exclude(mime_types.clone())
        } else {
            match &self.include_header {
                Some(mime_types) => FilterHeaders::Include(mime_types.clone()),
                None => FilterHeaders::All,
            }
        }
    }

    pub fn filter_parts(&self) -> FilterParts {
        if let Some(mime_types) = &self.exclude_part {
            FilterParts::Exclude(mime_types.clone())
        } else {
            match &self.include_part {
                Some(mime_types) if mime_types.len() == 1 => {
                    FilterParts::Only(mime_types[0].clone())
                }
                Some(mime_types) => FilterParts::Include(mime_types.clone()),
                None => FilterParts::All,
            }
        }
    }

    pub fn show_multiparts(&self) -> bool {
        self.show_multiparts
    }

    pub fn save_attachments(&self) -> bool {
        self.save_attachments
    }

    pub fn save_attachments_dir(&self) -> Option<PathBuf> {
        self.save_attachments_dir.clone()
    }

    pub fn show_attachments(&self) -> bool {
        !self.hide_attachments
    }

    pub fn show_inline_attachments(&self) -> bool {
        !self.hide_inline_attachments
    }

    pub fn show_plain_texts_signature(&self) -> bool {
        !self.hide_plain_texts_signature
    }
}

fn parse_mime(raw: &str) -> Result<MimeMessage, String> {
    match path_parser(raw) {
        Ok(path) => fs::read_to_string(path).map_err(|err| err.to_string()),
        Err(err) => {
            warn!(?err, "invalid path, processing arg as raw MIME message");
            Ok(format_str(&raw))
        }
    }
}
