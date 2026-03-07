#[cfg(feature = "interpreter")]
use std::path::PathBuf;
use std::{
    fs,
    io::{stdin, BufRead, BufReader},
};

#[cfg(feature = "interpreter")]
use anyhow::Context;
use anyhow::{bail, Result};
#[cfg(feature = "compiler")]
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::{CommandFactory, Parser, Subcommand};
use log::debug;
use mail_parser::MessageParser;
#[cfg(feature = "interpreter")]
use mml::message::{
    body::interpreter::FilterParts,
    interpreter::{FilterHeaders, MimeInterpreterBuilder},
};
use mml::template::{
    self,
    forward::{
        BuildForwardTemplateArgs, ForwardTemplatePostingStyle, ForwardTemplateSignatureStyle,
    },
    new::{BuildNewTemplateArgs, NewTemplateSignatureStyle},
    reply::{BuildReplyTemplateArgs, ReplyTemplatePostingStyle, ReplyTemplateSignatureStyle},
};
#[cfg(feature = "compiler")]
use mml::{error::Error, message::compiler::MmlCompilerBuilder};
#[cfg(feature = "interpreter")]
use pimalaya_toolbox::terminal::printer::Message;
use pimalaya_toolbox::{
    long_version,
    terminal::{
        clap::{
            args::{JsonFlag, LogFlags},
            commands::{CompletionCommand, ManualCommand},
            parsers::path_parser,
        },
        error::ErrorReport,
        log::Logger,
        printer::{Printer, StdoutPrinter},
    },
};

fn main() {
    let cli = MmlCli::parse();

    Logger::init(&cli.log);
    let mut printer = StdoutPrinter::new(&cli.json);

    let result = cli.command.execute(&mut printer);
    ErrorReport::eval(&mut printer, result)
}

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
struct MmlCli {
    #[command(subcommand)]
    pub command: MmlCommand,

    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

#[derive(Subcommand, Debug)]
enum MmlCommand {
    Completions(CompletionCommand),
    Manuals(ManualCommand),
    #[command(subcommand)]
    Template(TemplateCommand),
    #[cfg(feature = "compiler")]
    Compile {
        /// Read the message from the given file path.
        #[arg(value_parser = parse_mml)]
        mml: Option<MmlMessage>,
    },
    #[cfg(feature = "interpreter")]
    Interpret {
        /// Read the MIME message from the given file path.
        #[arg(value_parser = parse_mime)]
        mime: Option<MimeMessage>,

        /// Include header to the interpreted message.
        #[arg(
            long,
            value_name = "HEADER",
            value_delimiter = ',',
            conflicts_with = "exclude_header"
        )]
        include_header: Option<Vec<String>>,

        /// Exclude header from the interpreted message.
        #[arg(
            long,
            value_name = "HEADER",
            value_delimiter = ',',
            conflicts_with = "include_header"
        )]
        exclude_header: Option<Vec<String>>,

        /// Include parts to intepret by their MIME type.
        #[arg(
            long,
            value_name = "MIME",
            value_delimiter = ',',
            conflicts_with = "exclude_part"
        )]
        include_part: Option<Vec<String>>,

        /// Exclude parts to interpret by their MIME type.
        #[arg(
            long,
            value_name = "MIME",
            value_delimiter = ',',
            conflicts_with = "include_part"
        )]
        exclude_part: Option<Vec<String>>,

        /// Enable interpretation of multiparts.
        #[arg(long)]
        show_multiparts: bool,

        /// Save automatically attachments found in the original MIME
        /// message to the `save_attachments_dir` directory.
        #[arg(long)]
        save_attachments: bool,

        /// Define the directory attachments should point to.
        ///
        /// If `save_attachments` is true, attachments are automatically
        /// downloaded to this directory.
        #[arg(long, value_name = "DIR")]
        save_attachments_dir: Option<PathBuf>,

        /// If true, disable interpretation of all attachments.
        #[arg(long)]
        hide_attachments: bool,

        /// If true, disable interpretation of attachments with a content
        /// disposition set to inline.
        #[arg(long)]
        hide_inline_attachments: bool,

        /// If true, trim out signatures from text bodies.
        ///
        /// Only standard signatures can be trimmed out. Plain text found
        /// after a `-- \n` is considered a standard signature.
        #[arg(long)]
        hide_plain_texts_signature: bool,
    },
}

