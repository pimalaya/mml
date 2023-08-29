//! Module related to man CLI.
//!
//! This module provides subcommands and a command matcher related to
//! man.

use std::path::PathBuf;

use clap::Parser;

/// Generate all man pages to the given directory
#[derive(Parser, Debug)]
pub struct GenerateManCommand {
    /// Directory in which to generate man files.
    ///
    /// Represents the directory in which all man files of all commands and subcommands should be
    /// generated.
    pub dir: PathBuf,
}
