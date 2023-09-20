mod compl;
mod man;
mod mml;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use env_logger::{Builder as LoggerBuilder, Env, DEFAULT_FILTER_ENV};

#[derive(Parser, Debug)]
#[command(name= "mml", author, version, about, long_about = None, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Completion(compl::args::GenerateCompletionCommand),
    Man(man::args::GenerateManCommand),
    #[cfg(feature = "compiler")]
    Compile(mml::compiler::args::CompileCommand),
    #[cfg(feature = "interpreter")]
    Interpret(mml::interpreter::args::InterpretCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    LoggerBuilder::new()
        .parse_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "warn"))
        .format_timestamp(None)
        .init();

    match Cli::parse().command {
        Commands::Completion(cmd) => compl::handlers::generate(Cli::command(), cmd.shell),
        Commands::Man(cmd) => man::handlers::generate(cmd.dir, Cli::command()),
        #[cfg(feature = "compiler")]
        Commands::Compile(cmd) => mml::compiler::handlers::compile(cmd.mml()).await,
        #[cfg(feature = "interpreter")]
        Commands::Interpret(cmd) => {
            mml::interpreter::handlers::interpret(
                cmd.filter_headers(),
                cmd.filter_parts(),
                cmd.show_multiparts(),
                cmd.save_attachments(),
                cmd.save_attachments_dir(),
                cmd.show_attachments(),
                cmd.show_inline_attachments(),
                cmd.show_plain_texts_signature(),
                cmd.mime(),
            )
            .await
        }
    }
}
