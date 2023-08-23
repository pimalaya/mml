mod compl;
mod man;
mod mml;

use anyhow::Result;
use clap::Command;
use env_logger::{Builder as LoggerBuilder, Env, DEFAULT_FILTER_ENV};
use std::env;

fn create_app() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .propagate_version(true)
        .infer_subcommands(true)
        .arg_required_else_help(true)
        .subcommand(compl::args::subcmd())
        .subcommand(man::args::subcmd())
        .subcommands(mml::args::subcmds())
}

#[tokio::main]
async fn main() -> Result<()> {
    LoggerBuilder::new()
        .parse_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "warn"))
        .format_timestamp(None)
        .init();

    let app = create_app();
    let m = app.get_matches();

    if let Some(compl::args::Cmd::Generate(shell)) = compl::args::matches(&m)? {
        return compl::handlers::generate(create_app(), shell);
    }

    if let Some(man::args::Cmd::GenerateAll(dir)) = man::args::matches(&m)? {
        return man::handlers::generate(dir, create_app());
    }

    // finally check mml commands
    match mml::args::matches(&m).await? {
        #[cfg(feature = "compiler")]
        Some(mml::args::Cmd::Compile(mml)) => {
            return mml::handlers::compile(mml).await;
        }
        #[cfg(feature = "interpreter")]
        Some(mml::args::Cmd::Interpret(mime)) => {
            return mml::handlers::interpret(mime).await;
        }
        _ => (),
    }

    Ok(())
}
