//! TOML configuration for the `mml` CLI.
//!
//! Loaded from the first valid path among:
//!
//! - `$XDG_CONFIG_HOME/mml/config.toml`
//! - `$HOME/.config/mml/config.toml`
//! - `$HOME/.mmlrc`
//!
//! Override with `-c, --config <PATH>` or `MML_CONFIG=<PATH>`.
//!
//! The editor used by `compose` / `reply` / `forward` is picked up
//! from `$VISUAL` then `$EDITOR` (handled by the `edit` crate); mml
//! intentionally does not duplicate that knob in this file.
//!
//! ## Merge order
//!
//! Each command consumes a single [`Account`][crate::cli::account::Account],
//! built by layering, in order:
//!
//! 1. [`Account::default`][crate::cli::account::Account::default].
//! 2. The global [`Config`] fields and sections (`from`,
//!    `from_name`, `signature`, `signature_delim`, `[compose]`,
//!    `[reply]`, `[forward]`, `[read]`).
//! 3. The selected `[accounts.<name>]` entry, which mirrors the
//!    same shape and may override anything from the global layer.
//! 4. The command-line arguments.
//!
//! Anything `Some(_)` at a layer wins over the previous; `None` is
//! transparent. Strings (`from`, `from_name`) are treated as set
//! when non-empty.

use std::{collections::HashMap, path::PathBuf};

use pimalaya_config::toml::TomlConfig;
use serde::Deserialize;

use crate::template::{
    compose::TemplateComposeSignatureStyle,
    forward::{TemplateForwardPostingStyle, TemplateForwardSignatureStyle},
    reply::{TemplateReplyPostingStyle, TemplateReplySignatureStyle},
};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    pub from: Option<String>,
    pub from_name: Option<String>,

    pub signature: Option<String>,
    pub signature_delim: Option<String>,

    pub compose: Option<ComposeConfig>,
    pub reply: Option<ReplyConfig>,
    pub forward: Option<ForwardConfig>,

    pub read: Option<ReadConfig>,

    #[serde(default)]
    pub accounts: HashMap<String, AccountConfig>,
}

impl TomlConfig for Config {
    type Account = AccountConfig;

    fn project_name() -> &'static str {
        "mml"
    }

    fn take_named_account(&mut self, name: &str) -> Option<(String, Self::Account)> {
        self.accounts.remove_entry(name)
    }

    fn take_default_account(&mut self) -> Option<(String, Self::Account)> {
        let name = self
            .accounts
            .iter()
            .find_map(|(name, account)| account.default.then(|| name.clone()))?;

        self.take_named_account(&name)
    }
}

/// A single identity, addressed by `[accounts.<name>]`. Same shape
/// as the global [`Config`] (minus `accounts`), plus the `default`
/// flag. All identity / section fields are optional; missing ones
/// fall back to the global config, then to the hard-coded defaults.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountConfig {
    /// Marks this account as the fallback when `-a/--account` is
    /// not passed. Exactly one account should set this.
    #[serde(default)]
    pub default: bool,

    pub from: Option<String>,
    pub from_name: Option<String>,

    pub signature: Option<String>,
    pub signature_delim: Option<String>,

    /// Override the global `[compose]` section for this account.
    pub compose: Option<ComposeConfig>,
    /// Override the global `[reply]` section for this account.
    pub reply: Option<ReplyConfig>,
    /// Override the global `[forward]` section for this account.
    pub forward: Option<ForwardConfig>,

    /// Override the global `[read]` section for this account.
    pub read: Option<ReadConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ComposeConfig {
    pub signature_style: Option<TemplateComposeSignatureStyle>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReplyConfig {
    pub signature_style: Option<TemplateReplySignatureStyle>,
    pub posting_style: Option<TemplateReplyPostingStyle>,
    pub quote_headline: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ForwardConfig {
    pub signature_style: Option<TemplateForwardSignatureStyle>,
    pub posting_style: Option<TemplateForwardPostingStyle>,
    pub quote_headline: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReadConfig {
    pub include_headers: Option<Vec<String>>,
    pub exclude_headers: Option<Vec<String>>,

    pub include_parts: Option<Vec<String>>,
    pub exclude_parts: Option<Vec<String>>,

    pub save_attachments: Option<bool>,
    pub save_attachments_dir: Option<PathBuf>,

    pub show_multiparts: Option<bool>,

    pub hide_attachments: Option<bool>,
    pub hide_inline_attachments: Option<bool>,
    pub hide_plain_texts_signature: Option<bool>,
}
