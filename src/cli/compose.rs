//! `mml compose`: editor-driven new-message composer. Builds a draft
//! template from the merged account and CLI args, opens
//! [`crate::cli::utils::editor::edit_loop`] on it, then prints the
//! compiled MIME. Refuses to start when stdout is redirected, see
//! [`crate::cli::args::OutputArg::check_editor_can_run`].

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, IdentityArgs, OutputArg},
        error::{ErrorCode, FROM_MISSING, error},
        printer::rfc822,
        utils::editor::{compile_options, edit_loop},
    },
    template::compose::{MmlTemplateComposeBuilder, MmlTemplateComposeSignatureStyle},
};

/// Compose a new message interactively, using $EDITOR.
#[derive(Debug, Parser)]
pub struct ComposeCommand {
    #[command(flatten)]
    pub identity: IdentityArgs,
    /// How to attach the signature (`inlined`, `attached`,
    /// `hidden`).
    #[arg(long, short = 'S')]
    pub signature_style: Option<MmlTemplateComposeSignatureStyle>,

    #[command(flatten)]
    pub headers: HeaderRawArgs,
    /// Pre-fill the body before opening the editor.
    #[arg(long, short, default_value_t)]
    pub body: String,
}

impl ComposeCommand {
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
            .with_compose_signature_style(self.signature_style);

        let signature = account.signature();

        let Some(from) = account.from else {
            return Err(error(ErrorCode::FromMissing, FROM_MISSING));
        };

        let tpl = MmlTemplateComposeBuilder {
            signature,
            signature_style: account.compose_signature_style.unwrap_or_default(),
            from,
            from_name: account.from_name,
            headers: self.headers.raw,
            body: self.body,
        }
        .build()?;

        let opts = compile_options(account.compose_format, account.compose_flowed_column);

        match edit_loop(tpl, opts, output.path().is_some())? {
            Some(mime) => printer.out_bytes(rfc822(mime)),
            None => Err(error(ErrorCode::Aborted, "Aborted by user")),
        }
    }
}
