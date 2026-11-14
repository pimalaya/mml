//! `mml template` subcommand tree. Generates a draft MML template for
//! [`compose`], [`reply`] or [`forward`] without opening the editor:
//! stops one step before [`crate::cli::utils::editor::edit_loop`] and
//! prints the template as a [`TemplateOutput`].

pub mod compose;
pub mod forward;
pub mod reply;

use std::fmt;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    cli::{
        account::Account,
        template::{
            compose::TemplateComposeCommand, forward::TemplateForwardCommand,
            reply::TemplateReplyCommand,
        },
    },
    template::MmlTemplate,
};

/// Generate MML templates for composing new, reply and forward messages.
#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    #[clap(visible_alias = "new")]
    Compose(TemplateComposeCommand),
    Reply(TemplateReplyCommand),
    #[clap(visible_alias = "fwd")]
    Forward(TemplateForwardCommand),
}

impl TemplateCommand {
    /// Routes to the template builder.
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Compose(cmd) => cmd.execute(printer, account),
            Self::Reply(cmd) => cmd.execute(printer, account),
            Self::Forward(cmd) => cmd.execute(printer, account),
        }
    }
}

/// What a template command prints: the draft and where the caret
/// belongs. The text form is the draft alone.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TemplateOutput {
    /// The MML draft.
    pub content: String,
    /// Where the caret belongs once the draft is loaded.
    pub cursor: TemplateCursorOutput,
}

/// A caret position in a draft.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCursorOutput {
    /// The one-based line.
    pub row: usize,
    /// The zero-based column within the line.
    pub col: usize,
}

impl From<MmlTemplate> for TemplateOutput {
    fn from(tpl: MmlTemplate) -> Self {
        let cursor = TemplateCursorOutput {
            row: tpl.cursor.row,
            col: tpl.cursor.col,
        };

        Self {
            content: tpl.content,
            cursor,
        }
    }
}

impl fmt::Display for TemplateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}
