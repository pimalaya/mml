//! I/O-free PGP seam.
//!
//! The compiler and the interpreter never call a PGP backend. They
//! suspend, emit a `WantsPgp*` yield, and resume once the caller hands
//! back a [`MmlPgpAnswer`]. Backends therefore live entirely outside
//! this crate's core: gpg today, anything else tomorrow, blocking or
//! not.
//!
//! A caller with no backend answers [`MmlPgpAnswer::Unavailable`].
//! Read paths degrade visibly on it; write paths refuse, since
//! silently sending a message that was meant to be encrypted is a
//! security failure rather than a degraded experience.

#[cfg(feature = "pgp-gpg")]
pub mod gpg;

/// The caller's answer to a `WantsPgp*` yield.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MmlPgpAnswer {
    /// The produced bytes, answering `Encrypt`, `Decrypt` or `Sign`.
    Bytes(Vec<u8>),
    /// The verdict of a `Verify`.
    Verdict(MmlPgpVerdict),
    /// No backend could serve the request.
    ///
    /// NOTE: distinct from a failure. It means the caller carries no
    /// PGP backend at all, which read paths report in place of the
    /// part rather than aborting.
    Unavailable,
}

/// The outcome of a signature check.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "kebab-case"))]
#[non_exhaustive]
pub enum MmlPgpVerdict {
    /// The signature is valid, made by the named key.
    Valid(String),
    /// The signature does not check out.
    Invalid,
}
