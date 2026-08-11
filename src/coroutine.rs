//! I/O-free coroutine contract shared by the compiler and the
//! interpreter.
//!
//! Both are resumable state machines: they never call a PGP backend,
//! they yield what they need and continue once the caller resumes them
//! with a [`MmlPgpAnswer`]. Every exchange therefore ends on
//! [`MmlYield::Ok`] or [`MmlYield::Err`], with any number of
//! `WantsPgp*` steps in between.

use crate::error::MmlError;

/// One step of an I/O-free MML coroutine.
///
/// `T` is what the coroutine produces: MIME bytes for the compiler,
/// MML text for the interpreter.
#[derive(Debug)]
#[non_exhaustive]
pub enum MmlYield<T> {
    /// The coroutine is done; resuming it again is a logic error.
    Ok(T),
    /// The coroutine failed; resuming it again is a logic error.
    Err(MmlError),
    /// Encrypt the payload for the given recipients, then resume with
    /// [`MmlPgpAnswer::Bytes`].
    WantsPgpEncrypt {
        /// The cleartext to encrypt.
        plain: Vec<u8>,
        /// Recipient identities, as written in the MML directive.
        recipients: Vec<String>,
    },
    /// Decrypt an OpenPGP message, the second part of a
    /// `multipart/encrypted` (RFC 3156 section 4), then resume with
    /// [`MmlPgpAnswer::Bytes`].
    WantsPgpDecrypt(Vec<u8>),
    /// Sign the payload as the given identity, then resume with
    /// [`MmlPgpAnswer::Bytes`].
    WantsPgpSign {
        /// The cleartext to sign.
        plain: Vec<u8>,
        /// The identity to sign as.
        signer: String,
    },
    /// Verify a detached signature over the signed part, the shape of
    /// a `multipart/signed` (RFC 3156 section 5), then resume with
    /// [`MmlPgpAnswer::Verdict`].
    WantsPgpVerify {
        /// The exact bytes covered by the signature.
        signed: Vec<u8>,
        /// The detached `application/pgp-signature` payload.
        signature: Vec<u8>,
    },
}
