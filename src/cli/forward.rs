//! `mml forward`: editor-driven forward composer. Same shape as
//! [`crate::cli::reply`], built around
//! [`crate::template::forward::TemplateBuilderForward`].

use std::io::{Write, stdout};

use anyhow::{Result, bail};
use clap::Parser;
use mail_parser::MessageParser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, MessageArg, resolve_signature},
        utils::editor::edit_loop,
    },
    template::forward::{
        TemplateBuilderForward, TemplateForwardPostingStyle, TemplateForwardSignatureStyle,
    },
};

/// Forward the given message interactively, using $EDITOR.
#[derive(Debug, Parser)]
pub struct ForwardCommand {
    #[command(flatten)]
    pub mime: MessageArg,

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
    pub signature_style: Option<TemplateForwardSignatureStyle>,

    /// How to attach the forwarded message (`top`, `attached`).
    #[arg(long, short = 'P')]
    pub posting_style: Option<TemplateForwardPostingStyle>,
    /// Line printed above the forwarded message. Supports `\n` to
    /// insert a newline.
    #[arg(long, short = 'Q')]
    pub quote_headline: Option<String>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,
    /// Pre-fill the body before opening the editor.
    #[arg(long, short, default_value_t)]
    pub body: String,
}

impl ForwardCommand {
    pub fn execute(self, _printer: &mut impl Printer, account: Account) -> Result<()> {
        let account = account
            .with_from(self.from)
            .with_from_name(self.from_name)
            .with_signature(resolve_signature(self.signature))
            .with_forward_signature_style(self.signature_style)
            .with_forward_posting_style(self.posting_style)
            .with_forward_quote_headline(self.quote_headline);

        let signature = account.signature();

        let Some(from) = account.from else {
            bail!("missing `From` email from both config and args");
        };

        let mime = self.mime.parse()?.into_bytes();

        let Some(msg) = MessageParser::new().parse(&mime) else {
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
