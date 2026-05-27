//! `mml template compose`: builds a new-message MML template from
//! the merged account + CLI args and prints it on stdout, without
//! opening the editor or compiling to MIME. Use `mml compose` for
//! the full edit and compile flow.

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, resolve_signature},
    },
    template::compose::{TemplateBuilderCompose, TemplateComposeSignatureStyle},
};

/// Build a new message template, then print it on stdout.
#[derive(Debug, Parser)]
pub struct TemplateComposeCommand {
    /// Email address used as the `From:` header. Overrides the
    /// value from `[accounts.<name>]`.
    #[arg(long, short, value_name = "EMAIL")]
    pub from: Option<String>,
    /// Display name placed before the `From:` address.
    #[arg(long, short = 'F', value_name = "NAME")]
    pub from_name: Option<String>,

    /// Signature body, or path to a file containing it. Overrides
    /// `[accounts.<name>].signature`.
    #[arg(long, short)]
    pub signature: Option<String>,
    /// How to attach the signature (`inlined`, `attached`,
    /// `hidden`).
    #[arg(long, short = 'S', value_name = "STYLE")]
    pub signature_style: Option<TemplateComposeSignatureStyle>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,
    /// Pre-fill the body of the printed template.
    #[arg(long, short, default_value_t)]
    pub body: String,
}

impl TemplateComposeCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
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

        printer.out(tpl)
    }
}
