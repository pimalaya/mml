//! # Message module
//!
//! A message is composed of a header and a [body].
//!
//! ## Compilation
//!
//! A MML message/body can be compiled into a MIME message/body using
//! the [MmlCompilerBuilder]/[MmlBodyCompiler] builders.
//!
//! ## Interpretation
//!
//! A MIME message/body can be interpreted as a MML message/body using
//! the [MimeInterpreterBuilder]/[MimeBodyInterpreter] builder.

pub mod body;
#[cfg(feature = "compiler")]
pub mod compiler;
pub(crate) mod header;
#[cfg(feature = "interpreter")]
pub mod interpreter;
