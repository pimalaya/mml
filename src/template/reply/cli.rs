use std::fs;

use anyhow::{bail, Result};
use clap::Parser;
use log::debug;
use mail_parser::MessageParser;
use pimalaya_cli::{clap::parsers::path_parser, printer::Printer};

use crate::{
    cli::{
        account::Account,
        args::HeaderRawArgs,
        stdin::{format_stdin, format_str},
    },
    template::reply::builder::{
        TemplateBuilderReply, TemplateReplyPostingStyle, TemplateReplySignatureStyle,
    },
};

type MimeMessage = String;

/// Build a reply template from a source MIME message.
///
/// Identity and style fields default to the merged account (global
/// + `[accounts.<name>]`); CLI flags override them.
#[derive(Debug, Parser)]
pub struct TemplateReplyCommand {
    /// Source MIME message (file path or raw string). Defaults to
    /// stdin.
    #[arg(value_parser = parse_mime)]
    pub mime: Option<MimeMessage>,

    /// Include every recipient of the source message. CLI-only —
    /// not configurable.
    #[arg(long = "all", short = 'A')]
    pub reply_all: bool,

    #[arg(long, short)]
    pub from: Option<String>,

    #[arg(long, short = 'F')]
    pub from_name: Option<String>,

    #[arg(long, short)]
    pub signature: Option<String>,

    #[arg(long, short = 'S')]
    pub signature_style: Option<TemplateReplySignatureStyle>,

    #[arg(long, short = 'P')]
    pub posting_style: Option<TemplateReplyPostingStyle>,

    #[arg(long, short = 'Q')]
    pub quote_headline: Option<String>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,

    #[arg(short, long, default_value_t)]
    pub body: String,
}

impl TemplateReplyCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let account = account
            .with_from(self.from)
            .with_from_name(self.from_name)
            .with_signature(self.signature)
            .with_reply_signature_style(self.signature_style)
            .with_reply_posting_style(self.posting_style)
            .with_reply_quote_headline(self.quote_headline);

        let signature = account.signature();

        let Some(from) = account.from else {
            bail!("missing `From` email from both config and args");
        };

        let mime = self.mime.unwrap_or_else(format_stdin);
        let Some(msg) = MessageParser::new().parse(mime.as_bytes()) else {
            bail!("invalid or malformed MIME message");
        };

        let tpl = TemplateBuilderReply {
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

fn parse_mime(raw: &str) -> Result<MimeMessage, String> {
    match path_parser(raw) {
        Ok(path) => fs::read_to_string(path).map_err(|err| err.to_string()),
        Err(err) => {
            debug!("invalid path ({err}), processing arg as raw MIME message");
            Ok(format_str(raw))
        }
    }
}
