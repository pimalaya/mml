//! Module related to man handling.
//!
//! This module gathers all man commands.  

use clap::Command;
use clap_mangen::Man;
use color_eyre::Result;
use std::{fs, path::PathBuf};

/// Generates all man pages of all subcommands in the given directory.
pub fn generate(dir: PathBuf, cmd: Command) -> Result<()> {
    let mut buffer = Vec::new();
    let cmd_name = cmd.get_name().to_string();
    let subcmds = cmd.get_subcommands().cloned().collect::<Vec<_>>();
    Man::new(cmd).render(&mut buffer)?;
    fs::write(&dir.join(format!("{}.1", cmd_name)), buffer)?;

    for subcmd in subcmds {
        let mut buffer = Vec::new();
        let subcmd_name = subcmd.get_name().to_string();
        Man::new(subcmd).render(&mut buffer)?;
        fs::write(&dir.join(format!("{}-{}.1", cmd_name, subcmd_name)), buffer)?;
    }

    Ok(())
}
