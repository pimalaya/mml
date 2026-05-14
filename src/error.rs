use std::{io, path::PathBuf, result};

use thiserror::Error;

/// The global `Error` enum of the library.
#[derive(Debug, Error)]
pub enum MmlError {
    #[cfg(feature = "compiler")]
    #[error("cannot parse MML body")]
    ParseMmlError(Vec<chumsky::error::Rich<'static, char>>, String),
    #[cfg(feature = "compiler")]
    #[error("cannot compile template")]
    WriteCompiledPartToVecError(#[source] io::Error),
    #[cfg(feature = "compiler")]
    #[error("cannot read attachment at {1:?}")]
    ReadAttachmentError(#[source] io::Error, PathBuf),

    #[error("cannot parse MIME message")]
    ParseMimeMessageError,
    #[error("cannot save attachment at {1}")]
    WriteAttachmentError(#[source] io::Error, PathBuf),
    #[error("cannot build email")]
    WriteMessageError(#[source] io::Error),

    #[error("cannot parse template")]
    ParseMessageError,
    #[error("cannot parse MML message: empty body")]
    ParseMmlEmptyBodyError,
    #[error("cannot parse MML message: empty body content")]
    ParseMmlEmptyBodyContentError,
    #[error("cannot compile MML message to vec")]
    CompileMmlMessageToVecError(#[source] io::Error),
    #[error("cannot compile MML message to string")]
    CompileMmlMessageToStringError(#[source] io::Error),

    #[error("cannot parse raw email")]
    ParseRawEmailError,
    #[error("cannot build email")]
    BuildEmailError(#[source] io::Error),
}

/// The global `Result` alias of the library.
pub type Result<T> = result::Result<T, MmlError>;
