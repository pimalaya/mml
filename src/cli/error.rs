//! Stable codes of the failures a caller is expected to act on,
//! printed as `code` beside the message under `--json`.
//!
//! The message stays for humans and may be reworded at any time; the
//! code is the contract. A failure with no code prints `code: null`.
//! The mechanism lives in pimalaya_cli::error, only the vocabulary is
//! MML's. The library knows nothing of it: [`coded`] reads the
//! [`MmlError`] a failure carries and gives the chain its code.

use std::io::{IsTerminal, stderr};

use anyhow::Error;
use ariadne::{Color, Config, Label, Report, ReportKind, Source};
use chumsky::error::Rich;
use pimalaya_cli::error::{CodedError, code_of};
use schemars::JsonSchema;
use serde::Serialize;

use crate::error::MmlError;

/// The stable codes, each serialized in kebab-case.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// The MML source does not parse.
    MmlInvalid,
    /// The MIME source does not parse.
    MimeInvalid,
    /// No `From` address, neither in the configuration nor in the flags.
    FromMissing,
    /// A part asks for PGP and this build carries no backend.
    PgpUnavailable,
    /// The PGP backend failed to sign, encrypt or decrypt.
    PgpFailed,
    /// The person aborted the draft from the post-edit prompt.
    Aborted,
}

impl ErrorCode {
    /// The code of a library failure, if a caller can act on it.
    fn of(err: &MmlError) -> Option<Self> {
        match err {
            MmlError::ParseMmlError(..) => Some(Self::MmlInvalid),
            MmlError::ParseMmlEmptyBodyError | MmlError::ParseMmlEmptyBodyContentError => {
                Some(Self::MmlInvalid)
            }
            MmlError::ParseMimeMessageError | MmlError::ParseRawEmailError => {
                Some(Self::MimeInvalid)
            }
            MmlError::PgpUnavailableError => Some(Self::PgpUnavailable),
            #[cfg(feature = "gpg")]
            MmlError::PgpGpgError(_) | MmlError::PgpMissingRecipientsError => Some(Self::PgpFailed),
            _ => None,
        }
    }
}

/// The message of [`ErrorCode::FromMissing`].
pub const FROM_MISSING: &str =
    "Missing `From` address, set `from` in the configuration or pass --from";

/// The message of [`ErrorCode::MimeInvalid`] for a source that does not
/// parse at all.
pub const MIME_INVALID: &str = "Cannot parse the MIME message";

/// A failure carrying `code`, its message `message`.
pub fn error(code: ErrorCode, message: impl Into<String>) -> Error {
    CodedError::new(code, message).into()
}

/// The failure of an MML source that does not parse, coded
/// [`ErrorCode::MmlInvalid`], each parse error a source.
///
/// With `diagnose`, each error is first drawn on stderr against the
/// source, for a person to see where it sits.
pub fn mml_invalid(errs: &[Rich<'static, char>], source: &str, diagnose: bool) -> Error {
    if diagnose {
        let config = Config::default().with_color(stderr().is_terminal());

        for err in errs {
            let _ = Report::build(ReportKind::Error, err.span().into_range())
                .with_config(config)
                .with_message("cannot parse MML message")
                .with_label(
                    Label::new(err.span().into_range())
                        .with_message(err.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .eprint(Source::from(source));
        }
    }

    let mut reasons = errs.iter().rev().map(|err| {
        let span = err.span().into_range();
        format!("{} at {}..{}", err.reason(), span.start, span.end)
    });

    let Some(innermost) = reasons.next() else {
        return error(ErrorCode::MmlInvalid, "Cannot parse MML message");
    };

    let mut err = error(ErrorCode::MmlInvalid, innermost);

    for reason in reasons {
        err = err.context(reason);
    }

    err.context("Cannot parse MML message")
}

/// Gives a failure the code of the library cause it carries, its
/// messages unchanged, and leaves any other failure as it is.
///
/// The chain is rebuilt with a [`CodedError`] at its root, where the
/// shared report finds it, each message kept at its place.
pub fn coded(err: Error) -> Error {
    if code_of::<ErrorCode>(&err).is_some() {
        return err;
    }

    let code = err
        .chain()
        .find_map(|cause| cause.downcast_ref::<MmlError>())
        .and_then(ErrorCode::of);

    let Some(code) = code else {
        return err;
    };

    let mut messages: Vec<String> = err.chain().map(ToString::to_string).collect();
    let root = messages.pop().unwrap_or_default();
    let mut coded = error(code, root);

    for message in messages.into_iter().rev() {
        coded = coded.context(message);
    }

    coded
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, anyhow};
    use pimalaya_cli::error::code_of;

    use crate::{
        cli::error::{ErrorCode, coded},
        error::MmlError,
    };

    #[test]
    fn a_library_cause_is_coded_with_its_messages_kept() {
        let err = Err::<(), _>(MmlError::PgpUnavailableError)
            .context("cannot compile MML message")
            .unwrap_err();
        let before = format!("{err:#}");

        let err = coded(err);

        assert_eq!(code_of::<ErrorCode>(&err), Some(ErrorCode::PgpUnavailable));
        assert_eq!(format!("{err:#}"), before);
        assert_eq!(err.chain().count(), 2);
    }

    #[test]
    fn a_failure_with_no_library_cause_stays_uncoded() {
        let err = coded(anyhow!("Something else"));
        assert_eq!(code_of::<ErrorCode>(&err), None);
    }

    #[test]
    fn an_uncodable_library_cause_stays_uncoded() {
        let err = anyhow::Error::from(MmlError::ParseMessageError);
        assert_eq!(code_of::<ErrorCode>(&coded(err)), None);
    }
}
