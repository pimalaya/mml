//! `mml reply` — editor-driven reply composer. Reads the source MIME
//! message on stdin, builds a reply template via
//! [`crate::template::reply::TemplateBuilderReply`], runs
//! the [`crate::cli::editor::edit_loop`], and emits the compiled
//! MIME bytes on stdout.

use std::io::{stdout, Write};

use anyhow::{bail, Result};
use clap::Parser;
use mail_parser::MessageParser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{account::Account, args::HeaderRawArgs, editor::edit_loop, stdin::format_stdin},
    template::reply::{
        TemplateBuilderReply, TemplateReplyPostingStyle, TemplateReplySignatureStyle,
    },
};

/// Reply to a message: read source MIME on stdin, build the reply
/// template, drive `$EDITOR`, compile MML → MIME, write to stdout.
///
/// Plug into himalaya as `[message.composer.mml]` (with reply-with).
#[derive(Debug, Parser)]
pub struct ReplyCommand {
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

    #[arg(long, short, default_value_t)]
    pub body: String,
}

impl ReplyCommand {
    pub fn execute(self, _printer: &mut impl Printer, account: Account) -> Result<()> {
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

        let mime = format_stdin();
        let Some(msg) = MessageParser::new().parse(mime.as_bytes()) else {
            bail!("invalid or malformed MIME message on stdin");
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

        match edit_loop(tpl)? {
            Some(mime) => {
                stdout().write_all(&mime)?;
                Ok(())
            }
            None => bail!("aborted by user"),
        }
    }
}
