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
        args::{HeaderRawArgs, IdentityArgs, InputArg},
        crypto,
    },
    template::forward::{
        MmlTemplateForwardBuilder, MmlTemplateForwardPostingStyle, MmlTemplateForwardSignatureStyle,
    },
};

/// Build a new forward template, then print it on stdout.
#[derive(Debug, Parser)]
pub struct TemplateForwardCommand {
    #[command(flatten)]
    pub input: InputArg,

    #[command(flatten)]
    pub identity: IdentityArgs,
    /// How to attach the signature (`inlined`, `attached`,
    /// `hidden`).
    #[arg(short = 'S', long)]
    pub signature_style: Option<MmlTemplateForwardSignatureStyle>,

    /// How to attach the forwarded message (`top`, `attached`).
    #[arg(short = 'P', long)]
    pub posting_style: Option<MmlTemplateForwardPostingStyle>,
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
    /// Builds the template and prints it, without opening the editor.
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let account = self
            .identity
            .merge_into(account)
            .with_forward_signature_style(self.signature_style)
            .with_forward_posting_style(self.posting_style)
            .with_reply_quote_headline(self.quote_headline.map(|q| q.replace("\\n", "\n")));

        let signature = account.signature();

        let Some(from) = account.from else {
            bail!("Missing `From` email from both config and args");
        };

        let mime = self.input.read()?.into_bytes();

        let Some(msg) = MessageParser::new().parse(&mime) else {
            bail!("Invalid or malformed MIME message");
        };

        let quote = crypto::quote(&mime, MmlTemplateForwardBuilder::quote_options())?;

        let tpl = MmlTemplateForwardBuilder {
            signature,
            signature_style: account.forward_signature_style.unwrap_or_default(),
            posting_style: account.forward_posting_style.unwrap_or_default(),
            quote_headline: account.forward_quote_headline.unwrap_or_default(),
            from,
            from_name: account.from_name,
            headers: self.headers.raw,
            body: self.body,
        }
        .build(&msg, &quote)?;

        printer.out(tpl)
    }
}
