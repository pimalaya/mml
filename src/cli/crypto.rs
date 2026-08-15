//! Drives the library coroutines, serving PGP with the backend this
//! build carries.
//!
//! The coroutines never know which backend answers them. This module
//! is the only place that does, and it is gated once, here.

use std::borrow::Cow;

use crate::{
    compiler::message::{MmlCompile, MmlCompileOptions},
    coroutine::MmlYield,
    error::{MmlError, Result},
    interpreter::{layer::MmlInterpret, message::MmlInterpretOptions},
    pgp::{MmlPgpAnswer, MmlPgpVerdict},
};

/// Compiles MML into MIME, serving every PGP step.
pub fn compile(mml: &str, opts: MmlCompileOptions) -> Result<Vec<u8>> {
    let mut coroutine = MmlCompile::new(mml, opts);
    let mut answer = None;

    loop {
        let yielded = coroutine.resume(answer.take());

        match serve(&yielded)? {
            Some(served) => answer = Some(served),
            None => return finish(yielded),
        }
    }
}

/// Interprets MIME into MML, serving every PGP step, reporting the
/// signature verdicts alongside the rendered text.
pub fn interpret(mime: &[u8], opts: MmlInterpretOptions) -> Result<(String, Vec<MmlPgpVerdict>)> {
    let mut coroutine = MmlInterpret::new(Cow::Borrowed(mime), opts);
    let mut answer = None;

    loop {
        let yielded = coroutine.resume(answer.take());

        match serve(&yielded)? {
            Some(served) => answer = Some(served),
            None => return Ok((finish(yielded)?, coroutine.verdicts().to_vec())),
        }
    }
}

/// Unwraps a terminal yield.
fn finish<T>(yielded: MmlYield<T>) -> Result<T> {
    match yielded {
        MmlYield::Ok(output) => Ok(output),
        MmlYield::Err(err) => Err(err),
        _ => Err(MmlError::PgpUnavailableError),
    }
}

/// Serves one step through GPG, `None` on a terminal yield.
#[cfg(feature = "gpg")]
fn serve<T>(yielded: &MmlYield<T>) -> Result<Option<MmlPgpAnswer>> {
    crate::pgp::gpg::MmlPgpGpg::default()
        .serve(yielded)
        .transpose()
}

/// Reports every step unavailable, `None` on a terminal yield.
///
/// NOTE: read paths render the unresolved part, write paths abort.
/// The library decides which, not this build.
#[cfg(not(feature = "gpg"))]
fn serve<T>(yielded: &MmlYield<T>) -> Result<Option<MmlPgpAnswer>> {
    Ok(match yielded {
        MmlYield::Ok(_) | MmlYield::Err(_) => None,
        _ => Some(MmlPgpAnswer::Unavailable),
    })
}

/// Renders the quote a reply or forward draft wraps, serving every
/// PGP step so an encrypted thread is decrypted before quoting.
pub fn quote(mime: &[u8], opts: MmlInterpretOptions) -> Result<String> {
    interpret(mime, opts).map(|(quote, _)| quote)
}
