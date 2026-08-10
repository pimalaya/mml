//! `mml` binary entry point.
//!
//! Parses the CLI, opens the printer sink named by `--output`, loads
//! the TOML config, then hands over to [`mml::cli::Command::execute`].
//! The library architecture lives in the src/lib.rs header.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{error::ErrorReport, log::Logger};
use pimalaya_config::toml::TomlConfig;

use mml::cli::{Cli, config::Config, printer::MmlPrinter};

fn main() {
    let cli = Cli::parse();

    // NOTE: the error report always goes to stdout, even when
    // `--output` redirects the artifact to a file: a failure belongs
    // on the stream the user is watching.
    let mut errors = MmlPrinter::stdout(&cli.json);

    let printer = MmlPrinter::new(&cli.json, &cli.output);
    let mut printer = ErrorReport::eval(&mut errors, printer);

    let result = execute(cli, &mut printer);
    ErrorReport::eval(&mut errors, result);
}

fn execute(cli: Cli, printer: &mut MmlPrinter) -> Result<()> {
    Logger::try_init(&cli.log)?;
    let account = cli.account.name.as_deref();
    let config = Config::from_paths_or_default(&cli.config_paths)?.unwrap_or_default();
    cli.command.execute(printer, &cli.output, config, account)
}
