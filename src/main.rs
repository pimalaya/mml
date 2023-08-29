mod compl;
mod man;
mod mml;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use env_logger::{Builder as LoggerBuilder, Env, DEFAULT_FILTER_ENV};

#[cfg(feature = "compiler")]
use crate::mml::args::CompileCommand;
#[cfg(feature = "interpreter")]
use crate::mml::args::InterpreterCommand;
use crate::{compl::args::GenerateCompletionCommand, man::args::GenerateManCommand};

#[derive(Parser, Debug)]
#[command(name= "mml", author, version, about, long_about = None, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Completion(GenerateCompletionCommand),
    Man(GenerateManCommand),
    #[cfg(feature = "compiler")]
    Compile(CompileCommand),
    #[cfg(feature = "interpreter")]
    Interpret(InterpreterCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    LoggerBuilder::new()
        .parse_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "warn"))
        .format_timestamp(None)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Completion(cmd) => compl::handlers::generate(Cli::command(), cmd.shell),
        Commands::Man(cmd) => man::handlers::generate(cmd.dir, Cli::command()),
        #[cfg(feature = "compiler")]
        Commands::Compile(cmd) => mml::handlers::compile(cmd.mml()).await,
        #[cfg(feature = "interpreter")]
        Commands::Interpret(cmd) => mml::handlers::interpret(cmd.mime()).await,
    }
}
