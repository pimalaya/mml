//! MML → MIME body compilation: walks the parsed AST and emits MIME
//! parts (inline, attached, multipart) via [`mail_builder`].

use std::{ffi::OsStr, fs, ops::Deref, path::PathBuf};

use log::debug;
use mail_builder::{
    MessageBuilder,
    headers::content_type::ContentType,
    mime::{BodyPart, MimePart},
};

use crate::{
    compiler::{
        flowed::encode,
        parser::{
            parts,
            prelude::*,
            tokens::{Part, Props},
        },
    },
    error::{MmlError, Result},
    grammar::{
        ALTERNATIVE, ATTACHMENT, DISPOSITION, ENCODING, ENCODING_7BIT, ENCODING_8BIT,
        ENCODING_BASE64, ENCODING_QUOTED_PRINTABLE, ENCRYPT, FILENAME, FLOWED, FORMAT, INLINE,
        MIXED, MULTIPART_BEGIN, MULTIPART_BEGIN_ESCAPED, MULTIPART_END, MULTIPART_END_ESCAPED,
        NAME, PART_BEGIN, PART_BEGIN_ESCAPED, PART_END, PART_END_ESCAPED, RECIPIENT_FILENAME,
        RELATED, SIGN, TYPE,
    },
};

/// The format of a `text/plain` part (RFC 3676).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "cli", derive(serde::Deserialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "kebab-case"))]
pub enum MmlBodyFormat {
    /// Every newline is a hard break, shipped as written.
    #[default]
    Fixed,
    /// Lines longer than the column are broken with soft breaks the
    /// reader joins back.
    Flowed,
}

/// MML → MIME message body compiler.
///
/// The compiler follows the builder pattern, where the build function
/// is named `compile`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MmlBodyCompiler {
    /// The format of `text/plain` parts carrying no `format`
    /// property, bare text included.
    pub format: MmlBodyFormat,
    /// The column flowed lines are broken at, quote prefix included.
    ///
    /// 998, the RFC 5322 line limit, ships every paragraph whole.
    pub flowed_column: usize,
}

impl Default for MmlBodyCompiler {
    fn default() -> Self {
        Self {
            format: MmlBodyFormat::default(),
            flowed_column: 72,
        }
    }
}

/// The content type of a flowed part.
///
/// NOTE: mail-builder adds `charset=utf-8` only to a content type
/// carrying no attribute, so it is set here explicitly.
fn flowed_content_type() -> ContentType<'static> {
    ContentType::new("text/plain")
        .attribute("charset", "utf-8")
        .attribute(FORMAT, FLOWED)
}

impl MmlBodyCompiler {
    /// Whether the given `text/plain` part properties ask for a
    /// flowed body.
    ///
    /// A signed or encrypted part stays fixed whatever its format
    /// says, as `mml.el` does.
    fn flows(&self, props: &Props) -> bool {
        if props.contains_key(SIGN) || props.contains_key(ENCRYPT) {
            return false;
        }

        match props.get(FORMAT) {
            Some(format) => *format == FLOWED,
            None => self.format == MmlBodyFormat::Flowed,
        }
    }

    /// Replace escaped opening and closing tags by normal opening and
    /// closing tags.
    fn unescape_mml_markup(text: impl AsRef<str>) -> String {
        text.as_ref()
            .replace(PART_BEGIN_ESCAPED, PART_BEGIN)
            .replace(PART_END_ESCAPED, PART_END)
            .replace(MULTIPART_BEGIN_ESCAPED, MULTIPART_BEGIN)
            .replace(MULTIPART_END_ESCAPED, MULTIPART_END)
    }

