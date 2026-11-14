//! TOML configuration for the `mml` CLI.
//!
//! Loaded from every default path that exists, each a layer merged on
//! top of the ones before:
//!
//! - `$XDG_CONFIG_HOME/mml/config.toml`
//! - `$HOME/.config/mml/config.toml`
//! - `$HOME/.mmlrc`
//!
//! Override with `-c, --config <PATH>`, repeated to merge several files,
//! the first one the base. A missing configuration is the empty one,
//! unless `-c` names it. config.sample.toml is the annotated reference.
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

use crate::{
    compiler::body::MmlBodyFormat,
    template::{
        compose::MmlTemplateComposeSignatureStyle,
        forward::{MmlTemplateForwardPostingStyle, MmlTemplateForwardSignatureStyle},
        reply::{MmlTemplateReplyPostingStyle, MmlTemplateReplySignatureStyle},
    },
};

/// The whole `mml` configuration: a global layer, then named accounts.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    /// The `From` address of the drafts.
    pub from: Option<String>,
    /// The display name paired with the `From` address.
    pub from_name: Option<String>,

    /// The raw signature, its delimiter prepended at runtime.
    pub signature: Option<String>,
    /// The delimiter prepended to the signature, `-- \n` by default.
    pub signature_delim: Option<String>,

    /// The defaults of `compose` and `template compose`.
    pub compose: Option<ComposeConfig>,
    /// The defaults of `reply` and `template reply`.
    pub reply: Option<ReplyConfig>,
    /// The defaults of `forward` and `template forward`.
    pub forward: Option<ForwardConfig>,

    /// The defaults of `interpret`.
    pub read: Option<ReadConfig>,

    /// The accounts, by name.
    #[serde(default)]
    pub accounts: HashMap<String, AccountConfig>,
}

impl TomlConfig for Config {
    type Account = AccountConfig;

    fn project_name() -> &'static str {
        "mml"
    }

    fn sample_url() -> &'static str {
        concat!(
            env!("CARGO_PKG_REPOSITORY"),
            "/blob/master/config.sample.toml"
        )
    }

    fn accounts(&self) -> impl Iterator<Item = (&str, bool)> {
        let accounts = self.accounts.iter();
        accounts.map(|(name, account)| (name.as_str(), account.default))
    }

    fn take_named_account(&mut self, name: &str) -> Option<(String, Self::Account)> {
        self.accounts.remove_entry(name)
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
    /// not passed. At most one account sets it.
    #[serde(default)]
    pub default: bool,

    /// Override the global `from` for this account.
    pub from: Option<String>,
    /// Override the global `from-name` for this account.
    pub from_name: Option<String>,

    /// Override the global `signature` for this account.
    pub signature: Option<String>,
    /// Override the global `signature-delim` for this account.
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

/// The defaults of a new message draft.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ComposeConfig {
    /// Where the signature goes.
    pub signature_style: Option<MmlTemplateComposeSignatureStyle>,
    /// How plain text is sent.
    pub format: Option<MmlBodyFormat>,
    /// The column flowed lines break at.
    pub flowed_column: Option<usize>,
}

/// The defaults of a reply draft.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReplyConfig {
    /// Where the signature goes.
    pub signature_style: Option<MmlTemplateReplySignatureStyle>,
    /// Where the reply goes relative to the quote.
    pub posting_style: Option<MmlTemplateReplyPostingStyle>,
    /// The line above the quote.
    pub quote_headline: Option<String>,
    /// How plain text is sent.
    pub format: Option<MmlBodyFormat>,
    /// The column flowed lines break at.
    pub flowed_column: Option<usize>,
}

/// The defaults of a forward draft.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ForwardConfig {
    /// Where the signature goes.
    pub signature_style: Option<MmlTemplateForwardSignatureStyle>,
    /// Whether the forwarded message is quoted or attached.
    pub posting_style: Option<MmlTemplateForwardPostingStyle>,
    /// The line above the forwarded message.
    pub quote_headline: Option<String>,
    /// How plain text is sent.
    pub format: Option<MmlBodyFormat>,
    /// The column flowed lines break at.
    pub flowed_column: Option<usize>,
}

/// The defaults of `interpret`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReadConfig {
    /// The header names to keep, all others dropped.
    pub include_headers: Option<Vec<String>>,
    /// The header names to drop, ignored beside `include-headers`.
    pub exclude_headers: Option<Vec<String>>,

    /// The part types to render, all others dropped.
    pub include_parts: Option<Vec<String>>,
    /// The part types to skip, ignored beside `include-parts`.
    pub exclude_parts: Option<Vec<String>>,

    /// Whether attachments are saved while rendering.
    pub save_attachments: Option<bool>,
    /// Where attachments are saved, the system temporary directory by
    /// default.
    pub save_attachments_dir: Option<PathBuf>,

    /// Whether the multipart markers are rendered.
    pub show_multiparts: Option<bool>,

    /// Whether the attachment markers are left out.
    pub hide_attachments: Option<bool>,
    /// Whether the inline attachment markers are left out.
    pub hide_inline_attachments: Option<bool>,
    /// Whether the signature is stripped from plain text parts.
    pub hide_plain_texts_signature: Option<bool>,
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, slice};

    use pimalaya_config::{sample::variants, toml::TomlConfig};
    use tempfile::TempDir;

    use crate::cli::{account::Account, config::Config};

    /// Checks config.sample.toml: one `[accounts.example]` table and no
    /// other, read through the loader `-c` uses as written and with each
    /// commented line uncommented.
    #[test]
    fn the_sample_configuration_parses_with_each_variant() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
        let text = fs::read_to_string(&path).unwrap();
        let headers: Vec<_> = text.lines().filter(|l| l.starts_with('[')).collect();
        assert_eq!(headers, ["[accounts.example]"]);

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let variants = variants(&text, &[]);
        assert!(variants.len() > 1);

        for variant in variants {
            let name = variant.name;
            fs::write(&path, &variant.text).unwrap();

            let (_, account) = Config::load(slice::from_ref(&path))
                .and_then(|mut config| Ok(config.take_account(None)?))
                .unwrap_or_else(|err| panic!("config.sample.toml with {name}: {err:?}"));

            let account = Account::from(account);
            assert_eq!(account.from.as_deref(), Some("test@example.org"), "{name}");
        }
    }
}
