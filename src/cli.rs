//! `mml` command-line interface: the clap tree and its dispatch.
//!
//! [`Cli`] is the root parser and [`Command`] the subcommand enum.
//! [`Command::execute`] merges the per-account DTO from the loaded
//! [`config::Config`] and routes each subcommand to its own `execute`
//! method. The subcommands live in sibling modules (compile, interpret,
//! compose, reply, forward, mailto, template), with the shared pieces
//! under [`account`], [`args`], [`config`], [`error`], [`json_schema`],
//! [`printer`] and [`utils`].
//!
//! The configuration is optional: a command finding none runs on the
//! defaults and the flags. MML writes no configuration. A bare `mml`
//! shows the help when one exists, and the welcome ([`welcome`])
//! otherwise, which copies config.sample.toml from the release or the
//! package, or downloads it.

pub mod account;
pub mod args;
pub mod compile;
pub mod compose;
pub mod config;
pub mod crypto;
pub mod error;
pub mod forward;
pub mod interpret;
pub mod json_schema;
pub mod mailto;
pub mod printer;
pub mod reply;
pub mod template;
pub mod utils;

use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use log::trace;
use pimalaya_cli::{
    clap::{
        args::{AccountFlag, ConfigFlags, JsonFlag, LogFlags},
        commands::{
            completion::CompletionCommand, json_schema::JsonSchemaCommand, manual::ManualCommand,
        },
    },
    footer, long_version,
    printer::Printer,
    welcome::Welcome,
};
use pimalaya_config::toml::{AccountError, ConfigMissingError, TomlConfig};

use crate::cli::compile::CompileCommand;
use crate::cli::{account::Account, args::OutputArg, config::Config};
use crate::cli::{
    compose::ComposeCommand, forward::ForwardCommand, mailto::MailtoCommand, reply::ReplyCommand,
};
use crate::cli::{interpret::InterpretCommand, template::TemplateCommand};

/// The sample configuration as a raw file, for `curl` to fetch.
const SAMPLE_RAW_URL: &str =
    "https://raw.githubusercontent.com/pimalaya/mml/master/config.sample.toml";

/// Root CLI parser.
#[derive(Parser, Debug)]
#[command(name = "mml")]
#[command(author, version, about, long_version = long_version!())]
#[command(after_help = footer!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    /// The command to run, bare `mml` showing the help.
    #[command(subcommand)]
    pub command: Option<Command>,
    /// The configuration file(s) to read the identities from.
    #[command(flatten)]
    pub config: ConfigFlags,
    /// The account the command applies to.
    #[command(flatten)]
    pub account: AccountFlag,
    /// Where the result lands.
    #[command(flatten)]
    pub output: OutputArg,
    /// Whether the output is rendered as JSON.
    #[command(flatten)]
    pub json: JsonFlag,
    /// How much the command logs, and where.
    #[command(flatten)]
    pub log: LogFlags,
}

/// The `mml` subcommand set.
#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    #[command(visible_alias = "tpl", alias = "templates")]
    Template(TemplateCommand),
    Compile(CompileCommand),
    #[command(visible_alias = "read")]
    Interpret(InterpretCommand),
    Compose(ComposeCommand),
    Reply(ReplyCommand),
    Forward(ForwardCommand),
    Mailto(MailtoCommand),
    #[command(alias = "completions")]
    Completion(CompletionCommand),
    #[command(alias = "manuals")]
    Manual(ManualCommand),
    #[command(alias = "json-schemas")]
    JsonSchema(JsonSchemaCommand),
}

impl Command {
    /// Merges the account config, then routes to the subcommand, a
    /// library failure carrying its code.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        output: &OutputArg,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        trace!("config paths: {config_paths:?}");
        trace!("account name: {account_name:?}");

        let account = || take_account(config_paths, account_name);

        let result = match self {
            Self::Template(cmd) => cmd.execute(printer, account()?),

            Self::Compile(cmd) => cmd.execute(printer),
            Self::Interpret(cmd) => cmd.execute(printer, account()?),

            Self::Compose(cmd) => cmd.execute(printer, output, account()?),
            Self::Reply(cmd) => cmd.execute(printer, output, account()?),
            Self::Forward(cmd) => cmd.execute(printer, output, account()?),
            Self::Mailto(cmd) => cmd.execute(printer, output, account()?),

            Self::Completion(cmd) => cmd.execute(printer, Cli::command()),
            Self::Manual(cmd) => cmd.execute(printer, Cli::command()),
            Self::JsonSchema(cmd) => cmd.execute(printer, json_schema::registry()),
        };

        result.map_err(error::coded)
    }
}

/// Meets a bare `mml`: the help when a configuration exists, the
/// missing configuration otherwise, which a person meets as the welcome.
pub fn meet_bare_invocation(config_paths: &[PathBuf]) -> Result<()> {
    // NOTE: a broken file counts as a configuration, its error left to
    // the first command reading it.
    if let Err(err) = Config::load(config_paths)
        && err.is::<ConfigMissingError>()
    {
        return Err(err);
    }

    Cli::command().print_help()?;
    Ok(())
}

