//! Root CLI parser and dispatcher: defines the `mml` binary's clap
//! tree and routes each subcommand to its `execute` method after
//! merging the per-account DTO from the loaded
//! [`crate::cli::config::Config`].

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, ConfigPathsArg, JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
    },
    long_version,
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

#[cfg(feature = "interpreter")]
use crate::cli::read::ReadCommand;
#[cfg(feature = "compiler")]
use crate::cli::{compose::ComposeCommand, forward::ForwardCommand, reply::ReplyCommand};
#[cfg(feature = "compiler")]
use crate::compiler::cli::CompileCommand;
#[cfg(feature = "interpreter")]
use crate::interpreter::cli::InterpretCommand;
use crate::{
    cli::{account::Account, config::Config},
    template::cli::TemplateCommand,
};

/// Root CLI parser.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct MmlCli {
    #[command(subcommand)]
    pub command: MmlCommand,

    #[command(flatten)]
    pub config_paths: ConfigPathsArg,
    #[command(flatten)]
    pub account: AccountFlag,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

#[derive(Subcommand, Debug)]
pub enum MmlCommand {
    #[command(subcommand)]
    #[clap(visible_alias = "tpl")]
    Templates(TemplateCommand),

    #[cfg(feature = "compiler")]
    Compile(CompileCommand),
    #[cfg(feature = "interpreter")]
    Interpret(InterpretCommand),

    #[cfg(feature = "compiler")]
    Compose(ComposeCommand),
    #[cfg(feature = "compiler")]
    Reply(ReplyCommand),
    #[cfg(feature = "compiler")]
    Forward(ForwardCommand),
    #[cfg(feature = "interpreter")]
    Read(ReadCommand),

    Completions(CompletionCommand),
    Manuals(ManualCommand),
}

impl MmlCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        mut config: Config,
        account_name: Option<&str>,
    ) -> Result<()> {
        let account = move || -> Result<Account> {
            let account_config = config.take_account(account_name)?;

            let mut account = Account::from(config);

            if let Some((_, config)) = account_config {
                account = account.merge(Account::from(config));
            }

            Ok(account)
        };

        match self {
            Self::Templates(cmd) => cmd.execute(printer, account()?),

            #[cfg(feature = "compiler")]
            Self::Compile(cmd) => cmd.execute(printer),
            #[cfg(feature = "interpreter")]
            Self::Interpret(cmd) => cmd.execute(printer, account()?),

            #[cfg(feature = "compiler")]
            Self::Compose(cmd) => cmd.execute(printer, account()?),
            #[cfg(feature = "compiler")]
            Self::Reply(cmd) => cmd.execute(printer, account()?),
            #[cfg(feature = "compiler")]
            Self::Forward(cmd) => cmd.execute(printer, account()?),
            #[cfg(feature = "interpreter")]
            Self::Read(cmd) => cmd.execute(printer, account()?),

            Self::Completions(cmd) => cmd.execute(printer, MmlCli::command()),
            Self::Manuals(cmd) => cmd.execute(printer, MmlCli::command()),
        }
    }
}
