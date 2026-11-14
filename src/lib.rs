#![cfg_attr(docsrs, feature(doc_cfg))]

//! # mml
//!
//! A Rust implementation of the Emacs MIME Meta Language (MML): a
//! plain-text markup that compiles to a full MIME message, and its
//! inverse rendering back to readable text.
//!
//! ## Layout
//!
//! The crate is two feature-gated pipelines plus a template layer and
//! an optional CLI:
//!
//! - The compiler module (feature `compiler`) turns MML markup into a
//!   MIME message. Its parser submodule tokenizes and parses the
//!   `<#part>` / `<#multipart>` grammar, and its message submodule
//!   assembles the parts into MIME via mail-builder.
//! - The interpreter module (feature `interpreter`) walks a parsed
//!   MIME message (mail-parser) back into MML or plain text, applying
//!   the header and part filters.
//! - [`template`] builds compose, reply and forward drafts, the
//!   starting point both pipelines share.
//! - [`error`] is the crate-wide error type.
//! - The cli module (feature `cli`, which turns both pipelines on) is
//!   the `mml` binary: the clap tree, the optional TOML configuration
//!   and the first-run welcome, the editor-driven flow, the stable
//!   error codes and the JSON Schemas of what each command prints. It
//!   is not part of the library contract.
//!
//! Two private modules, grammar and header, hold helpers shared by the
//! compiler and interpreter.

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "compiler")]
pub mod compiler;
pub mod coroutine;
pub mod error;
#[cfg(feature = "interpreter")]
pub mod interpreter;
pub mod pgp;
pub mod template;

#[cfg(any(feature = "compiler", feature = "interpreter"))]
pub(crate) mod grammar;
#[cfg(any(feature = "compiler", feature = "interpreter"))]
pub(crate) mod header;