/// Loads the configuration at `config_paths`, the default locations
/// when none is given: a missing default configuration is the empty
/// one, a missing `-c` path a [`ConfigMissingError`].
pub fn load_config(config_paths: &[PathBuf]) -> Result<Config> {
    if config_paths.is_empty() {
        Ok(Config::from_paths_or_default(config_paths)?.unwrap_or_default())
    } else {
        Config::load(config_paths)
    }
}

/// Builds the [`Account`] a command runs against: the global layer of
/// the configuration at `config_paths`, then the account `-a` names, or
/// the one marked default.
///
/// No account marked default is not a failure: the global layer alone
/// applies. An unknown name and several defaults are.
pub fn take_account(config_paths: &[PathBuf], account_name: Option<&str>) -> Result<Account> {
    let mut config = load_config(config_paths)?;

    let account_config = match config.take_account(account_name) {
        Ok((_, account_config)) => Some(account_config),
        Err(AccountError::NoDefault) => None,
        Err(err) => return Err(err.into()),
    };

    let mut account = Account::from(config);

    if let Some(account_config) = account_config {
        account = account.merge(Account::from(account_config));
    }

    Ok(account)
}

/// The welcome a person meets when no configuration exists at `path`:
/// the commands creating it from `sample`, the shipped sample
/// configuration, or downloading it without one, then `compose`.
pub fn welcome<'a>(
    path: &'a Path,
    config_paths: &'a [PathBuf],
    sample: Option<&'a Path>,
) -> Welcome<'a> {
    Welcome {
        bin: "mml",
        intro: "Welcome to MML, the CLI for the Emacs MIME Meta Language.",
        path,
        config_paths,
        sample,
        sample_url: SAMPLE_RAW_URL,
        next: "Edit it and set your identity, then compose a first message:",
        command: "compose",
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use clap::Parser;
    use pimalaya_config::toml::{AccountError, ConfigMissingError};
    use tempfile::TempDir;

    use crate::cli::{Cli, take_account};

    /// A directory holding a configuration file made of `toml`, and the
    /// path of that file.
    fn config(toml: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, toml).unwrap();
        (dir, path)
    }

    fn account(name: &str, default: bool) -> String {
        format!("[accounts.{name}]\ndefault = {default}\nfrom = \"{name}@example.org\"\n")
    }

    #[test]
    fn a_missing_config_path_is_typed_for_the_welcome() {
        let paths = [PathBuf::from("/nonexistent/mml/config.toml")];
        let err = take_account(&paths, None).unwrap_err();
        let missing = err.downcast_ref::<ConfigMissingError>().unwrap();
        assert_eq!(missing.path, paths[0]);
    }

    #[test]
    fn no_default_account_leaves_the_global_layer() {
        let toml = format!("from = \"global@example.org\"\n{}", account("a", false));
        let (_dir, path) = config(&toml);
        let account = take_account(&[path], None).unwrap();
        assert_eq!(account.from.as_deref(), Some("global@example.org"));
    }

    #[test]
    fn the_default_account_overrides_the_global_layer() {
        let toml = format!("from = \"global@example.org\"\n{}", account("a", true));
        let (_dir, path) = config(&toml);
        let account = take_account(&[path], None).unwrap();
        assert_eq!(account.from.as_deref(), Some("a@example.org"));
    }

    #[test]
    fn an_unknown_account_lists_the_held_ones() {
        let (_dir, path) = config(&(account("b", false) + &account("a", true)));
        let err = take_account(&[path], Some("c")).unwrap_err();
        let err = err.to_string();

        assert!(err.contains("Account `c` not found"), "{err}");
        assert!(err.contains("a, b"), "{err}");
    }

    #[test]
    fn several_default_accounts_are_refused_naming_them() {
        let (_dir, path) = config(&(account("b", true) + &account("a", true)));
        let err = take_account(&[path], None).unwrap_err();

        match err.downcast_ref::<AccountError>() {
            Some(AccountError::SeveralDefaults(names)) => assert_eq!(names, &["a", "b"]),
            _ => panic!("{err:?}"),
        }
    }

    #[test]
    fn an_unknown_key_is_refused() {
        let (_dir, path) = config(&(account("a", true) + "signature-style = \"inlined\"\n"));
        let err = take_account(&[path], None).unwrap_err();
        assert!(format!("{err:#}").contains("signature-style"), "{err:#}");
    }

    /// A `:` belongs to a path (`C:\` on Windows), so `-c` never splits
    /// on it.
    #[test]
    fn a_delimited_config_value_stays_one_path() {
        let cli = Cli::try_parse_from(["mml", "-c", "a.toml:b.toml"]).unwrap();
        assert_eq!(cli.config.paths, [PathBuf::from("a.toml:b.toml")]);
    }
}
