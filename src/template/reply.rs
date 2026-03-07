//! # Reply template
//!
//! The main structure of this module is the [`ReplyTemplateBuilder`],
//! which helps you to build template in order to reply to a message.

use std::{borrow::Cow, collections::HashSet, fmt};

use mail_builder::{
    headers::{address::Address, raw::Raw},
    MessageBuilder,
};
use mail_parser::{Addr, HeaderValue, Message};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    error::Result,
    message::{body::interpreter::FilterParts, interpreter::MimeInterpreterBuilder},
    template::{address, Template, TemplateBody, TemplateCursor},
};

/// Regex used to trim out prefix(es) from a subject.
///
/// Everything starting by "Re:" (case and whitespace insensitive) is
/// considered a prefix.
static SUBJECT: Lazy<Regex> = Lazy::new(|| Regex::new("(?i:\\s*re\\s*:\\s*)*(.*)").unwrap());

/// Trim out prefix(es) from the given subject.
fn trim_prefix(subject: &str) -> &str {
    match SUBJECT.captures(subject).and_then(|cap| cap.get(1)) {
        Some(subject) => subject.as_str(),
        None => subject,
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BuildReplyTemplateArgs {
    pub signature: String,
    pub signature_style: ReplyTemplateSignatureStyle,
    pub posting_style: ReplyTemplatePostingStyle,
    pub quote_headline: String,
    pub from: String,
    pub from_name: String,
    pub reply_all: bool,
    pub headers: Vec<(String, String)>,
    pub body: String,
    // pub downloads_dir: Option<PathBuf>,
    // pub headers: Option<Vec<String>>,
    // pub from: Addr<'static>,
}

pub fn build(msg: &Message, args: BuildReplyTemplateArgs) -> Result<Template> {
    let interpreter = MimeInterpreterBuilder::new()
        // .with_save_attachments_dir(opts.downloads_dir)
        .with_show_only_headers(["From", "To", "In-Reply-To", "Cc", "Subject"]);

    let thread_interpreter = MimeInterpreterBuilder::new()
        // .with_save_attachments_dir(opts.downloads_dir)
        .with_hide_all_headers()
        .with_show_parts(false)
        .with_show_attachments(false)
        .with_show_inline_attachments(false)
        .with_show_plain_texts_signature(false)
        .with_filter_parts(FilterParts::Include(vec![
            "text/plain".into(),
            "text/html".into(),
        ]));

    let mut cursor = TemplateCursor::default();

    let mut builder = MessageBuilder::new();

    let me = Addr::new(
        if args.from_name.is_empty() {
            None
        } else {
            Some(&args.from_name)
        },
        &args.from,
    );

    let sender = msg.header("Sender").unwrap_or(&HeaderValue::Empty);
    let from = msg.header("From").unwrap_or(&HeaderValue::Empty);
    let to = msg.header("To").unwrap_or(&HeaderValue::Empty);
    let reply_to = msg.header("Reply-To").unwrap_or(&HeaderValue::Empty);

    // In-Reply-To

    match msg.header("Message-ID") {
        Some(HeaderValue::Text(message_id)) => {
            builder = builder.in_reply_to(vec![message_id.clone()]);
            cursor.row += 1;
        }
        Some(HeaderValue::TextList(message_id)) => {
            builder = builder.in_reply_to(message_id.clone());
            cursor.row += 1;
        }
        _ => (),
    }

    // From

    builder = builder.from((args.from_name.as_str(), args.from.as_str()));
    cursor.row += 1;

    // To

    let mut curr_rcpts = Vec::<Address>::default();
    let mut all_rcpts_email = HashSet::<Cow<str>>::default();
    all_rcpts_email.insert(me.address.clone().unwrap());

    if !address::is_empty(reply_to) {
        address::push_builder_address(&mut all_rcpts_email, &mut curr_rcpts, &reply_to);
    } else {
        let from = if !address::is_empty(from) {
            from
        } else {
            sender
        };
        address::push_builder_address(&mut all_rcpts_email, &mut curr_rcpts, &from);
        address::push_builder_address(&mut all_rcpts_email, &mut curr_rcpts, &to);
    }

    builder = builder.to(Address::new_list(curr_rcpts.clone()));
    cursor.row += 1;

    // Cc

    if args.reply_all {
        let cc = msg.header("Cc").unwrap_or(&HeaderValue::Empty);

        curr_rcpts.clear();
        address::push_builder_address(&mut all_rcpts_email, &mut curr_rcpts, &cc);

        if !curr_rcpts.is_empty() {
            builder = builder.cc(curr_rcpts);
            cursor.row += 1;
        }
    }

    // Subject

    // TODO: make this customizable?
    let prefix = String::from("Re: ");
    let subject = trim_prefix(msg.subject().unwrap_or_default());

    builder = builder.subject(prefix + subject);
    cursor.row += 1;

    // Additional headers

    for (key, val) in args.headers {
        builder = builder.header(key, Raw::new(val));
        cursor.row += 1;
    }

    // Body

    builder = builder.text_body({
        let mut body = TemplateBody::new(cursor);

        let reply_body = thread_interpreter.build().from_msg(msg)?;
        let reply_body = reply_body.trim();

        if !reply_body.is_empty() && args.posting_style.is_bottom() {
            body.push_str(&args.quote_headline);

            for line in reply_body.lines() {
                body.push('>');
                if !line.starts_with('>') {
                    body.push(' ')
                }
                body.push_str(line);
                body.push('\n');
            }

            // drop last line feed
            body.pop();
            body.flush();
        }

        // when interleaved posting style, only push non-empty
        // body and do not lock the cursor (it must be locked at
        // the beginning of the quote)
        if args.posting_style.is_interleaved() {
            if !args.body.is_empty() {
                body.push_str(&args.body);
                body.flush();
            }
        }
        // when bottom or top posting style, push the body and
        // lock the cursor at the end of it
        else {
            body.push_str(&args.body);
            body.flush();
            body.cursor.lock();
        }

        // NOTE: hide this block for interleaved posting style?
        if args.signature_style.is_above_quote() && !args.signature.is_empty() {
            body.push_str(&args.signature);
            body.flush();
        }

        if !reply_body.is_empty() && !args.posting_style.is_bottom() {
            if args.posting_style.is_top() && !args.quote_headline.is_empty() {
                body.push_str(&args.quote_headline);
            }

            let mut lines_count = 0;
            for line in reply_body.lines() {
                lines_count += 1;

                body.push('>');
                if !line.starts_with('>') {
                    body.push(' ')
                }
                body.push_str(line);
                body.push('\n');
            }

            // drop last line feed
            body.pop();
            body.flush();

            // if interleaved posting style, put the cursor at the
            // beginning of the quote instead of leaving it at the
            // end
            if args.posting_style.is_interleaved() {
                body.cursor.row -= lines_count - 1;
                body.cursor.col = 0;
            }
        }

        if args.signature_style.is_below_quote() && !args.signature.is_empty() {
            body.push_str(&args.signature);
            body.flush();
        }

        cursor = body.cursor.clone();
        body
    });

    if args.signature_style.is_attached() && !args.signature.is_empty() {
        builder = builder.attachment("text/plain", "signature.txt", &args.signature)
    }

    let content = interpreter.build().from_msg_builder(builder)?;

    Ok(Template::new_with_cursor(content, cursor))
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum ReplyTemplatePostingStyle {
    #[default]
    Top,
    Bottom,
    Interleaved,
}

impl ReplyTemplatePostingStyle {
    pub fn is_top(&self) -> bool {
        self == &Self::Top
    }

    pub fn is_bottom(&self) -> bool {
        self == &Self::Bottom
    }

    pub fn is_interleaved(&self) -> bool {
        self == &Self::Interleaved
    }
}

impl fmt::Display for ReplyTemplatePostingStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Top => write!(f, "top"),
            Self::Bottom => write!(f, "bottom"),
            Self::Interleaved => write!(f, "interleaved"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum ReplyTemplateSignatureStyle {
    AboveQuote,
    #[default]
    BelowQuote,
    Attached,
    Hidden,
}

impl ReplyTemplateSignatureStyle {
    pub fn is_above_quote(&self) -> bool {
        self == &Self::AboveQuote
    }

    pub fn is_below_quote(&self) -> bool {
        self == &Self::BelowQuote
    }

    pub fn is_attached(&self) -> bool {
        self == &Self::Attached
    }

    pub fn is_hidden(&self) -> bool {
        self == &Self::Hidden
    }
}

impl fmt::Display for ReplyTemplateSignatureStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AboveQuote => write!(f, "above-quote"),
            Self::BelowQuote => write!(f, "below-quote"),
            Self::Attached => write!(f, "attached"),
            Self::Hidden => write!(f, "hidden"),
        }
    }
}

#[cfg(test)]
mod tests {
    use concat_with::concat_line;
    use mail_parser::MessageParser;

    use crate::template::{
        reply::{
            build, BuildReplyTemplateArgs, ReplyTemplatePostingStyle, ReplyTemplateSignatureStyle,
        },
        Template,
    };

    #[test]
    fn default() {
        let msg = MessageParser::new()
            .parse(concat_line!(
                "Content-Type: text/plain",
                "From: sender@localhost",
                "To: me@localhost",
                "Subject: subject",
                "",
                "",
                "",
            ))
            .unwrap();

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "", // cursor here
                ),
                (5, 0),
            )
        );
    }

    #[test]
    fn with_body() {
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
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "", // cursor here
                    "",
                    "> Hello, world!",
                ),
                (5, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    body: "Hello, back!".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "Hello, back!", // cursor here
                    "",
                    "> Hello, world!",
                ),
                (5, 12),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    body: "\n\nHello\n,\nworld!\n\n!".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "Hello",
                    ",",
                    "world!",
                    "",
                    "!", // cursor here
                    "",
                    "> Hello, world!",
                ),
                (11, 1),
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
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "", // cursor here
                    "",
                    "> Hello, world!",
                    "",
                    "-- ",
                    "signature",
                ),
                (5, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // force signature above quote
                    signature_style: ReplyTemplateSignatureStyle::AboveQuote,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "", // cursor here
                    "",
                    "-- ",
                    "signature",
                    "",
                    "> Hello, world!",
                ),
                (5, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // force signature hidden
                    signature_style: ReplyTemplateSignatureStyle::Hidden,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "", // cursor here
                    "",
                    "> Hello, world!",
                ),
                (5, 0),
            ),
        );
    }

    #[test]
    fn with_quote() {
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
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    // force the bottom-posting style
                    posting_style: ReplyTemplatePostingStyle::Bottom,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "> Hello, world!",
                    "",
                    "", // cursor here
                ),
                (7, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    // force the interleaved posting style
                    posting_style: ReplyTemplatePostingStyle::Interleaved,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "> Hello, world!", // cursor here
                ),
                (5, 0),
            ),
        );
    }

    #[test]
    fn with_body_and_signature() {
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
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "", // cursor here
                    "",
                    "> Hello, world!",
                    "",
                    "-- ",
                    "signature"
                ),
                (5, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // with single line body
                    body: "Hello, back!".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "Hello, back!", // cursor here
                    "",
                    "> Hello, world!",
                    "",
                    "-- ",
                    "signature"
                ),
                (5, 12),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // with single line body
                    body: "Hello, back!".into(),
                    // force signature above quote
                    signature_style: ReplyTemplateSignatureStyle::AboveQuote,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "Hello, back!", // cursor here
                    "",
                    "-- ",
                    "signature",
                    "",
                    "> Hello, world!",
                ),
                (5, 12),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // with multi lines body
                    body: "\n\nHello\n,\nworld!\n\n!".into(),
                    // force signature hidden
                    signature_style: ReplyTemplateSignatureStyle::Hidden,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "Hello",
                    ",",
                    "world!",
                    "",
                    "!", // cursor here
                    "",
                    "> Hello, world!",
                ),
                (11, 1),
            ),
        );
    }

    #[test]
    fn with_signature_and_quote() {
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
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // force signature above quote
                    signature_style: ReplyTemplateSignatureStyle::AboveQuote,
                    // force bottom-posting style
                    posting_style: ReplyTemplatePostingStyle::Bottom,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "> Hello, world!",
                    "",
                    "", // cursor here
                    "",
                    "-- ",
                    "signature",
                ),
                (7, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // force signature hidden
                    signature_style: ReplyTemplateSignatureStyle::Hidden,
                    // force bottom-posting style
                    posting_style: ReplyTemplatePostingStyle::Bottom,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "> Hello, world!",
                    "",
                    "", // cursor here
                ),
                (7, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // force signature above quote
                    signature_style: ReplyTemplateSignatureStyle::AboveQuote,
                    // force interleaved posting style
                    posting_style: ReplyTemplatePostingStyle::Interleaved,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "-- ",
                    "signature",
                    "",
                    "> Hello, world!", // cursor here
                ),
                (8, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // force signature hidden
                    signature_style: ReplyTemplateSignatureStyle::Hidden,
                    // force interleaved posting style
                    posting_style: ReplyTemplatePostingStyle::Interleaved,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "> Hello, world!", // cursor here
                ),
                (5, 0),
            ),
        );
    }

    #[test]
    fn with_body_and_quote() {
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
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    // with single line body
                    body: "Hello, back!".into(),
                    // force the bottom-posting style with body
                    posting_style: ReplyTemplatePostingStyle::Bottom,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "> Hello, world!",
                    "",
                    "Hello, back!", // cursor here
                ),
                (7, 12),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    // with single line body
                    body: "Hello, back!".into(),
                    // force the interleaved posting style with body
                    posting_style: ReplyTemplatePostingStyle::Interleaved,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "Hello, back!",
                    "",
                    "> Hello, world!", // cursor here
                ),
                (7, 0),
            ),
        );
    }

    #[test]
    fn with_body_signature_and_quote() {
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
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    signature: "-- \nsignature".into(),
                    // with single line body
                    body: "Hello, back!".into(),
                    // force signature above quote
                    signature_style: ReplyTemplateSignatureStyle::AboveQuote,
                    // force interleaved posting style
                    posting_style: ReplyTemplatePostingStyle::Interleaved,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "Hello, back!",
                    "",
                    "-- ",
                    "signature",
                    "",
                    "> Hello, world!", // cursor here
                ),
                (10, 0),
            ),
        );
    }

    #[test]
    fn reply_to_self() {
        let msg = MessageParser::new()
            .parse(concat_line!(
                "Content-Type: text/plain",
                "From: me@localhost",
                "To: to@localhost, to2@localhost",
                "Cc: cc@localhost, cc2@localhost, dot-not-reply@localhost",
                "Bcc: bcc@localhost",
                "Subject: Re: subject",
                "",
                "Hello from myself!",
                "",
                "-- ",
                "Regards,",
            ))
            .unwrap();

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from: "me@localhost".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: me@localhost",
                    "To: to@localhost, to2@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "> Hello from myself!",
                ),
                (5, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from: "me@localhost".into(),
                    reply_all: true,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: me@localhost",
                    "To: to@localhost, to2@localhost",
                    "Cc: cc@localhost, cc2@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "> Hello from myself!",
                ),
                (6, 0),
            ),
        );
    }

    #[test]
    fn reply_mailing_list_using_sender() {
        let msg = MessageParser::new()
            .parse(concat_line!(
                "Content-Type: text/plain",
                "Sender: sender@localhost",
                "To: mlist@localhost,other@localhost",
                "Cc: sender@localhost, cc@localhost, cc2@localhost, noreply@localhost, me@localhost",
                "Bcc: bcc@localhost",
                "Subject: Re: subject",
                "",
                "Hello from mailing list!",
                "",
                "-- ",
                "Regards,",
            ))
            .unwrap();

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from: "me@localhost".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: me@localhost",
                    "To: sender@localhost, mlist@localhost, other@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "> Hello from mailing list!",
                ),
                (5, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from: "me@localhost".into(),
                    reply_all: true,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: me@localhost",
                    "To: sender@localhost, mlist@localhost, other@localhost",
                    "Cc: cc@localhost, cc2@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "> Hello from mailing list!",
                ),
                (6, 0),
            ),
        );
    }

    #[test]
    fn reply_mailing_list_using_from() {
        let msg = MessageParser::new()
            .parse(concat_line!(
                "Content-Type: text/plain",
                "Sender: sender@localhost",
                "From: from@localhost",
                "To: mlist@localhost,other@localhost",
                "Cc: from@localhost, cc@localhost, cc2@localhost, noreply@localhost, me@localhost",
                "Bcc: bcc@localhost",
                "Subject: Re: subject",
                "",
                "Hello from mailing list!",
                "",
                "-- ",
                "Regards,",
            ))
            .unwrap();

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from: "me@localhost".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: me@localhost",
                    "To: from@localhost, mlist@localhost, other@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "> Hello from mailing list!",
                ),
                (5, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from: "me@localhost".into(),
                    reply_all: true,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: me@localhost",
                    "To: from@localhost, mlist@localhost, other@localhost",
                    "Cc: cc@localhost, cc2@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "> Hello from mailing list!",
                ),
                (6, 0),
            ),
        );
    }

    #[test]
    fn reply_mailing_list_using_reply_to() {
        let msg = MessageParser::new()
            .parse(concat_line!(
                "Content-Type: text/plain",
                "From: from@localhost",
                "Sender: sender@localhost",
                "Reply-To: reply-to@localhost",
                "To: mlist@localhost,other@localhost",
                "Cc: from@localhost, cc@localhost, cc2@localhost, noreply@localhost",
                "Bcc: bcc@localhost",
                "Subject: Re: subject",
                "",
                "Hello from mailing list!",
                "",
                "-- ",
                "Regards,",
            ))
            .unwrap();

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from: "me@localhost".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: me@localhost",
                    "To: reply-to@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "> Hello from mailing list!",
                ),
                (5, 0),
            ),
        );

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from: "me@localhost".into(),
                    reply_all: true,
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: me@localhost",
                    "To: reply-to@localhost",
                    "Cc: from@localhost, cc@localhost, cc2@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "> Hello from mailing list!",
                ),
                (6, 0),
            ),
        );
    }

    #[test]
    fn reply_mailing_list_multiple_senders() {
        let msg = MessageParser::new()
            .parse(concat_line!(
                "Content-Type: text/plain",
                "From: from@localhost",
                "To: mlist@localhost,me@localhost",
                "Cc: cc@localhost, cc2@localhost",
                "Subject: subject",
                "",
                "Hello from mailing list!",
                "",
                "-- ",
                "Regards,",
            ))
            .unwrap();

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from: "me@localhost".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: me@localhost",
                    "To: from@localhost, mlist@localhost",
                    "Subject: Re: subject",
                    "",
                    "",
                    "",
                    "> Hello from mailing list!",
                ),
                (5, 0),
            ),
        );
    }

    #[test]
    fn trim_subject_prefix() {
        assert_eq!(super::trim_prefix("Hello, world!"), "Hello, world!");
        assert_eq!(super::trim_prefix("re:Hello, world!"), "Hello, world!");
        assert_eq!(super::trim_prefix("Re   :Hello, world!"), "Hello, world!");
        assert_eq!(super::trim_prefix("rE:   Hello, world!"), "Hello, world!");
        assert_eq!(
            super::trim_prefix("  RE:  re  :Hello, world!"),
            "Hello, world!"
        );
    }

    #[test]
    fn should_hide_part_markup_in_html_reply_thread() {
        let msg = MessageParser::new()
            .parse(concat_line!(
                "Content-Type: text/html",
                "From: sender@localhost",
                "To: me@localhost",
                "Subject: subject",
                "",
                "<h1>Hello, world!</h1>",
                "",
            ))
            .unwrap();

        assert_eq!(
            build(
                &msg,
                BuildReplyTemplateArgs {
                    from_name: "Me".into(),
                    from: "me@localhost".into(),
                    ..Default::default()
                },
            )
            .unwrap(),
            Template::new_with_cursor(
                concat_line!(
                    "From: Me <me@localhost>",
                    "To: sender@localhost",
                    "Subject: Re: subject",
                    "",
                    "", // cursor here
                    "",
                    "> Hello, world!",
                ),
                (5, 0),
            ),
        );
    }
}