fn parse_mime(raw: &str) -> Result<MimeMessage, String> {
    match path_parser(raw) {
        Ok(path) => fs::read_to_string(path).map_err(|err| err.to_string()),
        Err(err) => {
            debug!("invalid path ({err}), processing arg as raw MIME message");
            Ok(format_str(&raw))
        }
    }
}

type MmlMessage = String;
type MimeMessage = String;

fn parse_mml(raw: &str) -> Result<MmlMessage, String> {
    match path_parser(raw) {
        Ok(path) => fs::read_to_string(path).map_err(|err| err.to_string()),
        Err(err) => {
            debug!("invalid path ({err}), processing arg as raw MML message");
            Ok(format_str(&raw))
        }
    }
}

/// The envelope id argument parser.
#[derive(Debug, Parser)]
pub struct HeaderRawArgs {
    /// Prefill the template with custom headers.
    ///
    /// A raw header should follow the pattern KEY:VAL.
    #[arg(long = "header", short = 'H', required = false)]
    #[arg(name = "header-raw", value_name = "KEY:VAL", value_parser = raw_header_parser)]
    pub raw: Vec<(String, String)>,
}

pub fn raw_header_parser(raw_header: &str) -> Result<(String, String), String> {
    if let Some((key, val)) = raw_header.split_once(':') {
        Ok((key.trim().to_owned(), val.trim().to_owned()))
    } else {
        Err(format!("cannot parse raw header {raw_header:?}"))
    }
}

#[derive(Debug, Subcommand)]
enum TemplateCommand {
    New {
        #[arg(short, long, default_value_t)]
        signature: String,
        #[arg(short = 'S', long, default_value_t)]
        signature_style: NewTemplateSignatureStyle,
        #[arg(short, long)]
        from: String,
        #[arg(short = 'F', long, default_value_t)]
        from_name: String,
        #[command(flatten)]
        headers: HeaderRawArgs,
        #[arg(short, long, default_value_t)]
        body: String,
    },
    Reply {
        #[arg(value_parser = parse_mime)]
        mime: Option<MimeMessage>,
        #[arg(short = 'a', long = "all", default_value_t)]
        reply_all: bool,
        #[arg(short, long, default_value_t)]
        signature: String,
        #[arg(short = 'S', long, default_value_t)]
        signature_style: ReplyTemplateSignatureStyle,
        #[arg(short = 'P', long, default_value_t)]
        posting_style: ReplyTemplatePostingStyle,
        #[arg(short = 'Q', long, default_value_t)]
        quote_headline: String,
        #[arg(short, long)]
        from: String,
        #[arg(short = 'F', long, default_value_t)]
        from_name: String,
        #[command(flatten)]
        headers: HeaderRawArgs,
        #[arg(short, long, default_value_t)]
        body: String,
    },
    Forward {
        #[arg(value_parser = parse_mime)]
        mime: Option<MimeMessage>,
        #[arg(short, long, default_value_t)]
        signature: String,
        #[arg(short = 'S', long, default_value_t)]
        signature_style: ForwardTemplateSignatureStyle,
        #[arg(short = 'P', long, default_value_t)]
        posting_style: ForwardTemplatePostingStyle,
        #[arg(short = 'Q', long, default_value_t)]
        quote_headline: String,
        #[arg(short, long)]
        from: String,
        #[arg(short = 'F', long, default_value_t)]
        from_name: String,
        #[command(flatten)]
        headers: HeaderRawArgs,
        #[arg(short, long, default_value_t)]
        body: String,
    },
}

