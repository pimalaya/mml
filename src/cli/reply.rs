//! `mml reply`: editor-driven reply composer. Reads the source MIME
//! message on stdin (or from a path/inline value), builds a reply
//! template via [`crate::template::reply::TemplateBuilderReply`],
//! runs the [`crate::cli::utils::editor::edit_loop`], and emits the
//! compiled MIME bytes either to the optional output path or to
//! stdout. Source MIME after `--` is `MessageArg`'s
//! trailing-var-arg slot; the leading positional is the output
//! path.

use std::{
    fs,
    io::{Write, stdout},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use mail_parser::MessageParser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, MessageArg, resolve_signature},
        utils::editor::edit_loop,
    },
    template::reply::{
        TemplateBuilderReply, TemplateReplyPostingStyle, TemplateReplySignatureStyle,
    },
};

/// Reply to the given message interactively, using $EDITOR.
#[derive(Debug, Parser)]
pub struct ReplyCommand {
    /// Reply to every recipient of the source message (Cc included).
    /// CLI-only, no config equivalent.
    #[arg(long = "all", short = 'A')]
    pub reply_all: bool,

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
    /// How to attach the signature (`above-quote`, `below-quote`,
    /// `attached`, `hidden`).
    #[arg(long, short = 'S')]
    pub signature_style: Option<TemplateReplySignatureStyle>,

    /// Where to place the reply relative to the quoted thread
    /// (`top`, `bottom`, `interleaved`).
    #[arg(long, short = 'P')]
    pub posting_style: Option<TemplateReplyPostingStyle>,
    /// Line printed above the quoted thread. Supports `\n` to insert
    /// a newline.
    #[arg(long, short = 'Q')]
    pub quote_headline: Option<String>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,
    /// Pre-fill the body before opening the editor.
    #[arg(long, short, default_value_t)]
    pub body: String,

    /// Optional file path to write the compiled MIME message to.
    /// When omitted, the MIME bytes are written to stdout. See
    /// [`crate::cli::compose::ComposeCommand::output`] for the
    /// rationale.
    #[arg(value_name = "PATH")]
    pub output: Option<PathBuf>,

    #[command(flatten)]
    pub mime: MessageArg,
}

impl ReplyCommand {
    pub fn execute(self, _printer: &mut impl Printer, account: Account) -> Result<()> {
        let account = account
            .with_from(self.from)
            .with_from_name(self.from_name)
            .with_signature(resolve_signature(self.signature))
            .with_reply_signature_style(self.signature_style)
            .with_reply_posting_style(self.posting_style)
            .with_reply_quote_headline(self.quote_headline);

        let signature = account.signature();

        let Some(from) = account.from else {
            bail!("missing `From` email from both config and args");
        };

        let mime = self.mime.parse()?.into_bytes();

        let Some(msg) = MessageParser::new().parse(&mime) else {
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

        match edit_loop(tpl, self.output.is_some())? {
            Some(mime) => match self.output {
                Some(path) => fs::write(&path, &mime)
                    .with_context(|| format!("cannot write MIME to {}", path.display())),
                None => Ok(stdout().write_all(&mime)?),
            },
            None => bail!("aborted by user"),
        }
    }
}
