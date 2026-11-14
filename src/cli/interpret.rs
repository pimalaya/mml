//! `mml interpret`: MIME to MML pipeline. Reads a MIME message from
//! `--input` or stdin, applies the merged `[read]` defaults
//! (overridable per flag), and prints the rendered MML.
//!
//! Aliased `mml read`, so it can be piped straight from
//! `himalaya messages read --raw <id>` or any other MIME source.

use std::{fmt, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use pimalaya_cli::{error::PartialOutcome, printer::Printer};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    cli::{account::Account, args::InputArg, crypto},
    interpreter::{
        body::{MmlFilterParts, MmlMimeBodyInterpreter},
        message::{MmlFilterHeaders, MmlInterpretOptions},
    },
    pgp::MmlPgpVerdict,
};

/// Interpret a MIME message back into an MML message.
///
/// Exits 0 when the message carries no signature or a valid one, 3
/// when a signature is invalid, and 1 on any other failure. The
/// message is rendered either way.
#[derive(Debug, Parser)]
pub struct InterpretCommand {
    #[command(flatten)]
    pub input: InputArg,

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
    /// `--save-attachments` is set. Defaults to the system temporary
    /// directory.
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
    /// Interprets the source and prints the MML message.
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mime = self.input.read()?;

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
            (Some(headers), _) => MmlFilterHeaders::Exclude(headers),
            (None, Some(headers)) => MmlFilterHeaders::Include(headers),
            (None, None) => MmlFilterHeaders::All,
        };

        let filter_parts = match (account.read_exclude_parts, account.read_include_parts) {
            (Some(parts), _) => MmlFilterParts::Exclude(parts),
            (None, Some(parts)) if parts.len() == 1 => {
                MmlFilterParts::Only(parts.into_iter().next().unwrap())
            }
            (None, Some(parts)) => MmlFilterParts::Include(parts),
            (None, None) => MmlFilterParts::All,
        };

        let show_multiparts = account.read_show_multiparts.unwrap_or(false);
        let save_attachments = account.read_save_attachments.unwrap_or(false);
        let save_attachments_dir = account.read_save_attachments_dir;
        let show_attachments = !account.read_hide_attachments.unwrap_or(false);
        let show_inline_attachments = !account.read_hide_inline_attachments.unwrap_or(false);
        let show_plain_texts_signature = !account.read_hide_plain_texts_signature.unwrap_or(false);

        let interpreter = MmlInterpretOptions {
            show_headers: filter_headers,
            body: MmlMimeBodyInterpreter {
                filter_parts,
                show_multiparts,
                save_attachments,
                save_attachments_dir: save_attachments_dir
                    .unwrap_or_else(MmlMimeBodyInterpreter::default_save_attachments_dir),
                show_attachments,
                show_inline_attachments,
                show_plain_texts_signature,
                ..Default::default()
            },
        };

        let (mml, signatures) = crypto::interpret(mime.as_bytes(), interpreter)
            .context("cannot interpret MIME message")?;

        let invalid = signatures.contains(&MmlPgpVerdict::Invalid);
        let signatures = signatures.into_iter().map(SignatureOutput::from);
        let signatures = signatures.collect();

        printer.out(InterpretOutput { mml, signatures })?;

        // A bad signature does not stop the message from being read.
        if invalid {
            return Err(PartialOutcome::default().into());
        }

        Ok(())
    }
}

/// CLI bool flags are one-way: `true` becomes `Some(true)` (so the
/// merge picks it up), `false` becomes `None` (so the lower layers
/// remain in effect).
fn flag(v: bool) -> Option<bool> {
    if v { Some(true) } else { None }
}

/// What `interpret` prints: the rendered MML, plus what came of any
/// signature the message carried.
///
/// The text form is the MML alone, the raw artifact a composer or an
/// editor consumes; only `--json` surfaces the signatures.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InterpretOutput {
    /// The rendered MML message.
    pub mml: String,
    /// The verdict of each signature the message carried, in order.
    pub signatures: Vec<SignatureOutput>,
}

/// The verdict of one signature.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignatureOutput {
    /// Whether the signature checks out.
    pub valid: bool,
    /// The fingerprint of the signing key, `null` when invalid.
    pub fingerprint: Option<String>,
}

impl From<MmlPgpVerdict> for SignatureOutput {
    fn from(verdict: MmlPgpVerdict) -> Self {
        match verdict {
            MmlPgpVerdict::Valid(fingerprint) => Self {
                valid: true,
                fingerprint: Some(fingerprint),
            },
            _ => Self {
                valid: false,
                fingerprint: None,
            },
        }
    }
}

impl fmt::Display for InterpretOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mml)
    }
}
