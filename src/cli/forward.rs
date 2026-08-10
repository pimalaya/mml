//! `mml forward`: editor-driven forward composer. Same shape as
//! [`crate::cli::reply`], built around
//! [`crate::template::forward::MmlTemplateForwardBuilder`].

use anyhow::{Result, bail};
use clap::Parser;
use mail_parser::MessageParser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, IdentityArgs, InputArg, OutputArg},
        utils::editor::edit_loop,
    },
    template::forward::{
        MmlTemplateForwardBuilder, MmlTemplateForwardPostingStyle, MmlTemplateForwardSignatureStyle,
    },
};

/// Forward the given message interactively, using $EDITOR.
#[derive(Debug, Parser)]
pub struct ForwardCommand {
    #[command(flatten)]
    pub identity: IdentityArgs,
    /// How to attach the signature (`inlined`, `attached`,
    /// `hidden`).
    #[arg(long, short = 'S')]
    pub signature_style: Option<MmlTemplateForwardSignatureStyle>,

    /// How to attach the forwarded message (`top`, `attached`).
    #[arg(long, short = 'P')]
    pub posting_style: Option<MmlTemplateForwardPostingStyle>,
    /// Line printed above the forwarded message. Supports `\n` to
    /// insert a newline.
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

impl ForwardCommand {
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
            .with_forward_signature_style(self.signature_style)
            .with_forward_posting_style(self.posting_style)
            .with_forward_quote_headline(self.quote_headline);

        let signature = account.signature();

        let Some(from) = account.from else {
            bail!("missing `From` email from both config and args");
        };

        let mime = self.input.read()?.into_bytes();

        let Some(msg) = MessageParser::new().parse(&mime) else {
            bail!("invalid or malformed MIME message on stdin");
        };

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
        .build(&msg)?;

        match edit_loop(tpl, output.path().is_some())? {
            Some(mime) => printer.out(String::from_utf8(mime)?),
            None => bail!("aborted by user"),
        }
    }
}
