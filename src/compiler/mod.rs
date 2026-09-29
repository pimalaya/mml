//! MML → MIME compilation: the [`message`] entry point over the
//! [`body`] and [`parser`] layers.

pub mod body;
pub mod crypto;
mod flowed;
pub mod message;
pub(crate) mod parser;
