//! Merged runtime account — the DTO between TOML config
//! deserialization and the commands that consume it.
//!
//! Built by [`crate::cli::mml::MmlCommand::execute`] in this order:
//!
//! 1. [`Account::default`].
//! 2. Fold the global [`crate::cli::config::Config`] via
//!    `Account::from(config)`.
//! 3. Fold the selected `[accounts.<name>]` entry via
//!    [`Account::merge`] with `Account::from(account_config)`.
//! 4. Fold CLI args via per-field `with_*` calls inside each
//!    command's `execute`.

use std::path::PathBuf;

use crate::{
    cli::config::{AccountConfig, Config},
    template::{
        compose::builder::TemplateComposeSignatureStyle,
        forward::builder::{TemplateForwardPostingStyle, TemplateForwardSignatureStyle},
        reply::builder::{TemplateReplyPostingStyle, TemplateReplySignatureStyle},
    },
};

/// Official RFC 3676 signature delimiter (dash-dash-space + LF),
/// used when neither config nor CLI overrides it.
pub const DEFAULT_SIGNATURE_DELIM: &str = "-- \n";

/// Flat, fully-merged identity + section defaults. Commands read
/// `account.<section>_<field>` directly; setting a field uses the
/// corresponding `with_*` method.
#[derive(Clone, Debug, Default)]
pub struct Account {
    pub from: Option<String>,
    pub from_name: Option<String>,

    /// Raw signature body. Compose with [`Account::signature`] to
    /// get the final delim-prefixed text.
    pub signature: Option<String>,
    pub signature_delim: Option<String>,

    pub compose_signature_style: Option<TemplateComposeSignatureStyle>,

    pub reply_signature_style: Option<TemplateReplySignatureStyle>,
    pub reply_posting_style: Option<TemplateReplyPostingStyle>,
    pub reply_quote_headline: Option<String>,

    pub forward_signature_style: Option<TemplateForwardSignatureStyle>,
    pub forward_posting_style: Option<TemplateForwardPostingStyle>,
    pub forward_quote_headline: Option<String>,

    pub read_include_headers: Option<Vec<String>>,
    pub read_exclude_headers: Option<Vec<String>>,
    pub read_include_parts: Option<Vec<String>>,
    pub read_exclude_parts: Option<Vec<String>>,
    pub read_show_multiparts: Option<bool>,
    pub read_save_attachments: Option<bool>,
    pub read_save_attachments_dir: Option<PathBuf>,
    pub read_hide_attachments: Option<bool>,
    pub read_hide_inline_attachments: Option<bool>,
    pub read_hide_plain_texts_signature: Option<bool>,
}

impl Account {
    /// Compose the final signature as `signature_delim + signature`.
    /// Returns an empty string when the body is unset or empty; the
    /// delim falls back to [`DEFAULT_SIGNATURE_DELIM`].
    pub fn signature(&self) -> String {
        let body = self.signature.as_deref().unwrap_or_default();

        if body.is_empty() {
            return String::new();
        }

        let delim = self
            .signature_delim
            .as_deref()
            .unwrap_or(DEFAULT_SIGNATURE_DELIM);

        format!("{delim}{body}")
    }

    pub fn with_from(mut self, v: Option<String>) -> Self {
        if v.is_some() {
            self.from = v;
        }
        self
    }

    pub fn with_from_name(mut self, v: Option<String>) -> Self {
        if v.is_some() {
            self.from_name = v;
        }
        self
    }

    pub fn with_signature(mut self, v: Option<String>) -> Self {
        if v.is_some() {
            self.signature = v;
        }
        self
    }

    pub fn with_signature_delim(mut self, v: Option<String>) -> Self {
        if v.is_some() {
            self.signature_delim = v;
        }
        self
    }

    pub fn with_compose_signature_style(
        mut self,
        v: Option<TemplateComposeSignatureStyle>,
    ) -> Self {
        if v.is_some() {
            self.compose_signature_style = v;
        }
        self
    }

    pub fn with_reply_signature_style(mut self, v: Option<TemplateReplySignatureStyle>) -> Self {
        if v.is_some() {
            self.reply_signature_style = v;
        }
        self
    }

    pub fn with_reply_posting_style(mut self, v: Option<TemplateReplyPostingStyle>) -> Self {
        if v.is_some() {
            self.reply_posting_style = v;
        }
        self
    }

    pub fn with_reply_quote_headline(mut self, v: Option<String>) -> Self {
        if v.is_some() {
            self.reply_quote_headline = v;
        }
        self
    }

    pub fn with_forward_signature_style(
        mut self,
        v: Option<TemplateForwardSignatureStyle>,
    ) -> Self {
        if v.is_some() {
            self.forward_signature_style = v;
        }
        self
    }

    pub fn with_forward_posting_style(mut self, v: Option<TemplateForwardPostingStyle>) -> Self {
        if v.is_some() {
            self.forward_posting_style = v;
        }
        self
    }

    pub fn with_forward_quote_headline(mut self, v: Option<String>) -> Self {
        if v.is_some() {
            self.forward_quote_headline = v;
        }
        self
    }

    pub fn with_read_include_headers(mut self, v: Option<Vec<String>>) -> Self {
        if v.is_some() {
            self.read_include_headers = v;
        }
        self
    }

    pub fn with_read_exclude_headers(mut self, v: Option<Vec<String>>) -> Self {
        if v.is_some() {
            self.read_exclude_headers = v;
        }
        self
    }

    pub fn with_read_include_parts(mut self, v: Option<Vec<String>>) -> Self {
        if v.is_some() {
            self.read_include_parts = v;
        }
        self
    }

