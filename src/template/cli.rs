use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::account::Account,
    template::{
        compose::cli::TemplateComposeCommand, forward::cli::TemplateForwardCommand,
        reply::cli::TemplateReplyCommand,
    },
};

/// Generate MML templates for composing new, reply and forward messages.
#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    #[clap(visible_alias = "new")]
    Compose(TemplateComposeCommand),
    Reply(TemplateReplyCommand),
    #[clap(visible_alias = "fwd")]
    Forward(TemplateForwardCommand),
}

impl TemplateCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Compose(cmd) => cmd.execute(printer, account),
            Self::Reply(cmd) => cmd.execute(printer, account),
            Self::Forward(cmd) => cmd.execute(printer, account),
        }
    }
}
