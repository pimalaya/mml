use anyhow::{bail, Result};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{account::Account, args::HeaderRawArgs},
    template::compose::builder::{TemplateBuilderCompose, TemplateComposeSignatureStyle},
};

/// Build a new message template (headers + body, no `$EDITOR`).
///
/// Pure transformation: the args go in, the template is printed on
/// stdout. Use `mml compose` for the full template + editor + compile
/// flow.
///
/// Identity and style fields default to the merged account (global
/// + `[accounts.<name>]`); CLI flags override them.
#[derive(Debug, Parser)]
pub struct TemplateComposeCommand {
    /// Email address used as the `From:` header. Overrides the
    /// value from `[accounts.<name>]`.
    #[arg(long, short, value_name = "EMAIL")]
    pub from: Option<String>,

    /// Display name placed before the address.
    #[arg(long, short = 'F', value_name = "NAME")]
    pub from_name: Option<String>,

    /// Raw signature body (overrides config).
    #[arg(long, short)]
    pub signature: Option<String>,

    /// How to attach the signature.
    #[arg(long, short = 'S', value_name = "STYLE")]
    pub signature_style: Option<TemplateComposeSignatureStyle>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,

    #[arg(long, short, default_value_t)]
    pub body: String,
}

impl TemplateComposeCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
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

        printer.out(tpl)
    }
}