    pub fn with_read_exclude_parts(mut self, v: Option<Vec<String>>) -> Self {
        if v.is_some() {
            self.read_exclude_parts = v;
        }
        self
    }

    pub fn with_read_show_multiparts(mut self, v: Option<bool>) -> Self {
        if v.is_some() {
            self.read_show_multiparts = v;
        }
        self
    }

    pub fn with_read_save_attachments(mut self, v: Option<bool>) -> Self {
        if v.is_some() {
            self.read_save_attachments = v;
        }
        self
    }

    pub fn with_read_save_attachments_dir(mut self, v: Option<PathBuf>) -> Self {
        if v.is_some() {
            self.read_save_attachments_dir = v;
        }
        self
    }

    pub fn with_read_hide_attachments(mut self, v: Option<bool>) -> Self {
        if v.is_some() {
            self.read_hide_attachments = v;
        }
        self
    }

    pub fn with_read_hide_inline_attachments(mut self, v: Option<bool>) -> Self {
        if v.is_some() {
            self.read_hide_inline_attachments = v;
        }
        self
    }

    pub fn with_read_hide_plain_texts_signature(mut self, v: Option<bool>) -> Self {
        if v.is_some() {
            self.read_hide_plain_texts_signature = v;
        }
        self
    }

    /// Fold `other`'s set fields on top of `self`. Each field is
    /// taken from `other` when `Some`, otherwise from `self`.
    pub fn merge(self, other: Self) -> Self {
        Self {
            from: other.from.or(self.from),
            from_name: other.from_name.or(self.from_name),

            signature: other.signature.or(self.signature),
            signature_delim: other.signature_delim.or(self.signature_delim),

            compose_signature_style: other
                .compose_signature_style
                .or(self.compose_signature_style),

            reply_signature_style: other.reply_signature_style.or(self.reply_signature_style),
            reply_posting_style: other.reply_posting_style.or(self.reply_posting_style),
            reply_quote_headline: other.reply_quote_headline.or(self.reply_quote_headline),

            forward_signature_style: other
                .forward_signature_style
                .or(self.forward_signature_style),
            forward_posting_style: other.forward_posting_style.or(self.forward_posting_style),
            forward_quote_headline: other.forward_quote_headline.or(self.forward_quote_headline),

            read_include_headers: other.read_include_headers.or(self.read_include_headers),
            read_exclude_headers: other.read_exclude_headers.or(self.read_exclude_headers),
            read_include_parts: other.read_include_parts.or(self.read_include_parts),
            read_exclude_parts: other.read_exclude_parts.or(self.read_exclude_parts),
            read_show_multiparts: other.read_show_multiparts.or(self.read_show_multiparts),
            read_save_attachments: other.read_save_attachments.or(self.read_save_attachments),
            read_save_attachments_dir: other
                .read_save_attachments_dir
                .or(self.read_save_attachments_dir),
            read_hide_attachments: other.read_hide_attachments.or(self.read_hide_attachments),
            read_hide_inline_attachments: other
                .read_hide_inline_attachments
                .or(self.read_hide_inline_attachments),
            read_hide_plain_texts_signature: other
                .read_hide_plain_texts_signature
                .or(self.read_hide_plain_texts_signature),
        }
    }
}

impl From<Config> for Account {
    fn from(config: Config) -> Self {
        let compose = config.compose.unwrap_or_default();
        let reply = config.reply.unwrap_or_default();
        let forward = config.forward.unwrap_or_default();
        let read = config.read.unwrap_or_default();

        Self {
            from: config.from,
            from_name: config.from_name,
            signature: config.signature,
            signature_delim: config.signature_delim,

            compose_signature_style: compose.signature_style,

            reply_signature_style: reply.signature_style,
            reply_posting_style: reply.posting_style,
            reply_quote_headline: reply.quote_headline,

            forward_signature_style: forward.signature_style,
            forward_posting_style: forward.posting_style,
            forward_quote_headline: forward.quote_headline,

            read_include_headers: read.include_headers,
            read_exclude_headers: read.exclude_headers,
            read_include_parts: read.include_parts,
            read_exclude_parts: read.exclude_parts,
            read_show_multiparts: read.show_multiparts,
            read_save_attachments: read.save_attachments,
            read_save_attachments_dir: read.save_attachments_dir,
            read_hide_attachments: read.hide_attachments,
            read_hide_inline_attachments: read.hide_inline_attachments,
            read_hide_plain_texts_signature: read.hide_plain_texts_signature,
        }
    }
}

impl From<AccountConfig> for Account {
    fn from(config: AccountConfig) -> Self {
        let compose = config.compose.unwrap_or_default();
        let reply = config.reply.unwrap_or_default();
        let forward = config.forward.unwrap_or_default();
        let read = config.read.unwrap_or_default();

        Self {
            from: config.from,
            from_name: config.from_name,
            signature: config.signature,
            signature_delim: config.signature_delim,

            compose_signature_style: compose.signature_style,

            reply_signature_style: reply.signature_style,
            reply_posting_style: reply.posting_style,
            reply_quote_headline: reply.quote_headline,

            forward_signature_style: forward.signature_style,
            forward_posting_style: forward.posting_style,
            forward_quote_headline: forward.quote_headline,

            read_include_headers: read.include_headers,
            read_exclude_headers: read.exclude_headers,
            read_include_parts: read.include_parts,
            read_exclude_parts: read.exclude_parts,
            read_show_multiparts: read.show_multiparts,
            read_save_attachments: read.save_attachments,
            read_save_attachments_dir: read.save_attachments_dir,
            read_hide_attachments: read.hide_attachments,
            read_hide_inline_attachments: read.hide_inline_attachments,
            read_hide_plain_texts_signature: read.hide_plain_texts_signature,
        }
    }
}
