//! `mml interpret`: MIME to MML pipeline. Reads a MIME message from
//! the positional argument or stdin, applies the merged `[read]`
//! defaults (overridable per flag), and writes the rendered MML on
//! stdout. Exposed under the visible alias `mml read` so callers can
//! pipe it directly from `himalaya messages read --raw <id>` or any
//! other MIME source.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::{
    cli::{account::Account, args::MessageArg},
    interpreter::{
        body::FilterParts,
        message::{FilterHeaders, MimeInterpreterBuilder},
    },
};

/// Interpret a MIME message back into an MML message.
#[derive(Debug, Parser)]
pub struct InterpretCommand {
    /// Path to a MIME file, a raw MIME string, or nothing when piped
    /// on stdin.
    #[command(flatten)]
    pub mime: MessageArg,

    /// Whitelist of header names to keep; all others are dropped.
    #[arg(long, conflicts_with = "exclude_header")]
    #[arg(value_name = "HEADER", value_delimiter = ',')]
    pub include_header: Option<Vec<String>>,
    /// Blacklist of header names to drop; all others are kept.
    #[arg(long, conflicts_with = "include_header")]
    #[arg(value_name = "HEADER", value_delimiter = ',')]
    pub exclude_header: Option<Vec<String>>,

    /// Whitelist of MIME content types to render; all others are
    /// dropped.
    #[arg(long, conflicts_with = "exclude_part")]
    #[arg(value_name = "MIME", value_delimiter = ',')]
    pub include_part: Option<Vec<String>>,
    /// Blacklist of MIME content types to skip; all others are
    /// rendered.
    #[arg(long, conflicts_with = "include_part")]
    #[arg(value_name = "MIME", value_delimiter = ',')]
    pub exclude_part: Option<Vec<String>>,

    /// Render the `multipart/*` part markers in the output.
    #[arg(long)]
    pub show_multiparts: bool,

    /// Save attachments to disk while rendering.
    #[arg(long)]
    pub save_attachments: bool,
    /// Directory where attachments are written when
    /// `--save-attachments` is set. Defaults to `~/Downloads`.
    #[arg(long, value_name = "DIR")]
    pub save_attachments_dir: Option<PathBuf>,

    /// Suppress attachment markers in the rendered output.
    #[arg(long)]
    pub hide_attachments: bool,
    /// Suppress inline-attachment markers only.
    #[arg(long)]
    pub hide_inline_attachments: bool,
    /// Strip the trailing `-- \n` signature block from plain-text
    /// parts.
    #[arg(long)]
    pub hide_plain_texts_signature: bool,
}

impl InterpretCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mime = self.mime.parse().ok().unwrap_or_default();

        let account = account
            .with_read_include_headers(self.include_header)
            .with_read_exclude_headers(self.exclude_header)
            .with_read_include_parts(self.include_part)
            .with_read_exclude_parts(self.exclude_part)
            .with_read_show_multiparts(flag(self.show_multiparts))
            .with_read_save_attachments(flag(self.save_attachments))
            .with_read_save_attachments_dir(self.save_attachments_dir)
            .with_read_hide_attachments(flag(self.hide_attachments))
            .with_read_hide_inline_attachments(flag(self.hide_inline_attachments))
            .with_read_hide_plain_texts_signature(flag(self.hide_plain_texts_signature));

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

        let mml = interpreter
            .from_bytes(mime.as_bytes())
            .context("cannot interpret MIME message")?;

        printer.out(Message::new(mml))
    }
}

/// CLI bool flags are one-way: `true` becomes `Some(true)` (so the
/// merge picks it up), `false` becomes `None` (so the lower layers
/// remain in effect).
fn flag(v: bool) -> Option<bool> {
    if v { Some(true) } else { None }
}
