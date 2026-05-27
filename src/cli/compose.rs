//! `mml compose`: editor-driven new-message composer. Builds a
//! draft template from the merged account and CLI args, opens
//! [`crate::cli::utils::editor::edit_loop`] on it, then writes the
//! compiled MIME bytes to stdout. Designed to plug into himalaya v2
//! as `[message.composer.mml] command = "mml compose"`.

use std::io::{Write, stdout};

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, resolve_signature},
        utils::editor::edit_loop,
    },
    template::compose::{TemplateBuilderCompose, TemplateComposeSignatureStyle},
};

/// Compose a new message interactively, using $EDITOR.
#[derive(Debug, Parser)]
pub struct ComposeCommand {
    /// Email address used as the `From:` header. Overrides the
    /// value from `[accounts.<name>]`.
    #[arg(long, short)]
    pub from: Option<String>,
    /// Display name placed before the `From:` address.
    #[arg(long, short = 'F')]
    pub from_name: Option<String>,

    /// Signature body, or path to a file containing it. Overrides
    /// `[accounts.<name>].signature`.
    #[arg(long, short)]
    pub signature: Option<String>,
    /// How to attach the signature (`inlined`, `attached`,
    /// `hidden`).
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
            .with_signature(resolve_signature(self.signature))
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
