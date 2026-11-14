//! `mml reply`: editor-driven reply composer. Reads the source MIME,
//! builds a template via
//! [`crate::template::reply::MmlTemplateReplyBuilder`], runs the
//! [`crate::cli::utils::editor::edit_loop`], prints the compiled MIME.

use anyhow::Result;
use clap::Parser;
use mail_parser::MessageParser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, IdentityArgs, InputArg, OutputArg},
        crypto,
        error::{ErrorCode, FROM_MISSING, MIME_INVALID, error},
        printer::rfc822,
        utils::editor::{compile_options, edit_loop},
    },
    template::reply::{
        MmlTemplateReplyBuilder, MmlTemplateReplyPostingStyle, MmlTemplateReplySignatureStyle,
    },
};

/// Reply to the given message interactively, using $EDITOR.
#[derive(Debug, Parser)]
pub struct ReplyCommand {
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
    /// Pre-fill the body before opening the editor.
    #[arg(long, short, default_value_t)]
    pub body: String,

    #[command(flatten)]
    pub input: InputArg,
}

impl ReplyCommand {
    /// Opens the editor on the draft, then prints the compiled MIME
    /// message.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        output: &OutputArg,
        account: Account,
    ) -> Result<()> {
        output.check_editor_can_run()?;

        let account = self
            .identity
            .merge_into(account)
            .with_reply_signature_style(self.signature_style)
            .with_reply_posting_style(self.posting_style)
            .with_reply_quote_headline(self.quote_headline.map(|q| q.replace("\\n", "\n")));

        let signature = account.signature();

        let Some(from) = account.from else {
            return Err(error(ErrorCode::FromMissing, FROM_MISSING));
        };

        let mime = self.input.read()?.into_bytes();

        let Some(msg) = MessageParser::new().parse(&mime) else {
            return Err(error(ErrorCode::MimeInvalid, MIME_INVALID));
        };

        let quote = crypto::quote(&mime, MmlTemplateReplyBuilder::quote_options())?;

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
        .build(&msg, &quote)?;

        let opts = compile_options(account.reply_format, account.reply_flowed_column);

        match edit_loop(tpl, opts, output.path().is_some())? {
            Some(mime) => printer.out_bytes(rfc822(mime)),
            None => Err(error(ErrorCode::Aborted, "Aborted by user")),
        }
    }
}
