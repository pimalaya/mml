//! # Template
//!
//! A template is a simplified version of an email MIME message, based
//! on [MML](https://www.gnu.org/software/emacs/manual/html_node/emacs-mime/Composing.html).
//!
//! Holds the types the compose, reply and forward builders share,
//! one submodule per builder next to them.

use std::{
    borrow::Cow,
    fmt,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "interpreter")]
pub(crate) mod address;
#[cfg(feature = "interpreter")]
pub mod compose;
#[cfg(feature = "interpreter")]
pub mod forward;
#[cfg(feature = "interpreter")]
pub mod reply;

/// A rendered template: MML content plus a cursor position.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct MmlTemplate {
    /// The MML markup.
    pub content: String,
    /// Where the caret belongs once loaded.
    pub cursor: MmlTemplateCursor,
}

impl MmlTemplate {
    /// Builds a template with the default cursor.
    pub fn new(content: impl ToString) -> Self {
        Self::new_with_cursor(content, MmlTemplateCursor::default())
    }

    /// Builds a template with an explicit cursor.
    pub fn new_with_cursor(content: impl ToString, cursor: impl Into<MmlTemplateCursor>) -> Self {
        let content = content.to_string();
        let cursor = cursor.into();
        Self { content, cursor }
    }

    /// Appends a section, unless the template is empty.
    pub fn append(&mut self, section: impl AsRef<str>) {
        if !self.content.is_empty() {
            self.content.push_str(section.as_ref())
        }
    }
}

impl Deref for MmlTemplate {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl DerefMut for MmlTemplate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl From<String> for MmlTemplate {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl fmt::Display for MmlTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// A caret position: one-based row, zero-based column.
///
/// NOTE: equality ignores the lock flag.
#[derive(Clone, Debug, Eq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct MmlTemplateCursor {
    /// One-based line number.
    pub row: usize,
    /// Zero-based column offset within the line.
    pub col: usize,
    #[cfg_attr(feature = "cli", serde(skip))]
    locked: bool,
}

impl MmlTemplateCursor {
    /// Builds an unlocked cursor.
    pub fn new(row: usize, col: usize) -> Self {
        Self {
            row,
            col,
            locked: false,
        }
    }

    /// Freezes the cursor against later flushes.
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Whether the cursor is frozen.
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl Default for MmlTemplateCursor {
    fn default() -> Self {
        Self {
            row: 1,
            col: 0,
            locked: false,
        }
    }
}

impl PartialEq for MmlTemplateCursor {
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row && self.col == other.col
    }
}

impl From<(usize, usize)> for MmlTemplateCursor {
    fn from((row, col): (usize, usize)) -> Self {
        MmlTemplateCursor::new(row, col)
    }
}

/// The body buffer a template builder writes into.
///
/// Sections are staged through the [`Deref`] handle, then committed
/// with [`MmlTemplateBody::flush`].
#[derive(Debug, Eq, PartialEq)]
pub struct MmlTemplateBody {
    content: String,
    buffer: String,
    /// Caret position across the flushed sections.
    pub cursor: MmlTemplateCursor,
    push_new_lines: bool,
}

impl MmlTemplateBody {
    /// Builds an empty body one row below the given cursor, leaving
    /// room for the blank line after the headers.
    pub fn new(mut cursor: MmlTemplateCursor) -> Self {
        cursor.row += 1;

        Self {
            content: Default::default(),
            buffer: Default::default(),
            cursor,
            push_new_lines: false,
        }
    }

    /// Commits the staged buffer, prefixing a blank line for every
    /// section after the first and advancing the cursor if unlocked.
    pub fn flush(&mut self) {
        let mut buffer = String::new();

        if self.push_new_lines {
            buffer.push_str("\n\n");
        } else {
            self.push_new_lines = true;
        };

        buffer.push_str(self.buffer.drain(..).as_str());

        if !self.cursor.is_locked() {
            match buffer.rsplit_once('\n') {
                Some((left, right)) => {
                    // NOTE: left.lines().count() does not distinguish
                    // "hello" from "hello\n" (returns 1)
                    let left_lines_count = left
                        .chars()
                        .fold(1, |count, c| count + if c == '\n' { 1 } else { 0 });

                    self.cursor.row += left_lines_count;
                    self.cursor.col = right.len();
                }
                None => {
                    self.cursor.col += buffer.len();
                }
            }
        }

        self.content.push_str(&buffer)
    }
}

impl Deref for MmlTemplateBody {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for MmlTemplateBody {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl From<MmlTemplateBody> for Cow<'_, str> {
    fn from(value: MmlTemplateBody) -> Self {
        value.content.into()
    }
}
