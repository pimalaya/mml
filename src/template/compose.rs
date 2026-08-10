//! Compose template builder: see [`MmlTemplateComposeBuilder`] for
//! building a draft template for a brand-new message (no source).

use mail_builder::{
    MessageBuilder,
    headers::{address::Address, raw::Raw},
};

use crate::{
    error::Result,
    interpreter::{
        body::MmlMimeBodyInterpreter,
        message::{MmlFilterHeaders, MmlMimeInterpreter},
    },
    template::{MmlTemplate, MmlTemplateBody, MmlTemplateCursor},
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MmlTemplateComposeBuilder {
    pub signature: String,
    pub signature_style: MmlTemplateComposeSignatureStyle,
    pub from: String,
    pub from_name: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl MmlTemplateComposeBuilder {
    pub fn build(self) -> Result<MmlTemplate> {
        let mut show_headers =
            MmlFilterHeaders::include(["From", "To", "In-Reply-To", "Cc", "Subject"]);
        show_headers.extend(self.headers.iter().map(|(header, _)| header));

        let interpreter = MmlMimeInterpreter {
            show_headers,
            body: MmlMimeBodyInterpreter {
                save_attachments: true,
                ..Default::default()
            },
        };

        let mut msg = MessageBuilder::default();
        let mut cursor = MmlTemplateCursor::default();

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
            let mut body = MmlTemplateBody::new(cursor);

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

        let content = interpreter.interpret_builder(msg)?;

        Ok(MmlTemplate::new_with_cursor(content, cursor))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum, serde::Deserialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "kebab-case"))]
pub enum MmlTemplateComposeSignatureStyle {
    #[default]
    Inlined,
    Attached,
    Hidden,
}

impl MmlTemplateComposeSignatureStyle {
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
        MmlTemplate,
        compose::{MmlTemplateComposeBuilder, MmlTemplateComposeSignatureStyle},
    };

    #[test]
    fn default() {
        assert_eq!(
            MmlTemplateComposeBuilder {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            MmlTemplate::new_with_cursor(
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
            MmlTemplateComposeBuilder {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                headers: vec![("In-Reply-To".into(), "".into()), ("Cc".into(), "".into())],
                ..Default::default()
            }
            .build()
            .unwrap(),
            MmlTemplate::new_with_cursor(
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
            MmlTemplateComposeBuilder {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                body: "Hello, world!".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            MmlTemplate::new_with_cursor(
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
            MmlTemplateComposeBuilder {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                body: "\n\nHello\n,\nworld!\n\n!".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            MmlTemplate::new_with_cursor(
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
            MmlTemplateComposeBuilder {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            MmlTemplate::new_with_cursor(
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
            MmlTemplateComposeBuilder {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                signature_style: MmlTemplateComposeSignatureStyle::Hidden,
                ..Default::default()
            }
            .build()
            .unwrap(),
            MmlTemplate::new_with_cursor(
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
            MmlTemplateComposeBuilder {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                signature_style: MmlTemplateComposeSignatureStyle::Inlined,
                ..Default::default()
            }
            .build()
            .unwrap(),
            MmlTemplate::new_with_cursor(
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
            MmlTemplateComposeBuilder {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                body: "Hello, world!".into(),
                signature: "-- \nsignature".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            MmlTemplate::new_with_cursor(
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
            MmlTemplateComposeBuilder {
                from_name: Some("Me".into()),
                from: "me@localhost".into(),
                body: "\n\nHello,\n\nworld\n\n!".into(),
                signature: "-- \nsignature".into(),
                ..Default::default()
            }
            .build()
            .unwrap(),
            MmlTemplate::new_with_cursor(
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
