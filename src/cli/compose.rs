//! `mml compose`: editor-driven new-message composer. Builds a
//! draft template from the merged account and CLI args, opens
//! [`crate::cli::utils::editor::edit_loop`] on it, then writes the
//! compiled MIME bytes either to the optional output path (when
//! given) or to stdout. The path-arg form is what parent pipelines
//! (e.g. `himalaya messages send <path>`) and process-substitution
//! (`mml compose >(himalaya messages send)`) rely on to keep
//! `$EDITOR`'s stdout connected to the terminal.

use std::{
    fs,
    io::{Write, stdout},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
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

    /// Optional file path to write the compiled MIME message to.
    /// When omitted, the MIME bytes are written to stdout. Passing a
    /// path lets the caller pipe mml into a consumer that opens an
    /// editor (which inherits mml's terminal stdout) without the
    /// stdout pipe corrupting the editor's UI.
    #[arg(value_name = "OUTPUT")]
    pub output: Option<PathBuf>,
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

        match edit_loop(tpl, self.output.is_some())? {
            Some(mime) => match self.output {
                Some(path) => fs::write(&path, &mime)
                    .with_context(|| format!("cannot write MIME to {}", path.display())),
                None => Ok(stdout().write_all(&mime)?),
            },
            None => bail!("aborted by user"),
        }
    }
}
