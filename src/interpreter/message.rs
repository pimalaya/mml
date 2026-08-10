//! MIME → MML message interpretation: filters headers, hands the body
//! off to [`crate::interpreter::body`], returns the rendered MML as a
//! [`String`].

use mail_builder::MessageBuilder;
use mail_parser::{HeaderValue, Message, MessageParser};

use crate::{
    error::{MmlError, Result},
    header,
    interpreter::body::MmlMimeBodyInterpreter,
};

/// Filters headers to show in the interpreted message.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MmlFilterHeaders {
    /// Include all available headers to the interpreted message.
    #[default]
    All,
    /// Include given headers to the interpreted message.
    Include(Vec<String>),
    /// Exclude given headers from the interpreted message.
    Exclude(Vec<String>),
}

impl MmlFilterHeaders {
    /// Includes the given headers, dropping duplicates.
    pub fn include(headers: impl IntoIterator<Item = impl ToString>) -> Self {
        Self::Include(dedup(headers))
    }

    /// Excludes the given headers, dropping duplicates.
    pub fn exclude(headers: impl IntoIterator<Item = impl ToString>) -> Self {
        Self::Exclude(dedup(headers))
    }

    /// Whether the given header is already covered by this filter.
    pub fn contains(&self, header: &String) -> bool {
        match self {
            Self::All => false,
            Self::Include(headers) => headers.contains(header),
            Self::Exclude(headers) => !headers.contains(header),
        }
    }

    /// Adds headers on top of the current filter.
    ///
    /// FIXME: an `All` filter collapses into an `Include` of just the
    /// added headers, which drops every other one. It needs a shape
    /// that can express "all, plus these".
    pub fn extend(&mut self, headers: impl IntoIterator<Item = impl ToString>) {
        let next: Vec<String> = dedup(headers)
            .into_iter()
            .filter(|header| !self.contains(header))
            .collect();

        match self {
            Self::All => *self = Self::Include(next),
            Self::Include(headers) | Self::Exclude(headers) => headers.extend(next),
        }
    }
}

fn dedup(headers: impl IntoIterator<Item = impl ToString>) -> Vec<String> {
    headers.into_iter().fold(Vec::new(), |mut headers, header| {
        let header = header.to_string();
        if !headers.contains(&header) {
            headers.push(header)
        }
        headers
    })
}

/// MIME → MML message interpreter.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MmlMimeInterpreter {
    /// The strategy to display headers.
    pub show_headers: MmlFilterHeaders,
    /// The body-level interpreter, carrying the part options.
    pub body: MmlMimeBodyInterpreter,
}

impl MmlMimeInterpreter {
    /// Interprets the given MIME message bytes as MML.
    pub fn interpret(&self, mime: impl AsRef<[u8]>) -> Result<String> {
        let msg = MessageParser::new()
            .parse(mime.as_ref())
            .ok_or(MmlError::ParseRawEmailError)?;

        self.interpret_msg(&msg)
    }

    /// Interprets the message a builder would produce as MML.
    pub fn interpret_builder(&self, builder: MessageBuilder<'_>) -> Result<String> {
        let mime = builder.write_to_vec().map_err(MmlError::BuildEmailError)?;

        self.interpret(&mime)
    }

    /// Interprets an already-parsed MIME message as MML.
    ///
    /// NOTE: kept next to [`Self::interpret`] so a caller walking a
    /// thread does not reparse every message.
    pub fn interpret_msg(&self, msg: &Message<'_>) -> Result<String> {
        let mut mml = String::new();

        let mut push = |key: &str, val: &HeaderValue<'_>| {
            let val = header::display_value(key, val);
            mml.push_str(&format!("{key}: {val}\n"));
        };

        match &self.show_headers {
            MmlFilterHeaders::All => msg
                .headers()
                .iter()
                .for_each(|header| push(header.name.as_str(), &header.value)),
            MmlFilterHeaders::Include(keys) => keys
                .iter()
                .filter_map(|key| msg.header(key.as_str()).map(|val| (key, val)))
                .for_each(|(key, val)| push(key, val)),
            MmlFilterHeaders::Exclude(keys) => msg
                .headers()
                .iter()
                .filter(|header| !keys.contains(&header.name.as_str().to_owned()))
                .for_each(|header| push(header.name.as_str(), &header.value)),
        };

        if !mml.is_empty() {
            mml.push('\n');
        }

        mml.push_str(&self.body.interpret_msg(msg)?);

        Ok(mml)
    }
}

#[cfg(test)]
mod tests {
    use concat_with::concat_line;
    use mail_builder::MessageBuilder;

    use crate::interpreter::message::{MmlFilterHeaders, MmlMimeInterpreter};

    fn msg_builder() -> MessageBuilder<'static> {
        MessageBuilder::new()
            .message_id("id@localhost")
            .in_reply_to("reply-id@localhost")
            .date(0_u64)
            .from("from@localhost")
            .to("to@localhost")
            .subject("subject")
            .text_body("Hello, world!")
    }

    #[test]
    fn all_headers() {
        let mml = MmlMimeInterpreter::default()
            .interpret_builder(msg_builder())
            .unwrap();

        let expected_mml = concat_line!(
            "Message-ID: <id@localhost>",
            "In-Reply-To: <reply-id@localhost>",
            "Date: Thu, 1 Jan 1970 00:00:00 +0000",
            "From: from@localhost",
            "To: to@localhost",
            "Subject: subject",
            "MIME-Version: 1.0",
            "Content-Type: text/plain; charset=utf-8",
            "Content-Transfer-Encoding: 7bit",
            "",
            "Hello, world!",
        );

        assert_eq!(mml, expected_mml);
    }

    #[test]
    fn only_headers() {
        let mml = MmlMimeInterpreter {
            show_headers: MmlFilterHeaders::include(["From", "Subject"]),
            ..Default::default()
        }
        .interpret_builder(msg_builder())
        .unwrap();

        let expected_mml = concat_line!(
            "From: from@localhost",
            "Subject: subject",
            "",
            "Hello, world!",
        );

        assert_eq!(mml, expected_mml);
    }

    #[test]
    fn only_headers_duplicated() {
        let mml = MmlMimeInterpreter {
            show_headers: MmlFilterHeaders::include(["From", "Subject", "From"]),
            ..Default::default()
        }
        .interpret_builder(msg_builder())
        .unwrap();

        let expected_mml = concat_line!(
            "From: from@localhost",
            "Subject: subject",
            "",
            "Hello, world!",
        );

        assert_eq!(mml, expected_mml);
    }

    #[test]
    fn no_headers() {
        let mml = MmlMimeInterpreter {
            show_headers: MmlFilterHeaders::Include(Vec::new()),
            ..Default::default()
        }
        .interpret_builder(msg_builder())
        .unwrap();

        let expected_mml = concat_line!("Hello, world!");

        assert_eq!(mml, expected_mml);
    }

    #[test]
    fn mml_markup_escaped() {
        let msg_builder = MessageBuilder::new()
            .message_id("id@localhost")
            .in_reply_to("reply-id@localhost")
            .date(0_u64)
            .from("from@localhost")
            .to("to@localhost")
            .subject("subject")
            .text_body("<#part>Should be escaped.<#/part>");

        let mml = MmlMimeInterpreter {
            show_headers: MmlFilterHeaders::include(["From", "Subject"]),
            ..Default::default()
        }
        .interpret_builder(msg_builder)
        .unwrap();

        let expected_mml = concat_line!(
            "From: from@localhost",
            "Subject: subject",
            "",
            "<#!part>Should be escaped.<#!/part>",
        );

        assert_eq!(mml, expected_mml);
    }
}
