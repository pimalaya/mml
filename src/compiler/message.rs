//! MML → MIME message compilation: parses MML headers + body, hands
//! the body off to [`crate::compiler::body`], stitches the result
//! into MIME bytes.

use mail_builder::headers::text::Text;
use mail_parser::MessageParser;

use crate::{
    compiler::body::MmlBodyCompiler,
    error::{MmlError, Result},
};
/// MML → MIME message compiler.
#[derive(Clone, Debug, Default)]
pub struct MmlCompiler {
    /// The body-level compiler, carrying the part options.
    pub body: MmlBodyCompiler,
}

impl MmlCompiler {
    /// Compiles an MML message into MIME bytes.
    ///
    /// Headers are copied over verbatim from the MML source, with a
    /// `MIME-Version` added.
    pub fn compile(&self, mml: &str) -> Result<Vec<u8>> {
        let mml_msg = MessageParser::new()
            .parse(mml.as_bytes())
            .ok_or(MmlError::ParseMessageError)?;

        let mml_body = mml_msg
            .text_bodies()
            .next()
            .ok_or(MmlError::ParseMmlEmptyBodyError)?
            .text_contents()
            .ok_or(MmlError::ParseMmlEmptyBodyContentError)?;

        let mut mime_msg = self.body.compile(mml_body)?;

        mime_msg = mime_msg.header("MIME-Version", Text::new("1.0"));

        for header in mml_msg.headers() {
            let key = header.name.as_str();
            let val = crate::header::to_builder_val(header);
            mime_msg = mime_msg.header(key, val);
        }

        mime_msg
            .write_to_vec()
            .map_err(MmlError::CompileMmlMessageToVecError)
    }
}

#[cfg(all(test, feature = "interpreter"))]
mod tests {
    use concat_with::concat_line;

    use crate::{
        compiler::message::MmlCompiler,
        interpreter::message::{MmlFilterHeaders, MmlMimeInterpreter},
    };

    #[test]
    fn non_ascii_headers() {
        let mml = concat_line!(
            "Message-ID: <id@localhost>",
            "Date: Thu, 1 Jan 1970 00:00:00 +0000",
            "From: Frȯm <from@localhost>",
            "To: Tó <to@localhost>",
            "Subject: Subjêct",
            "",
            "Hello, world!",
            "",
        );

        let mime = MmlCompiler::default().compile(mml).unwrap();

        let mml_msg = MmlMimeInterpreter {
            show_headers: MmlFilterHeaders::include(["From", "To", "Subject"]),
            ..Default::default()
        }
        .interpret(&mime)
        .unwrap();

        let expected_mml_msg = concat_line!(
            "From: Frȯm <from@localhost>",
            "To: Tó <to@localhost>",
            "Subject: Subjêct",
            "",
            "Hello, world!",
            "",
        );

        assert_eq!(mml_msg, expected_mml_msg);
    }

    #[test]
    fn message_id_with_angles() {
        let mml = concat_line!(
            "From: Hugo Osvaldo Barrera <hugo@localhost>",
            "To: Hugo Osvaldo Barrera <hugo@localhost>",
            "Cc:",
            "Subject: Blah",
            "Message-ID: <bfb64e12-b7d4-474c-a658-8a221365f8ca@localhost>",
            "",
            "Test message",
            "",
        );

        let mime = MmlCompiler::default().compile(mml).unwrap();

        let mml_msg = MmlMimeInterpreter {
            show_headers: MmlFilterHeaders::include(["Message-ID"]),
            ..Default::default()
        }
        .interpret(&mime)
        .unwrap();

        let expected_mml_msg = concat_line!(
            "Message-ID: <bfb64e12-b7d4-474c-a658-8a221365f8ca@localhost>",
            "",
            "Test message",
            "",
        );

        assert_eq!(mml_msg, expected_mml_msg);
    }

    #[test]
    fn message_id_without_angles() {
        let mml = concat_line!(
            "From: Hugo Osvaldo Barrera <hugo@localhost>",
            "To: Hugo Osvaldo Barrera <hugo@localhost>",
            "Cc:",
            "Subject: Blah",
            "Message-ID: bfb64e12-b7d4-474c-a658-8a221365f8ca@localhost",
            "",
            "Test message",
            "",
        );

        let mime = MmlCompiler::default().compile(mml).unwrap();

        let mml_msg = MmlMimeInterpreter {
            show_headers: MmlFilterHeaders::include(["Message-ID"]),
            ..Default::default()
        }
        .interpret(&mime)
        .unwrap();

        let expected_mml_msg = concat_line!(
            "Message-ID: <bfb64e12-b7d4-474c-a658-8a221365f8ca@localhost>",
            "",
            "Test message",
            "",
        );

        assert_eq!(mml_msg, expected_mml_msg);
    }

    #[test]
    fn mml_markup_unescaped() {
        let mml = concat_line!(
            "Message-ID: <id@localhost>",
            "Date: Thu, 1 Jan 1970 00:00:00 +0000",
            "From: from@localhost",
            "To: to@localhost",
            "Subject: subject",
            "",
            "<#!part>This should be unescaped<#!/part>",
            "",
        );

        let mime = MmlCompiler::default().compile(mml).unwrap();
        let mime_msg_str = String::from_utf8(mime.clone()).unwrap();

        let mml_msg = MmlMimeInterpreter {
            show_headers: MmlFilterHeaders::include(["From", "To", "Subject"]),
            ..Default::default()
        }
        .interpret(&mime)
        .unwrap();

        let expected_mml_msg = concat_line!(
            "From: from@localhost",
            "To: to@localhost",
            "Subject: subject",
            "",
            "<#!part>This should be unescaped<#!/part>",
            "",
        );

        assert!(!mime_msg_str.contains("<#!part>"));
        assert!(mime_msg_str.contains("<#part>"));

        assert!(!mime_msg_str.contains("<#!/part>"));
        assert!(mime_msg_str.contains("<#/part>"));

        assert_eq!(mml_msg, expected_mml_msg);
    }
}
