//! # Template
//!
//! A template is a simplified version of an email MIME message, based
//! on [MML](https://www.gnu.org/software/emacs/manual/html_node/emacs-mime/Composing.html).

pub(crate) mod address;
#[cfg(feature = "cli")]
pub mod cli;
pub mod compose;
pub mod forward;
pub mod reply;
pub mod types;
