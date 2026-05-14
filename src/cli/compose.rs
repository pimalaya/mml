//! `mml compose` — editor-driven new-message composer. Builds a
//! draft template from the merged account + CLI args, opens
//! [`crate::cli::editor::edit_loop`] on it, then writes the compiled
//! MIME bytes to stdout. Designed to plug into himalaya v2 as
//! `[message.composer.mml] command = "mml compose"`.

use std::io::{stdout, Write};

use anyhow::{bail, Result};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{account::Account, args::HeaderRawArgs, editor::edit_loop},
    template::compose::builder::{TemplateBuilderCompose, TemplateComposeSignatureStyle},
};

/// Compose a new message: build the template, drive `$EDITOR`,
/// compile MML → MIME, then write the MIME bytes to stdout.
///
/// Designed to be plugged into himalaya as
/// `[message.composer.mml] command = "mml compose"`.
#[derive(Debug, Parser)]
pub struct ComposeCommand {
    /// Email address used as the `From:` header. Overrides the
    /// value from `[accounts.<name>]`.
    #[arg(long, short)]
    pub from: Option<String>,

    /// Display name placed before the address.
    #[arg(long, short = 'F')]
    pub from_name: Option<String>,

    /// Raw signature body (overrides config).
    #[arg(long, short)]
    pub signature: Option<String>,

    /// How to attach the signature.
    #[arg(long, short = 'S')]
    pub signature_style: Option<TemplateComposeSignatureStyle>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,

    /// Pre-fill the body before opening the editor.
    #[arg(long, short, default_value_t)]
    pub body: String,
}

impl ComposeCommand {
    pub fn execute(self, _printer: &mut impl Printer, account: Account) -> Result<()> {
        let account = account
            .with_from(self.from)
            .with_from_name(self.from_name)
            .with_signature(self.signature)
            .with_compose_signature_style(self.signature_style);

        let signature = account.signature();

        let Some(from) = account.from else {
            bail!("missing `From` email from both config and args");
        };

        let tpl = TemplateBuilderCompose {
            signature,
            signature_style: account.compose_signature_style.unwrap_or_default(),
            from,
            from_name: account.from_name,
            headers: self.headers.raw,
            body: self.body,
        }
        .build()?;

        match edit_loop(tpl)? {
            Some(mime) => {
                stdout().write_all(&mime)?;
                Ok(())
            }
            None => bail!("aborted by user"),
        }
    }
}
