//! Post-edit choices for the `compose` / `reply` / `forward`
//! commands. The set is intentionally smaller than the v1.2.0
//! himalaya equivalent: send / save-as-draft live one level up,
//! inside himalaya's `*_with` flows.

use std::fmt;

use anyhow::Result;
use pimalaya_cli::prompt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PostEditChoice {
    Done,
    ViewMime,
    ViewTemplate,
    Edit,
    Abort,
}

impl fmt::Display for PostEditChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Done => write!(f, "Compile to MIME message"),
            Self::ViewMime => write!(f, "Preview MIME message"),
            Self::ViewTemplate => write!(f, "Preview MML template"),
            Self::Edit => write!(f, "Edit MML again"),
            Self::Abort => write!(f, "Abort"),
        }
    }
}

/// Show the post-edit prompt. `compile_ok` hides `Validate` and
/// `ViewMime` when the current buffer doesn't compile — the user
/// has to fix it first.
pub fn post_edit(compile_ok: bool) -> Result<PostEditChoice> {
    let choices = if compile_ok {
        vec![
            PostEditChoice::Done,
            PostEditChoice::ViewMime,
            PostEditChoice::ViewTemplate,
            PostEditChoice::Edit,
            PostEditChoice::Abort,
        ]
    } else {
        vec![
            PostEditChoice::Edit,
            PostEditChoice::ViewTemplate,
            PostEditChoice::Abort,
        ]
    };

    let choice = prompt::item("Pick an action:", choices, None)?;

    Ok(choice)
}
