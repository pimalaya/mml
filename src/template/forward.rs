//! Forward template builder: see [`TemplateBuilderForward`] for
//! building a draft forward from a source MIME [`Message`].

use mail_builder::{
    MessageBuilder,
    headers::{address::Address, raw::Raw},
};
use mail_parser::Message;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    error::Result,
    interpreter::message::MimeInterpreterBuilder,
    template::types::{Template, TemplateBody, TemplateCursor},
};

/// Regex used to trim out prefix(es) from a subject.
///
/// Everything starting by "Fwd:" (case and whitespace insensitive) is
/// considered a prefix.
static SUBJECT: Lazy<Regex> = Lazy::new(|| Regex::new("(?i:\\s*fwd\\s*:\\s*)*(.*)").unwrap());

/// Trim out prefix(es) from the given subject.
fn trim_prefix(subject: &str) -> &str {
    match SUBJECT.captures(subject).and_then(|cap| cap.get(1)) {
        Some(subject) => subject.as_str(),
        None => subject,
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TemplateBuilderForward {
    pub signature: String,
    pub signature_style: TemplateForwardSignatureStyle,
    pub posting_style: TemplateForwardPostingStyle,
    pub quote_headline: String,
    pub from: String,
    pub from_name: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl TemplateBuilderForward {
    pub fn build(self, msg: &Message) -> Result<Template> {
        let interpreter = MimeInterpreterBuilder::new()
            // .with_save_attachments_dir(opts.downloads_dir)
            .with_show_only_headers(["From", "To", "In-Reply-To", "Cc", "Subject"]);

        let thread_interpreter = MimeInterpreterBuilder::new()
            .with_show_only_headers(["Date", "From", "To", "Cc", "Subject"])
            .with_save_attachments(true);

        let mut cursor = TemplateCursor::default();

        let mut builder = MessageBuilder::new();

        // From

        builder = builder.from((
            self.from_name.as_deref().unwrap_or_default(),
            self.from.as_str(),
        ));
        cursor.row += 1;

        // To

        builder = builder.to(Vec::<Address>::new());
        cursor.row += 1;

        // Subject

        // TODO: make this customizable?
        let prefix = String::from("Fwd: ");
        let subject = trim_prefix(msg.subject().unwrap_or_default());

        builder = builder.subject(prefix + subject);
        cursor.row += 1;

        // Additional headers

        for (key, val) in self.headers {
            builder = builder.header(key, Raw::new(val));
            cursor.row += 1;
        }

        // Body

        builder = builder.text_body({
            let mut body = TemplateBody::new(cursor);

            body.push_str(&self.body);
            body.flush();
            body.cursor.lock();

            if self.signature_style.is_inlined() && !self.signature.is_empty() {
                body.push_str(&self.signature);
                body.flush();
            }

            if self.posting_style.is_top() {
                body.push_str(&self.quote_headline);
                body.push_str(thread_interpreter.build().from_msg(msg)?.trim());
                body.flush()
            }

            cursor = body.cursor.clone();
            body
        });

        if self.signature_style.is_attached() && !self.signature.is_empty() {
            builder = builder.attachment("text/plain", "signature.txt", self.signature)
        }

        if self.posting_style.is_attached() {
            let file_name = msg.message_id().unwrap_or("message");
            builder = builder.attachment(
                "message/rfc822",
                format!("{file_name}.eml"),
                msg.raw_message(),
            )
        }

        let content = interpreter.build().from_msg_builder(builder)?;

        Ok(Template::new_with_cursor(content, cursor))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum, serde::Deserialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "kebab-case"))]
pub enum TemplateForwardPostingStyle {
    #[default]
    Top,
    Attached,
}

impl TemplateForwardPostingStyle {
    pub fn is_top(&self) -> bool {
        self == &Self::Top
    }

    pub fn is_attached(&self) -> bool {
        self == &Self::Attached
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum, serde::Deserialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "kebab-case"))]
pub enum TemplateForwardSignatureStyle {
    #[default]
    Inlined,
    Attached,
    Hidden,
}

impl TemplateForwardSignatureStyle {
    pub fn is_inlined(&self) -> bool {
        self == &Self::Inlined
    }

    pub fn is_attached(&self) -> bool {
        self == &Self::Attached
    }

    pub fn is_hidden(&self) -> bool {
        self == &Self::Hidden
    }
}

#[cfg(test)]
mod tests {
    use concat_with::concat_line;
    use mail_parser::MessageParser;

    use crate::template::{forward::TemplateBuilderForward, types::Template};

    #[test]
    fn default() {
        let msg = MessageParser::new()
            .parse(concat_line!(
                "Content-Type: text/plain",
                "From: sender@localhost",
                "To: me@localhost",
                "Subject: subject",
                "",
                "Hello, world!",
                "",
            ))
            .unwrap();

        assert_eq!(
            TemplateBuilderForward {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                quote_headline: "-------- Forwarded Message --------\n".into(),
                ..Default::default()
            }
            .build(&msg)
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: Fwd: subject",
                    "",
                    "", // cursor here
                    "",
                    "-------- Forwarded Message --------",
                    "From: sender@localhost",
                    "To: me@localhost",
                    "Subject: subject",
                    "",
                    "Hello, world!",
                ),
                (5, 0),
            ),
        );
    }

    #[test]
    fn with_signature() {
        let msg = MessageParser::new()
            .parse(concat_line!(
                "Content-Type: text/plain",
                "From: sender@localhost",
                "To: me@localhost",
                "Subject: subject",
                "",
                "Hello, world!",
                "",
            ))
            .unwrap();

        assert_eq!(
            TemplateBuilderForward {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                quote_headline: "-------- Forwarded Message --------\n".into(),
                ..Default::default()
            }
            .build(&msg)
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: Fwd: subject",
                    "",
                    "", // cursor here
                    "",
                    "-- ",
                    "signature",
                    "",
                    "-------- Forwarded Message --------",
                    "From: sender@localhost",
                    "To: me@localhost",
                    "Subject: subject",
                    "",
                    "Hello, world!",
                ),
                (5, 0),
            ),
        );
    }

    #[test]
    fn trim_subject_prefix() {
        assert_eq!(super::trim_prefix("Hello, world!"), "Hello, world!");
        assert_eq!(super::trim_prefix("fwd:Hello, world!"), "Hello, world!");
        assert_eq!(super::trim_prefix("Fwd   :Hello, world!"), "Hello, world!");
        assert_eq!(super::trim_prefix("fWd:   Hello, world!"), "Hello, world!");
        assert_eq!(
            super::trim_prefix("  FWD:  fwd  :Hello, world!"),
            "Hello, world!"
        );
    }
}
