//! Compose template builder: see [`TemplateBuilderCompose`] for
//! building a draft template for a brand-new message (no source).

use mail_builder::{
    MessageBuilder,
    headers::{address::Address, raw::Raw},
};

use crate::{
    error::Result,
    interpreter::message::MimeInterpreterBuilder,
    template::types::{Template, TemplateBody, TemplateCursor},
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TemplateBuilderCompose {
    pub signature: String,
    pub signature_style: TemplateComposeSignatureStyle,
    pub from: String,
    pub from_name: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl TemplateBuilderCompose {
    pub fn build(self) -> Result<Template> {
        let interpreter = MimeInterpreterBuilder::new()
            .with_show_only_headers(["From", "To", "In-Reply-To", "Cc", "Subject"])
            .with_show_additional_headers(self.headers.iter().map(|(h, _)| h))
            .with_save_attachments(true);

        let mut msg = MessageBuilder::default();
        let mut cursor = TemplateCursor::default();

        msg = msg.from((
            self.from_name.as_deref().unwrap_or_default(),
            self.from.as_str(),
        ));

        cursor.row += 1;

        msg = msg.to(Vec::<Address>::new());
        cursor.row += 1;

        msg = msg.subject("");
        cursor.row += 1;

        for (key, val) in self.headers {
            msg = msg.header(key, Raw::new(val));
            cursor.row += 1;
        }

        msg = msg.text_body({
            let mut body = TemplateBody::new(cursor);

            body.push_str(&self.body);
            body.flush();
            body.cursor.lock();

            if self.signature_style.is_inlined() && !self.signature.is_empty() {
                body.push_str(&self.signature);
                body.flush();
            }

            cursor = body.cursor.clone();
            body
        });

        if self.signature_style.is_attached() && !self.signature.is_empty() {
            msg = msg.attachment("application/octet-stream", "signature", self.signature)
        }

        let content = interpreter.build().from_msg_builder(msg)?;

        Ok(Template::new_with_cursor(content, cursor))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum, serde::Deserialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "kebab-case"))]
pub enum TemplateComposeSignatureStyle {
    #[default]
    Inlined,
    Attached,
    Hidden,
}

impl TemplateComposeSignatureStyle {
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

    use crate::template::{
        compose::{TemplateBuilderCompose, TemplateComposeSignatureStyle},
        types::Template,
    };

    #[test]
    fn default() {
        assert_eq!(
            TemplateBuilderCompose {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: ",
                    "",
                    "", // cursor here
                ),
                (5, 0),
            ),
        );
    }

    #[test]
    fn with_headers() {
        assert_eq!(
            TemplateBuilderCompose {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                headers: vec![("In-Reply-To".into(), "".into()), ("Cc".into(), "".into())],
                ..Default::default()
            }
            .build()
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "In-Reply-To: ",
                    "Cc: ",
                    "Subject: ",
                    "",
                    "", // cursor here
                ),
                (7, 0),
            )
        );
    }

    #[test]
    fn with_body() {
        assert_eq!(
            TemplateBuilderCompose {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                body: "Hello, world!".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: ",
                    "",
                    "Hello, world!", // cursor here
                ),
                (5, 13),
            )
        );

        assert_eq!(
            TemplateBuilderCompose {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                body: "\n\nHello\n,\nworld!\n\n!".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: ",
                    "",
                    "",
                    "",
                    "Hello",
                    ",",
                    "world!",
                    "",
                    "!", // cursor here
                ),
                (11, 1),
            )
        );
    }

    #[test]
    fn with_signature() {
        assert_eq!(
            TemplateBuilderCompose {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: ",
                    "",
                    "", // cursor here
                    "",
                    "-- ",
                    "signature",
                ),
                (5, 0),
            )
        );

        assert_eq!(
            TemplateBuilderCompose {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                signature_style: TemplateComposeSignatureStyle::Hidden,
                ..Default::default()
            }
            .build()
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: ",
                    "",
                    "", // cursor here
                ),
                (5, 0),
            )
        );

        assert_eq!(
            TemplateBuilderCompose {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                signature_style: TemplateComposeSignatureStyle::Inlined,
                ..Default::default()
            }
            .build()
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: ",
                    "",
                    "", // cursor here
                    "",
                    "-- ",
                    "signature",
                ),
                (5, 0),
            )
        );
    }

    #[test]
    fn with_body_and_signature() {
        assert_eq!(
            TemplateBuilderCompose {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                body: "Hello, world!".into(),
                signature: "-- \nsignature".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: ",
                    "",
                    "Hello, world!", // cursor here
                    "",
                    "-- ",
                    "signature",
                ),
                (5, 13),
            )
        );

        assert_eq!(
            TemplateBuilderCompose {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                body: "\n\nHello,\n\nworld\n\n!".into(),
                signature: "-- \nsignature".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: ",
                    "Subject: ",
                    "",
                    "",
                    "",
                    "Hello,",
                    "",
                    "world",
                    "",
                    "!", // cursor
                    "",
                    "-- ",
                    "signature",
                ),
                (11, 1),
            )
        );
    }
}
