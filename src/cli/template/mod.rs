//! `mml template` subcommand tree. Generates a draft MML template
//! for [`compose`], [`reply`] or [`forward`] without opening the
//! editor: stops one step before
//! [`crate::cli::utils::editor::edit_loop`] and prints the template
//! on stdout.

pub mod compose;
pub mod forward;
pub mod reply;
mod template;

pub use template::*;
