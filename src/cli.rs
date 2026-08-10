//! `mml` command-line interface: the clap tree and its dispatch.
//!
//! [`Cli`] is the root parser and [`Command`] the subcommand enum.
//! [`Command::execute`] merges the per-account DTO from the loaded
//! [`config::Config`] and routes each subcommand to its own `execute`
//! method. The subcommands live in sibling modules (compile, interpret,
//! compose, reply, forward, template), with the shared pieces under
//! [`account`], [`args`], [`config`] and [`utils`].

pub mod account;
pub mod args;
#[cfg(feature = "compiler")]
pub mod compile;
#[cfg(all(feature = "compiler", feature = "interpreter"))]
pub mod compose;
pub mod config;
#[cfg(all(feature = "compiler", feature = "interpreter"))]
pub mod forward;
#[cfg(feature = "interpreter")]
pub mod interpret;
#[cfg(all(feature = "compiler", feature = "interpreter"))]
pub mod mailto;
pub mod printer;
#[cfg(all(feature = "compiler", feature = "interpreter"))]
pub mod reply;
#[cfg(feature = "interpreter")]
pub mod template;
pub mod utils;

use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, JsonFlag, LogFlags},
        commands::{CompletionCommand, ManualCommand},
        parsers::path_parser,
    },
    long_version,
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

#[cfg(feature = "compiler")]
use crate::cli::compile::CompileCommand;
use crate::cli::{account::Account, args::OutputArg, config::Config};
#[cfg(all(feature = "compiler", feature = "interpreter"))]
use crate::cli::{
    compose::ComposeCommand, forward::ForwardCommand, mailto::MailtoCommand, reply::ReplyCommand,
};
#[cfg(feature = "interpreter")]
use crate::cli::{interpret::InterpretCommand, template::TemplateCommand};

/// Root CLI parser.
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    /// Override the default configuration file path.
    ///
    /// The given paths are shell-expanded then canonicalized (if
    /// applicable). Multiple paths can be provided by delimiting them
    /// with `:` (like `$PATH` in a POSIX shell); subsequent paths are
    /// merged with the first.
    #[arg(short, long = "config", global = true, env = "MML_CONFIG")]
    #[arg(value_name = "PATH", value_parser = path_parser, value_delimiter = ':')]
    pub config_paths: Vec<PathBuf>,
    #[command(flatten)]
    pub account: AccountFlag,
    #[command(flatten)]
    pub output: OutputArg,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

/// The `mml` subcommand set.
#[derive(Subcommand, Debug)]
pub enum Command {
    #[cfg(feature = "interpreter")]
    #[command(subcommand)]
    #[clap(visible_alias = "tpl")]
    Templates(TemplateCommand),
    #[cfg(feature = "compiler")]
    Compile(CompileCommand),
    #[cfg(feature = "interpreter")]
    #[command(visible_alias = "read")]
    Interpret(InterpretCommand),
    #[cfg(all(feature = "compiler", feature = "interpreter"))]
    Compose(ComposeCommand),
    #[cfg(all(feature = "compiler", feature = "interpreter"))]
    Reply(ReplyCommand),
    #[cfg(all(feature = "compiler", feature = "interpreter"))]
    Forward(ForwardCommand),
    #[cfg(all(feature = "compiler", feature = "interpreter"))]
    Mailto(MailtoCommand),
    Completions(CompletionCommand),
    Manuals(ManualCommand),
}

impl Command {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        output: &OutputArg,
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
            #[cfg(feature = "interpreter")]
            Self::Templates(cmd) => cmd.execute(printer, account()?),

            #[cfg(feature = "compiler")]
            Self::Compile(cmd) => cmd.execute(printer),
            #[cfg(feature = "interpreter")]
            Self::Interpret(cmd) => cmd.execute(printer, account()?),

            #[cfg(all(feature = "compiler", feature = "interpreter"))]
            Self::Compose(cmd) => cmd.execute(printer, output, account()?),
            #[cfg(all(feature = "compiler", feature = "interpreter"))]
            Self::Reply(cmd) => cmd.execute(printer, output, account()?),
            #[cfg(all(feature = "compiler", feature = "interpreter"))]
            Self::Forward(cmd) => cmd.execute(printer, output, account()?),
            #[cfg(all(feature = "compiler", feature = "interpreter"))]
            Self::Mailto(cmd) => cmd.execute(printer, output, account()?),

            Self::Completions(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manuals(cmd) => cmd.execute(printer, Cli::command()),
        }
    }
}
