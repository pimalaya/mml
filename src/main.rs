use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::StdoutPrinter};
use pimalaya_config::toml::TomlConfig;

use mml::cli::{Cli, config::Config};

fn main() {
    let cli = Cli::parse();
    let mut printer = StdoutPrinter::new(&cli.json);

    let result = execute(cli, &mut printer);
    ErrorReport::eval(&mut printer, result);
}

fn execute(cli: Cli, printer: &mut StdoutPrinter) -> Result<()> {
    Logger::try_init(&cli.log)?;
    let account = cli.account.name.as_deref();
    let config = Config::from_paths_or_default(&cli.config_paths)?.unwrap_or_default();
    cli.command.execute(printer, config, account)
}
