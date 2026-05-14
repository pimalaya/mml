use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use log::debug;
use pimalaya_cli::{
    clap::parsers::path_parser,
    printer::{Message, Printer},
};

use crate::{
    cli::{
        account::Account,
        stdin::{format_stdin, format_str},
    },
    interpreter::{
        body::FilterParts,
        message::{FilterHeaders, MimeInterpreterBuilder},
    },
};

type MimeMessage = String;

/// Interpret a MIME message back into an MML message.
///
/// Header / part filters and attachment flags can be passed on the
/// command line and override the values merged from the global
/// `[read]` section and the selected account's `read` override.
/// With no config and no flags, the defaults match the previous
/// behavior (`FilterHeaders::All`, `FilterParts::All`, no attachment
/// saving).
#[derive(Debug, Parser)]
pub struct InterpretCommand {
    /// Path to a MIME file, or a raw MIME string.
    #[arg(value_parser = parse_mime)]
    pub mime: Option<MimeMessage>,

    /// Include only these headers (CSV).
    #[arg(
        long,
        value_name = "HEADER",
        value_delimiter = ',',
        conflicts_with = "exclude_header"
    )]
    pub include_header: Option<Vec<String>>,

    /// Exclude these headers (CSV).
    #[arg(
        long,
        value_name = "HEADER",
        value_delimiter = ',',
        conflicts_with = "include_header"
    )]
    pub exclude_header: Option<Vec<String>>,

    /// Interpret only these MIME parts (CSV).
    #[arg(
        long,
        value_name = "MIME",
        value_delimiter = ',',
        conflicts_with = "exclude_part"
    )]
    pub include_part: Option<Vec<String>>,

    /// Skip these MIME parts (CSV).
    #[arg(
        long,
        value_name = "MIME",
        value_delimiter = ',',
        conflicts_with = "include_part"
    )]
    pub exclude_part: Option<Vec<String>>,

    /// Render multipart wrappers as MML `<#multipart>` blocks.
    #[arg(long)]
    pub show_multiparts: bool,

    /// Download attachments to `save_attachments_dir`.
    #[arg(long)]
    pub save_attachments: bool,

    /// Directory where attachments are saved (and pointed to).
    #[arg(long, value_name = "DIR")]
    pub save_attachments_dir: Option<PathBuf>,

    /// Drop attachment parts from the output.
    #[arg(long)]
    pub hide_attachments: bool,

    /// Drop inline attachments from the output.
    #[arg(long)]
    pub hide_inline_attachments: bool,

    /// Trim out standard `-- \n` signatures from text bodies.
    #[arg(long)]
    pub hide_plain_texts_signature: bool,
}

impl InterpretCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        run(
            InterpreterSettings {
                mime: self.mime,
                include_header: self.include_header,
                exclude_header: self.exclude_header,
                include_part: self.include_part,
                exclude_part: self.exclude_part,
                show_multiparts: self.show_multiparts,
                save_attachments: self.save_attachments,
                save_attachments_dir: self.save_attachments_dir,
                hide_attachments: self.hide_attachments,
                hide_inline_attachments: self.hide_inline_attachments,
                hide_plain_texts_signature: self.hide_plain_texts_signature,
            },
            printer,
            account,
        )
    }
}

/// Plain settings bag fed into [`run`].
///
/// Decouples the clap-derived [`InterpretCommand`] from the
/// stdin-driven `cli::read::ReadCommand` so both can share the
/// merge-and-render path without one masquerading as the other.
pub(crate) struct InterpreterSettings {
    pub mime: Option<String>,
    pub include_header: Option<Vec<String>>,
    pub exclude_header: Option<Vec<String>>,
    pub include_part: Option<Vec<String>>,
    pub exclude_part: Option<Vec<String>>,
    pub show_multiparts: bool,
    pub save_attachments: bool,
    pub save_attachments_dir: Option<PathBuf>,
    pub hide_attachments: bool,
    pub hide_inline_attachments: bool,
    pub hide_plain_texts_signature: bool,
}

/// Shared interpretation entry point used by both `mml interpret` and
/// `mml read`.
pub(crate) fn run(
    settings: InterpreterSettings,
    printer: &mut impl Printer,
    account: Account,
) -> Result<()> {
    let account = account
        .with_read_include_headers(settings.include_header)
        .with_read_exclude_headers(settings.exclude_header)
        .with_read_include_parts(settings.include_part)
        .with_read_exclude_parts(settings.exclude_part)
        .with_read_show_multiparts(flag(settings.show_multiparts))
        .with_read_save_attachments(flag(settings.save_attachments))
        .with_read_save_attachments_dir(settings.save_attachments_dir)
        .with_read_hide_attachments(flag(settings.hide_attachments))
        .with_read_hide_inline_attachments(flag(settings.hide_inline_attachments))
        .with_read_hide_plain_texts_signature(flag(settings.hide_plain_texts_signature));

    let filter_headers = match (account.read_exclude_headers, account.read_include_headers) {
        (Some(headers), _) => FilterHeaders::Exclude(headers),
        (None, Some(headers)) => FilterHeaders::Include(headers),
        (None, None) => FilterHeaders::All,
    };

    let filter_parts = match (account.read_exclude_parts, account.read_include_parts) {
        (Some(parts), _) => FilterParts::Exclude(parts),
        (None, Some(parts)) if parts.len() == 1 => {
            FilterParts::Only(parts.into_iter().next().unwrap())
        }
        (None, Some(parts)) => FilterParts::Include(parts),
        (None, None) => FilterParts::All,
    };

    let show_multiparts = account.read_show_multiparts.unwrap_or(false);
    let save_attachments = account.read_save_attachments.unwrap_or(false);
    let save_attachments_dir = account.read_save_attachments_dir;
    let show_attachments = !account.read_hide_attachments.unwrap_or(false);
    let show_inline_attachments = !account.read_hide_inline_attachments.unwrap_or(false);
    let show_plain_texts_signature = !account.read_hide_plain_texts_signature.unwrap_or(false);

    let interpreter = MimeInterpreterBuilder::new()
        .with_show_headers(filter_headers)
        .with_filter_parts(filter_parts)
        .with_show_multiparts(show_multiparts)
        .with_save_attachments(save_attachments)
        .with_save_some_attachments_dir(save_attachments_dir)
        .with_show_attachments(show_attachments)
        .with_show_inline_attachments(show_inline_attachments)
        .with_show_plain_texts_signature(show_plain_texts_signature)
        .build();

    let mime = settings.mime.unwrap_or_else(format_stdin);
    let mml = interpreter
        .from_bytes(mime.as_bytes())
        .context("cannot interpret MIME message")?;

    printer.out(Message::new(mml))
}

/// CLI bool flags are one-way: `true` becomes `Some(true)` (so the
/// merge picks it up), `false` becomes `None` (so the lower layers
/// remain in effect).
fn flag(v: bool) -> Option<bool> {
    if v {
        Some(true)
    } else {
        None
    }
}

fn parse_mime(raw: &str) -> Result<MimeMessage, String> {
    match path_parser(raw) {
        Ok(path) => fs::read_to_string(path).map_err(|err| err.to_string()),
        Err(err) => {
            debug!("invalid path ({err}), processing arg as raw MIME message");
            Ok(format_str(raw))
        }
    }
}