    /// Compile given parts parsed from a MML body to a
    /// [MessageBuilder].
    fn compile_parts(&self, parts: Vec<Part<'_>>) -> Result<MessageBuilder<'static>> {
        let mut builder = MessageBuilder::new();

        builder = match parts.len() {
            0 => builder.text_body(String::new()),
            1 => builder.body(self.compile_part(parts.into_iter().next().unwrap())?),
            _ => {
                let mut compiled_parts = Vec::new();

                for part in parts {
                    let part = self.compile_part(part)?;
                    compiled_parts.push(part);
                }

                builder.body(MimePart::new("multipart/mixed", compiled_parts))
            }
        };

        Ok(builder)
    }

    /// Compile the given part parsed from MML body to a [MimePart].
    pub(crate) fn compile_part(&self, part: Part<'_>) -> Result<MimePart<'static>> {
        match part {
            Part::Multi(props, parts) => {
                let no_parts = BodyPart::Multipart(Vec::new());

                let mut multi_part = match props.get(TYPE) {
                    Some(&MIXED) | None => MimePart::new("multipart/mixed", no_parts),
                    Some(&ALTERNATIVE) => MimePart::new("multipart/alternative", no_parts),
                    Some(&RELATED) => MimePart::new("multipart/related", no_parts),
                    Some(unknown) => {
                        debug!("unknown multipart type {unknown}, falling back to mixed");
                        MimePart::new("multipart/mixed", no_parts)
                    }
                };

                for part in parts {
                    multi_part.add_part(self.compile_part(part)?)
                }

                Ok(multi_part)
            }
            Part::Single(ref props, body) => {
                let fpath = match props.get(FILENAME) {
                    Some(fpath) => {
                        let fpath = match shellexpand::full(fpath) {
                            Ok(path) => PathBuf::from(path.as_ref()),
                            Err(_) => PathBuf::from(fpath),
                        };

                        Some(fpath.canonicalize().unwrap_or(fpath))
                    }
                    None => None,
                };

                let mut part = match &fpath {
                    Some(fpath) => {
                        let contents = fs::read(&fpath)
                            .map_err(|err| MmlError::ReadAttachmentError(err, fpath.clone()))?;
                        let mut ctype =
                            ContentType::new(Part::get_or_guess_content_type(props, &contents));
                        if let Some(name) = props.get(NAME) {
                            ctype = ctype.attribute("name", name.to_string());
                        }
                        MimePart::new(ctype, contents)
                    }
                    None => {
                        let ctype = Part::get_or_guess_content_type(props, body.as_bytes());

                        // NOTE: a type carrying its own parameters is
                        // opaque, only a bare text/plain is flowed.
                        let (mut ctype, body) = if ctype == "text/plain" && self.flows(props) {
                            (flowed_content_type(), encode(body, self.flowed_column))
                        } else {
                            (ContentType::new(ctype), body.to_owned())
                        };

                        if let Some(name) = props.get(NAME) {
                            ctype = ctype.attribute("name", name.to_string());
                        }

                        MimePart::new(ctype, body)
                    }
                };

                part = match props.get(ENCODING) {
                    Some(&ENCODING_7BIT) => part.transfer_encoding(ENCODING_7BIT),
                    Some(&ENCODING_8BIT) => part.transfer_encoding(ENCODING_8BIT),
                    Some(&ENCODING_QUOTED_PRINTABLE) => {
                        part.transfer_encoding(ENCODING_QUOTED_PRINTABLE)
                    }
                    Some(&ENCODING_BASE64) => part.transfer_encoding(ENCODING_BASE64),
                    _ => part,
                };

                part = match props.get(DISPOSITION) {
                    Some(&INLINE) => part.inline(),
                    Some(&ATTACHMENT) => part.attachment(
                        props
                            .get(RECIPIENT_FILENAME)
                            .map(Deref::deref)
                            .or_else(|| match &fpath {
                                Some(fpath) => fpath.file_name().and_then(OsStr::to_str),
                                None => None,
                            })
                            .unwrap_or("noname")
                            .to_owned(),
                    ),
                    _ if fpath.is_some() => part.attachment(
                        props
                            .get(RECIPIENT_FILENAME)
                            .map(ToString::to_string)
                            .or_else(|| {
                                fpath
                                    .unwrap()
                                    .file_name()
                                    .and_then(OsStr::to_str)
                                    .map(ToString::to_string)
                            })
                            .unwrap_or_else(|| "noname".to_string()),
                    ),
                    _ => part,
                };

                Ok(part)
            }
            Part::PlainText(body) => {
                let body = Self::unescape_mml_markup(body);

                let part = match self.format {
                    MmlBodyFormat::Fixed => MimePart::new("text/plain", body),
                    MmlBodyFormat::Flowed => {
                        MimePart::new(flowed_content_type(), encode(&body, self.flowed_column))
                    }
                };

                Ok(part)
            }
        }
    }

    /// Parses the given raw MML body into its parts.
    pub(crate) fn parse<'a>(&self, mml_body: &'a str) -> Result<Vec<Part<'a>>> {
        let res = parts::parts().parse(mml_body);

        match res.output() {
            Some(parts) => Ok(parts.to_owned()),
            None => {
                let errs = res.errors().map(|err| err.clone().into_owned()).collect();
                Err(MmlError::ParseMmlError(errs, mml_body.to_owned()))
            }
        }
    }

