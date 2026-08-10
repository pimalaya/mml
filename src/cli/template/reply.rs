//! `mml template reply`: builds a reply MML template from the
//! source MIME message and the merged account + CLI args, then
//! prints it on stdout without opening the editor or compiling. Use
//! `mml reply` for the full edit and compile flow.

use anyhow::{Result, bail};
use clap::Parser;
use mail_parser::MessageParser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, IdentityArgs, InputArg},
    },
    template::reply::{
        MmlTemplateReplyBuilder, MmlTemplateReplyPostingStyle, MmlTemplateReplySignatureStyle,
    },
};

/// Build a new reply template, then print it on stdout.
#[derive(Debug, Parser)]
pub struct TemplateReplyCommand {
    #[command(flatten)]
    pub input: InputArg,

    /// Reply to every recipient of the source message (Cc included).
    /// CLI-only, no config equivalent.
    #[arg(long = "all", short = 'A')]
    pub reply_all: bool,

    #[command(flatten)]
    pub identity: IdentityArgs,
    /// How to attach the signature (`above-quote`, `below-quote`,
    /// `attached`, `hidden`).
    #[arg(long, short = 'S')]
    pub signature_style: Option<MmlTemplateReplySignatureStyle>,

    /// Where to place the reply relative to the quoted thread
    /// (`top`, `bottom`, `interleaved`).
    #[arg(long, short = 'P')]
    pub posting_style: Option<MmlTemplateReplyPostingStyle>,
    /// Line printed above the quoted thread. Supports `\n` to insert
    /// a newline.
    #[arg(long, short = 'Q')]
    pub quote_headline: Option<String>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,
    /// Pre-fill the body of the printed template.
    #[arg(short, long, default_value_t)]
    pub body: String,
}

impl TemplateReplyCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let account = self
            .identity
            .merge_into(account)
            .with_reply_signature_style(self.signature_style)
            .with_reply_posting_style(self.posting_style)
            .with_reply_quote_headline(self.quote_headline.map(|q| q.replace("\\n", "\n")));

        let signature = account.signature();

        let Some(from) = account.from else {
            bail!("Missing `From` email from both config and args");
        };

        let mime = self.input.read()?.into_bytes();

        let Some(msg) = MessageParser::new().parse(&mime) else {
            bail!("Invalid or malformed MIME message");
        };

        let tpl = MmlTemplateReplyBuilder {
            reply_all: self.reply_all,
            signature,
            signature_style: account.reply_signature_style.unwrap_or_default(),
            posting_style: account.reply_posting_style.unwrap_or_default(),
            quote_headline: account.reply_quote_headline.unwrap_or_default(),
            from,
            from_name: account.from_name,
            headers: self.headers.raw,
            body: self.body,
        }
        .build(&msg)?;

        printer.out(tpl)
    }
}
