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
    template::forward::{
        TemplateBuilderForward, TemplateForwardPostingStyle, TemplateForwardSignatureStyle,
    },
};

type MimeMessage = String;

/// Build a forward template from a source MIME message.
///
/// Identity and style fields default to the merged account (global
/// + `[accounts.<name>]`); CLI flags override them.
#[derive(Debug, Parser)]
pub struct TemplateForwardCommand {
    /// Source MIME message (file path or raw string). Defaults to
    /// stdin.
    #[arg(value_parser = parse_mime)]
    pub mime: Option<MimeMessage>,

    #[arg(short, long)]
    pub from: Option<String>,

    #[arg(short = 'F', long)]
    pub from_name: Option<String>,

    #[arg(short, long)]
    pub signature: Option<String>,

    #[arg(short = 'S', long)]
    pub signature_style: Option<TemplateForwardSignatureStyle>,

    #[arg(short = 'P', long)]
    pub posting_style: Option<TemplateForwardPostingStyle>,

    #[arg(short = 'Q', long)]
    pub quote_headline: Option<String>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,

    #[arg(short, long, default_value_t)]
    pub body: String,
}

impl TemplateForwardCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
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

        let mime = self.mime.unwrap_or_else(format_stdin);
        let Some(msg) = MessageParser::new().parse(mime.as_bytes()) else {
            bail!("invalid or malformed MIME message");
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

fn parse_mime(raw: &str) -> Result<MimeMessage, String> {
    match path_parser(raw) {
        Ok(path) => fs::read_to_string(path).map_err(|err| err.to_string()),
        Err(err) => {
            debug!("invalid path ({err}), processing arg as raw MIME message");
            Ok(format_str(raw))
        }
    }
}
