//! `mml` binary entry point.
//!
//! Parses the CLI, opens the printer sink named by `--output`, then
//! hands over to [`mml::cli::Command::execute`], or meets a bare
//! invocation. A person missing the configuration a bare `mml` or a
//! `-c` path looked for meets the welcome on stderr, a script or a JSON
//! caller the error.
//!
//! Exits 0 on success, 1 with the failure object, 2 on a command line
//! the grammar refuses, 3 when `interpret` met an invalid signature. The
//! library architecture lives in the src/lib.rs header.

use std::process;

use anyhow::Result;
use mml::cli::{self, Cli, error::ErrorCode, printer::MmlPrinter};
use pimalaya_cli::{
    clap::parse,
    error::ErrorReport,
    log::Logger,
    welcome::{is_wanted, shipped_sample},
};
use pimalaya_config::toml::ConfigMissingError;

fn main() {
    let cli: Cli = parse();

    // NOTE: the error report always goes to stdout, even when
    // `--output` redirects the artifact to a file: a failure belongs
    // on the stream the user is watching.
    let mut errors = MmlPrinter::stdout(&cli.json);
    let config_paths = cli.config.paths.clone();

    let result = execute(cli);

    if let Err(err) = &result
        && let Some(missing) = err.downcast_ref::<ConfigMissingError>()
        && is_wanted(&errors)
    {
        let sample = shipped_sample("mml");
        let welcome = cli::welcome(&missing.path, &config_paths, sample.as_deref());
        eprint!("{welcome}");
        process::exit(1);
    }

    ErrorReport::<ErrorCode>::eval_coded(&mut errors, result)
}

fn execute(cli: Cli) -> Result<()> {
    Logger::try_init(&cli.log)?;

    let config_paths = cli.config.paths.as_slice();
    let account_name = cli.account.name.as_deref();

    match cli.command {
        Some(command) => {
            let mut printer = MmlPrinter::new(&cli.json, &cli.output)?;
            command.execute(&mut printer, &cli.output, config_paths, account_name)
        }
        None => cli::meet_bare_invocation(config_paths),
    }
}
