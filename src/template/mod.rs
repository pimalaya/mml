//! # Template
//!
//! A template is a simplified version of an email MIME message, based
//! on [MML](https://www.gnu.org/software/emacs/manual/html_node/emacs-mime/Composing.html).

pub(crate) mod address;
pub mod forward;
pub mod new;
pub mod reply;
mod template;

#[doc(inline)]
pub use template::*;
