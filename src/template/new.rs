//! # New template
//!
//! The main structure of this module is the [`NewTemplateBuilder`],
//! which helps you to build template in order to compose a new
//! message from scratch.

use std::fmt;

use mail_builder::{
    headers::{address::Address, raw::Raw},
    MessageBuilder,
};

use crate::{
    error::Result,
    message::interpreter::MimeInterpreterBuilder,
    template::{Template, TemplateBody, TemplateCursor},
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BuildNewTemplateArgs {
    pub signature: String,
    pub signature_style: NewTemplateSignatureStyle,
    pub from: String,
    pub from_name: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

pub fn build(args: BuildNewTemplateArgs) -> Result<Template> {
    let interpreter = MimeInterpreterBuilder::new().with_show_only_headers([
        "From",
        "To",
        "In-Reply-To",
        "Cc",
        "Subject",
    ]);

    let mut msg = MessageBuilder::default();
    let mut cursor = TemplateCursor::default();

    msg = msg.from((args.from_name.as_str(), args.from.as_str()));
    cursor.row += 1;

    msg = msg.to(Vec::<Address>::new());
    cursor.row += 1;

    msg = msg.subject("");
    cursor.row += 1;

    for (key, val) in args.headers {
        msg = msg.header(key, Raw::new(val));
        cursor.row += 1;
    }

    msg = msg.text_body({
        let mut body = TemplateBody::new(cursor);

        body.push_str(&args.body);
        body.flush();
        body.cursor.lock();

        if args.signature_style.is_inlined() && !args.signature.is_empty() {
            body.push_str(&args.signature);
            body.flush();
        }

        cursor = body.cursor.clone();
        body
    });

    if args.signature_style.is_attached() && !args.signature.is_empty() {
        msg = msg.attachment("text/plain", "signature.txt", args.signature)
    }

    let content = interpreter.build().from_msg_builder(msg)?;

    Ok(Template::new_with_cursor(content, cursor))
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum NewTemplateSignatureStyle {
    #[default]
    Inlined,
    Attached,
    Hidden,
}

impl NewTemplateSignatureStyle {
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

impl fmt::Display for NewTemplateSignatureStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inlined => write!(f, "inlined"),
            Self::Attached => write!(f, "attached"),
            Self::Hidden => write!(f, "hidden"),
        }
    }
}

#[cfg(test)]
mod tests {
    use concat_with::concat_line;

    use crate::template::{
        new::{build, BuildNewTemplateArgs, NewTemplateSignatureStyle},
        Template,
    };

    #[test]
    fn default() {
        assert_eq!(
            build(BuildNewTemplateArgs {
                from_name: "Me".into(),
                from: "me@localhost".into(),
                ..Default::default()
            })
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
            build(BuildNewTemplateArgs {
                from_name: "Me".into(),
                from: "me@localhost".into(),
                headers: vec![("In-Reply-To".into(), "".into()), ("Cc".into(), "".into())],
                ..Default::default()
            })
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
            build(BuildNewTemplateArgs {
                from_name: "Me".into(),
                from: "me@localhost".into(),
                body: "Hello, world!".into(),
                ..Default::default()
            })
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
            build(BuildNewTemplateArgs {
                from_name: "Me".into(),
                from: "me@localhost".into(),
                body: "\n\nHello\n,\nworld!\n\n!".into(),
                ..Default::default()
            })
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
            build(BuildNewTemplateArgs {
                from_name: "Me".into(),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                ..Default::default()
            })
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
            build(BuildNewTemplateArgs {
                from_name: "Me".into(),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                signature_style: NewTemplateSignatureStyle::Hidden,
                ..Default::default()
            })
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
            build(BuildNewTemplateArgs {
                from_name: "Me".into(),
                from: "me@localhost".into(),
                signature: "-- \nsignature".into(),
                signature_style: NewTemplateSignatureStyle::Inlined,
                ..Default::default()
            })
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
            build(BuildNewTemplateArgs {
                from_name: "Me".into(),
                from: "me@localhost".into(),
                body: "Hello, world!".into(),
                signature: "-- \nsignature".into(),
                ..Default::default()
            })
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
            build(BuildNewTemplateArgs {
                from_name: "Me".into(),
                from: "me@localhost".into(),
                body: "\n\nHello,\n\nworld\n\n!".into(),
                signature: "-- \nsignature".into(),
                ..Default::default()
            })
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
