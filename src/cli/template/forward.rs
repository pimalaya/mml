//! `mml template forward`: builds a forward MML template from the
//! source MIME message and the merged account + CLI args, then
//! prints it on stdout without opening the editor or compiling. Use
//! `mml forward` for the full edit and compile flow.

use anyhow::{Result, bail};
use clap::Parser;
use mail_parser::MessageParser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, MessageArg, resolve_signature},
    },
    template::forward::{
        TemplateBuilderForward, TemplateForwardPostingStyle, TemplateForwardSignatureStyle,
    },
};

/// Build a new forward template, then print it on stdout.
#[derive(Debug, Parser)]
pub struct TemplateForwardCommand {
    #[command(flatten)]
    pub mime: MessageArg,

    /// Email address used as the `From:` header. Overrides the
    /// value from `[accounts.<name>]`.
    #[arg(short, long)]
    pub from: Option<String>,
    /// Display name placed before the `From:` address.
    #[arg(short = 'F', long)]
    pub from_name: Option<String>,

    /// Signature body, or path to a file containing it. Overrides
    /// `[accounts.<name>].signature`.
    #[arg(short, long)]
    pub signature: Option<String>,
    /// How to attach the signature (`inlined`, `attached`,
    /// `hidden`).
    #[arg(short = 'S', long)]
    pub signature_style: Option<TemplateForwardSignatureStyle>,

    /// How to attach the forwarded message (`top`, `attached`).
    #[arg(short = 'P', long)]
    pub posting_style: Option<TemplateForwardPostingStyle>,
    /// Line printed above the forwarded message. Supports `\n` to
    /// insert a newline.
    #[arg(short = 'Q', long)]
    pub quote_headline: Option<String>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,
    /// Pre-fill the body of the printed template.
    #[arg(short, long, default_value_t)]
    pub body: String,
}

impl TemplateForwardCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let account = account
            .with_from(self.from)
            .with_from_name(self.from_name)
            .with_signature(resolve_signature(self.signature))
            .with_forward_signature_style(self.signature_style)
            .with_forward_posting_style(self.posting_style)
            .with_reply_quote_headline(self.quote_headline.map(|q| q.replace("\\n", "\n")));

        let signature = account.signature();

        let Some(from) = account.from else {
            bail!("Missing `From` email from both config and args");
        };

        let mime = self.mime.parse()?.into_bytes();

        let Some(msg) = MessageParser::new().parse(&mime) else {
            bail!("Invalid or malformed MIME message");
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

        printer.out(tpl)
    }
}
