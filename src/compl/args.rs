//! Module related to completion CLI.
//!
//! This module provides subcommands and a command matcher related to completion.

use clap::Parser;
use clap_complete::Shell;

/// Generates the completion script for the given shell.
#[derive(Parser, Debug)]
pub struct GenerateCompletionCommand {
    pub shell: Shell,
}