impl MmlCommand {
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        match self {
            Self::Template(TemplateCommand::New {
                signature,
                signature_style,
                from,
                from_name,
                headers,
                body,
            }) => {
                let args = BuildNewTemplateArgs {
                    signature,
                    signature_style,
                    from,
                    from_name,
                    headers: headers.raw,
                    body,
                };

                printer.out(template::new::build(args)?)
            }
            Self::Template(TemplateCommand::Reply {
                mime,
                reply_all,
                signature,
                signature_style,
                posting_style,
                quote_headline,
                from,
                from_name,
                headers,
                body,
            }) => {
                let mime = mime.unwrap_or_else(format_stdin);
                let Some(mime) = MessageParser::new().parse(mime.as_bytes()) else {
                    bail!("Invalid or malformed MIME message");
                };

                let args = BuildReplyTemplateArgs {
                    reply_all,
                    signature,
                    signature_style,
                    posting_style,
                    quote_headline,
                    from,
                    from_name,
                    headers: headers.raw,
                    body,
                };

                printer.out(template::reply::build(&mime, args)?)
            }
            Self::Template(TemplateCommand::Forward {
                mime,
                signature,
                signature_style,
                posting_style,
                quote_headline,
                from,
                from_name,
                headers,
                body,
            }) => {
                let mime = mime.unwrap_or_else(format_stdin);

                let Some(mime) = MessageParser::new().parse(mime.as_bytes()) else {
                    bail!("Invalid or malformed MIME message");
                };

                let args = BuildForwardTemplateArgs {
                    signature,
                    signature_style,
                    posting_style,
                    quote_headline,
                    from,
                    from_name,
                    headers: headers.raw,
                    body,
                };

                printer.out(template::forward::build(&mime, args)?)
            }
            #[cfg(feature = "compiler")]
            Self::Compile { mml } => {
                let mml = mml.unwrap_or_else(format_stdin);

                let compiler = MmlCompilerBuilder::new()
                    .build(&mml)
                    .context("cannot build MML compiler")?;

                match compiler.compile() {
                    Err(Error::ParseMmlError(errs, body)) => {
                        errs.into_iter().for_each(|err| {
                            Report::build(ReportKind::Error, (), err.span().start)
                                .with_message("cannot parse MML message")
                                .with_label(
                                    Label::new(err.span().into_range())
                                        .with_message(err.reason().to_string())
                                        .with_color(Color::Red),
                                )
                                .finish()
                                .print(Source::from(&body))
                                .unwrap()
                        });
                        Ok(())
                    }
                    Err(err) => Err(err).context("cannot compile MML message"),
                    Ok(res) => {
                        let mime = res.into_string()?;
                        print!("{mime}");
                        Ok(())
                    }
                }
            }
            #[cfg(feature = "interpreter")]
            Self::Interpret {
                mime,
                include_header,
                exclude_header,
                include_part,
                exclude_part,
                show_multiparts,
                save_attachments,
                save_attachments_dir,
                hide_attachments,
                hide_inline_attachments,
                hide_plain_texts_signature,
            } => {
                let mime = mime.unwrap_or_else(format_stdin);

                let filter_headers = if let Some(mime_types) = &exclude_header {
                    FilterHeaders::Exclude(mime_types.clone())
                } else {
                    match &include_header {
                        Some(mime_types) => FilterHeaders::Include(mime_types.clone()),
                        None => FilterHeaders::All,
                    }
                };

                let filter_parts = if let Some(mime_types) = &exclude_part {
                    FilterParts::Exclude(mime_types.clone())
                } else {
                    match &include_part {
                        Some(mime_types) if mime_types.len() == 1 => {
                            FilterParts::Only(mime_types[0].clone())
                        }
                        Some(mime_types) => FilterParts::Include(mime_types.clone()),
                        None => FilterParts::All,
                    }
                };

                let show_attachments = !hide_attachments;
                let show_inline_attachments = !hide_inline_attachments;
                let show_plain_texts_signature = !hide_plain_texts_signature;

                let interpreter = MimeInterpreterBuilder::new()
                    .with_show_headers(filter_headers)
                    // TODO:
                    // .with_show_additional_headers(…);
                    .with_filter_parts(filter_parts)
                    .with_show_multiparts(show_multiparts)
                    .with_save_attachments(save_attachments)
                    .with_save_some_attachments_dir(save_attachments_dir)
                    .with_show_attachments(show_attachments)
                    .with_show_inline_attachments(show_inline_attachments)
                    .with_show_plain_texts_signature(show_plain_texts_signature)
                    .build();

                let mml = interpreter
                    .from_bytes(mime.as_bytes())
                    .context("cannot interpreter MIME message")?;

                printer.out(Message::new(mml))
            }
            Self::Completions(cmd) => cmd.execute(printer, MmlCli::command()),
            Self::Manuals(cmd) => cmd.execute(printer, MmlCli::command()),
        }
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

fn format_stdin() -> String {
    let mut lines = BufReader::new(stdin()).lines();
    let mut output = String::new();

    while let Some(Ok(ref line)) = lines.next() {
        output.push_str(line);
        output.push('\r');
        output.push('\n');
    }

    output
}
