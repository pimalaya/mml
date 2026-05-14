#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "compiler")]
pub mod compiler;
pub mod error;
#[cfg(feature = "interpreter")]
pub mod interpreter;
pub mod template;

pub(crate) mod grammar;
pub(crate) mod header;
