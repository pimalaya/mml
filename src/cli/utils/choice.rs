//! Post-edit choices for the `compose` / `reply` / `forward`
//! commands. The set is intentionally smaller than the v1.2.0
//! himalaya equivalent: send / save-as-draft live one level up,
//! inside himalaya's `*_with` flows.

use std::fmt;

use anyhow::Result;
use pimalaya_cli::prompt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PostEditChoice {
    SaveToFile,
    SaveToStdout,
    ViewMime,
    ViewTemplate,
    Edit,
    Abort,
}

impl fmt::Display for PostEditChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SaveToFile => write!(f, "Save MIME message to file"),
            Self::SaveToStdout => write!(f, "Save MIME message to stdout"),
            Self::ViewMime => write!(f, "Preview MIME message"),
            Self::ViewTemplate => write!(f, "Preview MML template"),
            Self::Edit => write!(f, "Edit MML again"),
            Self::Abort => write!(f, "Abort"),
        }
    }
}

/// Show the post-edit prompt. `compile_ok` hides the save and
/// `ViewMime` choices when the current buffer doesn't compile, so
/// the user has to fix it first. `has_output_path` toggles the save
/// label between "Save to file" (when an output path was passed on
/// the command line) and "Save to stdout".
pub fn post_edit(compile_ok: bool, has_output_path: bool) -> Result<PostEditChoice> {
    let save = if has_output_path {
        PostEditChoice::SaveToFile
    } else {
        PostEditChoice::SaveToStdout
    };

    let choices = if compile_ok {
        vec![
            save,
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
