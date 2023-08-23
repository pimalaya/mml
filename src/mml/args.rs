#![cfg_attr(
    not(all(feature = "compiler", feature = "interpreter")),
    allow(dead_code, unused_imports)
)]

use anyhow::{anyhow, Result};
use atty::Stream;
use clap::{Arg, ArgMatches, Command};
use log::{debug, warn};
use std::{ffi::OsStr, path::PathBuf};
use tokio::{
    fs,
    io::{self, AsyncBufReadExt, BufReader},
};

const ARG_RAW: &str = "raw";
const ARG_PATH_OR_RAW: &str = "path-or-raw";
const CMD_COMPILE: &str = "compile";
const CMD_INTERPRET: &str = "interpret";

type MmlMessage = String;
type MimeMessage = String;

/// Represents the server commands.
#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    #[cfg(feature = "compiler")]
    Compile(MmlMessage),
    #[cfg(feature = "interpreter")]
    Interpret(MimeMessage),
}

/// Represents the server command matcher.
pub async fn matches(m: &ArgMatches) -> Result<Option<Cmd>> {
    #[cfg(feature = "compiler")]
    if let Some(ref m) = m.subcommand_matches(CMD_COMPILE) {
        debug!("compile MML message command matched");

        let mml = match parse_path_or_raw_arg(m) {
            Some(mml) => match shellexpand_full(mml) {
                Ok(path) => fs::read_to_string(PathBuf::from(path)).await?,
                Err(err) => {
                    warn!("{err}");
                    warn!("invalid path, processing it as raw MML message");
                    format_str(mml)
                }
            },
            None if atty::is(Stream::Stdin) => format_str(&parse_raw_arg(m)),
            None => format_stdin().await,
        };

        return Ok(Some(Cmd::Compile(mml)));
    }

    #[cfg(feature = "interpreter")]
    if let Some(ref m) = m.subcommand_matches(CMD_INTERPRET) {
        debug!("interpret MIME message command matched");

        let mime = match parse_path_or_raw_arg(m) {
            Some(mime) => match shellexpand_full(mime) {
                Ok(path) => fs::read_to_string(PathBuf::from(path)).await?,
                Err(err) => {
                    warn!("{err}");
                    warn!("invalid path, processing it as raw MIME message");
                    format_str(mime)
                }
            },
            None if atty::is(Stream::Stdin) => format_str(&parse_raw_arg(m)),
            None => format_stdin().await,
        };

        return Ok(Some(Cmd::Interpret(mime)));
    }

    Ok(None)
}

/// Represents the email raw argument.
pub fn path_or_raw_arg() -> Arg {
    Arg::new(ARG_PATH_OR_RAW)
        .help("Take data from the given file path or from the argument itself")
        .long_help(
            "Take data from the given file path or from the argument itself.

If the argument points to a valid file, its content is used.
Otherwise the argument is treated as raw data.",
        )
}

/// Represents the email raw argument parser.
pub fn parse_path_or_raw_arg(m: &ArgMatches) -> Option<&str> {
    m.get_one::<String>(ARG_PATH_OR_RAW).map(String::as_str)
}

/// Represents the email raw argument.
pub fn raw_arg() -> Arg {
    Arg::new(ARG_RAW)
        .help("Take data from the standard input or from the argument itself")
        .long_help(
            "Take data from the standard input or from the argument itself.

If the current terminal is considered interactive, take data from stdin.
Otherwise all arguments after -- are treated as raw data.",
        )
        .raw(true)
}

/// Represents the email raw argument parser.
pub fn parse_raw_arg(m: &ArgMatches) -> String {
    m.get_raw(ARG_RAW)
        .map(|arg| {
            arg.flat_map(OsStr::to_str)
                .fold(String::new(), |mut args, arg| {
                    if !args.is_empty() {
                        args.push(' ')
                    }
                    args.push_str(arg);
                    args
                })
        })
        .unwrap_or_default()
}

/// Represents the client subcommands.
pub fn subcmds() -> Vec<Command> {
    vec![
        #[cfg(feature = "compiler")]
        Command::new(CMD_COMPILE)
            .about("Compile the given MML message to a valid MIME message")
            .arg(path_or_raw_arg())
            .arg(raw_arg()),
        #[cfg(feature = "interpreter")]
        Command::new(CMD_INTERPRET)
            .about("Interpret the given MIME message as a MML message")
            .arg(path_or_raw_arg())
            .arg(raw_arg()),
    ]
}

fn shellexpand_full(path_or_content: &str) -> Result<PathBuf> {
    let path_str = shellexpand::full(path_or_content)?;
    let path_str = path_str.as_ref();
    let path = PathBuf::from(path_str);

    if path.is_file() {
        Ok(path)
    } else {
        Err(anyhow!("cannot read file at path {path_str:?}"))
    }
}

fn format_str(input: &str) -> String {
    let input = input.replace("\\r", "").replace("\\n", "\n");
    let mut lines = input.lines();
    let mut output = String::new();

    while let Some(ref line) = lines.next() {
        output.push_str(line);
        output.push('\r');
        output.push('\n');
    }

    output
}

async fn format_stdin() -> String {
    let mut lines = BufReader::new(io::stdin()).lines();
    let mut output = String::new();

    while let Ok(Some(ref line)) = lines.next_line().await {
        output.push_str(line);
        output.push('\r');
        output.push('\n');
    }

    output
}
