//! `mml read` — himalaya `read-with` adapter. Reads MIME on stdin,
//! renders it through [`crate::interpreter`] using the merged
//! `[read]` settings, writes text on stdout.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::account::Account,
    interpreter::cli::{run, InterpreterSettings},
};

/// Reader command for himalaya's `read-with` flow.
///
/// Reads a MIME message on stdin, applies the merged `[read]`
/// defaults from the resolved account (CLI flags override), and
/// writes the interpreted MML/text on stdout.
#[derive(Debug, Parser)]
pub struct ReadCommand {
    #[arg(
        long,
        value_name = "HEADER",
        value_delimiter = ',',
        conflicts_with = "exclude_header"
    )]
    pub include_header: Option<Vec<String>>,

    #[arg(
        long,
        value_name = "HEADER",
        value_delimiter = ',',
        conflicts_with = "include_header"
    )]
    pub exclude_header: Option<Vec<String>>,

    #[arg(
        long,
        value_name = "MIME",
        value_delimiter = ',',
        conflicts_with = "exclude_part"
    )]
    pub include_part: Option<Vec<String>>,

    #[arg(
        long,
        value_name = "MIME",
        value_delimiter = ',',
        conflicts_with = "include_part"
    )]
    pub exclude_part: Option<Vec<String>>,

    #[arg(long)]
    pub show_multiparts: bool,

    #[arg(long)]
    pub save_attachments: bool,

    #[arg(long, value_name = "DIR")]
    pub save_attachments_dir: Option<PathBuf>,

    #[arg(long)]
    pub hide_attachments: bool,

    #[arg(long)]
    pub hide_inline_attachments: bool,

    #[arg(long)]
    pub hide_plain_texts_signature: bool,
}

impl ReadCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        run(
            InterpreterSettings {
                mime: None,
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
