//! # Template
//!
//! A template is a simplified version of an email MIME message, based
//! on [MML](https://www.gnu.org/software/emacs/manual/html_node/emacs-mime/Composing.html).

#[cfg(feature = "interpreter")]
pub(crate) mod address;
#[cfg(feature = "interpreter")]
pub mod compose;
#[cfg(feature = "interpreter")]
pub mod forward;
#[cfg(feature = "interpreter")]
pub mod reply;
pub mod types;