    /// Compile the given raw MML body to MIME body.
    pub fn compile(&self, mml_body: &str) -> Result<MessageBuilder<'static>> {
        self.compile_parts(self.parse(mml_body)?)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use concat_with::concat_line;
    use tempfile::Builder;

    use crate::compiler::body::MmlBodyCompiler;

    #[test]
    fn plain() {
        let mml_body = concat_line!("Hello, world!", "");

        let msg = MmlBodyCompiler::default()
            .compile(mml_body)
            .unwrap()
            .message_id("id@localhost")
            .date(0_u64)
            .write_to_string()
            .unwrap();

        let expected_msg = concat_line!(
            "Message-ID: <id@localhost>\r",
            "Date: Thu, 1 Jan 1970 00:00:00 +0000\r",
            "MIME-Version: 1.0\r",
            "Content-Type: text/plain; charset=\"utf-8\"\r",
            "Content-Transfer-Encoding: 7bit\r",
            "\r",
            "Hello, world!\r",
            "",
        );

        assert_eq!(msg, expected_msg);
    }

    #[test]
    fn html() {
        let mml_body = concat_line!(
            "<#part type=\"text/html\">",
            "<h1>Hello, world!</h1>",
            "<#/part>",
        );

        let msg = MmlBodyCompiler::default()
            .compile(mml_body)
            .unwrap()
            .message_id("id@localhost")
            .date(0_u64)
            .write_to_string()
            .unwrap();

        let expected_msg = concat_line!(
            "Message-ID: <id@localhost>\r",
            "Date: Thu, 1 Jan 1970 00:00:00 +0000\r",
            "MIME-Version: 1.0\r",
            "Content-Type: text/html; charset=\"utf-8\"\r",
            "Content-Transfer-Encoding: 7bit\r",
            "\r",
            "<h1>Hello, world!</h1>\r",
            "",
        );

        assert_eq!(msg, expected_msg);
    }

    #[test]
    fn attachment() {
        let dir = Builder::new().prefix("attachment").tempdir().unwrap();
        let attachment = dir.path().join("attachment.txt");
        fs::write(&attachment, "Hello, world!").unwrap();
        let attachment_path = attachment.display();

        let mml_body = format!(
            "<#part filename={attachment_path} type=text/plain name=custom recipient-filename=/tmp/custom encoding=base64>discarded body<#/part>"
        );

        let msg = MmlBodyCompiler::default()
            .compile(&mml_body)
            .unwrap()
            .message_id("id@localhost")
            .date(0_u64)
            .write_to_string()
            .unwrap();

        let expected_msg = concat_line!(
            "Message-ID: <id@localhost>\r",
            "Date: Thu, 1 Jan 1970 00:00:00 +0000\r",
            "MIME-Version: 1.0\r",
            "Content-Type: text/plain; name=\"custom\"\r",
            "Content-Transfer-Encoding: base64\r",
            "Content-Disposition: attachment; filename=\"/tmp/custom\"\r",
            "\r",
            "Hello, world!",
        );

        assert_eq!(msg, expected_msg);
    }
}
