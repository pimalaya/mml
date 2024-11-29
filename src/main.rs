#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod compl;
mod man;
mod mml;

use clap::{CommandFactory, Parser, Subcommand};
use color_eyre::Result;
use pimalaya_tui::{terminal::cli::tracing, long_version};

#[derive(Parser, Debug)]
#[command(name = "mml", author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
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
    let tracing = tracing::install()?;

    let res = match Cli::parse().command {
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
    };

    tracing.with_debug_and_trace_notes(res)
}
