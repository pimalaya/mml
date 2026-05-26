//! `mml forward` — editor-driven forward composer. Same shape as
//! [`crate::cli::reply`], built around
//! [`crate::template::forward::TemplateBuilderForward`].

use std::io::{stdout, Write};

use anyhow::{bail, Result};
use clap::Parser;
use mail_parser::MessageParser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{account::Account, args::HeaderRawArgs, editor::edit_loop, stdin::format_stdin},
    template::forward::{
        TemplateBuilderForward, TemplateForwardPostingStyle, TemplateForwardSignatureStyle,
    },
};

/// Forward a message: read source MIME on stdin, build the forward
/// template, drive `$EDITOR`, compile MML → MIME, write to stdout.
#[derive(Debug, Parser)]
pub struct ForwardCommand {
    #[arg(long, short)]
    pub from: Option<String>,

    #[arg(long, short = 'F')]
    pub from_name: Option<String>,

    #[arg(long, short)]
    pub signature: Option<String>,

    #[arg(long, short = 'S')]
    pub signature_style: Option<TemplateForwardSignatureStyle>,

    #[arg(long, short = 'P')]
    pub posting_style: Option<TemplateForwardPostingStyle>,

    #[arg(long, short = 'Q')]
    pub quote_headline: Option<String>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,

    #[arg(long, short, default_value_t)]
    pub body: String,
}

impl ForwardCommand {
    pub fn execute(self, _printer: &mut impl Printer, account: Account) -> Result<()> {
        let account = account
            .with_from(self.from)
            .with_from_name(self.from_name)
            .with_signature(self.signature)
            .with_forward_signature_style(self.signature_style)
            .with_forward_posting_style(self.posting_style)
            .with_forward_quote_headline(self.quote_headline);

        let signature = account.signature();

        let Some(from) = account.from else {
            bail!("missing `From` email from both config and args");
        };

        let mime = format_stdin();
        let Some(msg) = MessageParser::new().parse(mime.as_bytes()) else {
            bail!("invalid or malformed MIME message on stdin");
        };

        let tpl = TemplateBuilderForward {
            signature,
            signature_style: account.forward_signature_style.unwrap_or_default(),
            posting_style: account.forward_posting_style.unwrap_or_default(),
            quote_headline: account.forward_quote_headline.unwrap_or_default(),
            from,
            from_name: account.from_name,
            headers: self.headers.raw,
            body: self.body,
        }
        .build(&msg)?;

        match edit_loop(tpl)? {
            Some(mime) => {
                stdout().write_all(&mime)?;
                Ok(())
            }
            None => bail!("aborted by user"),
        }
    }
}
