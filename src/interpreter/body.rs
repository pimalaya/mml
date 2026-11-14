//! MIME → MML body interpretation: walks MIME parts, emits MML
//! directives, optionally renders HTML to text via [`nanohtml2text`]
//! and saves attachments to disk.

use std::{env, fs, path::PathBuf};

use log::debug;
use mail_builder::MessageBuilder;
use mail_parser::{Message, MessageParser, MessagePart, MimeHeaders, PartType};
use nanohtml2text::html2text;

use crate::{
    error::{MmlError, Result},
    grammar::{
        DELSP, FLOWED, FORMAT, MULTIPART_BEGIN, MULTIPART_BEGIN_ESCAPED, MULTIPART_END,
        MULTIPART_END_ESCAPED, PART_BEGIN, PART_BEGIN_ESCAPED, PART_END, PART_END_ESCAPED, YES,
    },
    interpreter::flowed::decode,
};

/// Filters parts to show by MIME type.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MmlFilterParts {
    /// Show all parts. This filter enables MML markup since multiple
    /// parts with different MIME types can be mixed together, which
    /// can be hard to navigate through.
    #[default]
    All,

    /// Show only parts matching the given MIME type. This filter
    /// disables MML markup since only one MIME type is shown.
    Only(String),

    /// Show only parts matching the given list of MIME types. This
    /// filter enables MML markup since multiple parts with different
    /// MIME types can be mixed together, which can be hard to
    /// navigate through.
    Include(Vec<String>),

    /// Show all parts except those matching the given list of MIME
    /// types. This filter enables MML markup since multiple parts
    /// with different MIME types can be mixed together, which can be
    /// hard to navigate through.
    Exclude(Vec<String>),
}

impl MmlFilterParts {
    /// Whether the filter shows this MIME type exclusively.
    pub fn only(&self, that: impl AsRef<str>) -> bool {
        match self {
            Self::All => false,
            Self::Only(this) => this == that.as_ref(),
            Self::Include(_) => false,
            Self::Exclude(_) => false,
        }
    }

    /// Whether the filter shows the given MIME type.
    pub fn contains(&self, that: impl ToString + AsRef<str>) -> bool {
        match self {
            Self::All => true,
            Self::Only(this) => this == that.as_ref(),
            Self::Include(this) => this.contains(&that.to_string()),
            Self::Exclude(this) => !this.contains(&that.to_string()),
        }
    }
}

/// MIME → MML message body interpreter.
///
/// Every option is a public field: build it with a struct literal and
/// `..Default::default()`, then call one of the `interpret_*` methods.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MmlMimeBodyInterpreter {
    /// Defines visibility of the multipart markup `<#multipart>`.
    ///
    /// When `true`, multipart markup is visible. This is useful when
    /// you need to see multiparts nested structure.
    ///
    /// When `false`, multipart markup is hidden. The structure is
    /// flatten, which means all parts and subparts are shown at the
    /// same top level.
    ///
    /// This option shows or hides the multipart markup, not their
    /// content. The content is always shown. To filter parts with
    /// their content, see [`MmlMimeBodyInterpreter::filter_parts`] and
    /// [`MmlFilterParts`].
    pub show_multiparts: bool,

    /// Defines visibility of the part markup `<#part>`.
    ///
    /// When `true`, part markup is visible. This is useful when you
    /// want to get more information about parts being interpreted
    /// (MIME type, description etc).
    ///
    /// When `false`, part markup is hidden. Only the content is
    /// shown.
    ///
    /// This option shows or hides the part markup, not their
    /// content. The content is always shown. To filter parts with
    /// their content, see [`MmlMimeBodyInterpreter::filter_parts`] and
    /// [`MmlFilterParts`].
    pub show_parts: bool,

    /// Defines visibility of the part markup `<#part
    /// disposition=attachment>`.
    ///
    /// This option is dedicated to attachment parts, and it overrides
    /// [`Self::show_parts`].
    pub show_attachments: bool,

    /// Defines visibility of the part markup `<#part
    /// disposition=inline>`.
    ///
    /// This option is dedicated to inline attachment parts, and it
    /// overrides [`Self::show_parts`].
    pub show_inline_attachments: bool,

    /// Defines parts visibility.
    ///
    /// This option filters parts to show or hide by their MIME
    /// type. If you want to show or hide MML markup instead, see
    /// [`Self::show_multiparts`], [`Self::show_parts`],
    /// [`Self::show_attachments`] and
    /// [`Self::show_inline_attachments`].
    pub filter_parts: MmlFilterParts,

    /// Defines visibility of signatures in `text/plain` parts.
    ///
    /// When `false`, this option tries to remove signatures from
    /// plain text parts starting by the standard delimiter `-- \n`.
    pub show_plain_texts_signature: bool,

    /// Defines the saving strategy of attachments content.
    ///
    /// An attachment is interpreted this way: `<#part
    /// filename=attachment.ext>`.
    ///
    /// When `true`, the file (with its content) is automatically
    /// created at the given filename. Directory can be customized via
    /// [`Self::save_attachments_dir`]. This option is particularly
    /// useful when transferring a message with its attachments.
    pub save_attachments: bool,

    /// Defines the directory for [`Self::save_attachments`] strategy.
    ///
    /// This option saves attachments to the given directory instead
    /// of the default temporary one given by
    /// [`std::env::temp_dir()`].
    pub save_attachments_dir: PathBuf,
}

impl Default for MmlMimeBodyInterpreter {
    fn default() -> Self {
        Self {
            show_multiparts: false,
            show_parts: true,
            show_attachments: true,
            show_inline_attachments: true,
            filter_parts: Default::default(),
            show_plain_texts_signature: true,
            save_attachments: Default::default(),
            save_attachments_dir: Self::default_save_attachments_dir(),
        }
    }
}

impl MmlMimeBodyInterpreter {
    /// The fallback attachment directory, the system temp directory.
    pub fn default_save_attachments_dir() -> PathBuf {
        env::temp_dir()
    }

    /// Replace normal opening and closing tags by escaped opening and
    /// closing tags.
    fn escape_mml_markup(text: String) -> String {
        text.replace(PART_BEGIN, PART_BEGIN_ESCAPED)
            .replace(PART_END, PART_END_ESCAPED)
            .replace(MULTIPART_BEGIN, MULTIPART_BEGIN_ESCAPED)
            .replace(MULTIPART_END, MULTIPART_END_ESCAPED)
    }

    fn interpret_attachment(&self, ctype: &str, part: &MessagePart, data: &[u8]) -> Result<String> {
        let mut tpl = String::new();

        if self.show_attachments && self.filter_parts.contains(ctype) {
            let fname = self
                .save_attachments_dir
                .join(part.attachment_name().unwrap_or("noname"));

            if self.save_attachments {
                fs::write(&fname, data)
                    .map_err(|err| MmlError::WriteAttachmentError(err, fname.clone()))?;
            }

            let fname = fname.to_string_lossy();
            tpl = format!("<#part type={ctype} filename=\"{fname}\"><#/part>\n");
        }

        Ok(tpl)
    }

    fn interpret_inline_attachment(
        &self,
        ctype: &str,
        part: &MessagePart,
        data: &[u8],
    ) -> Result<String> {
        let mut tpl = String::new();

        if self.show_inline_attachments && self.filter_parts.contains(ctype) {
            let ctype = get_ctype(part);
            let fname = self.save_attachments_dir.join(
                part.attachment_name()
                    .or(part.content_id())
                    .unwrap_or("noname"),
            );

            if self.save_attachments {
                fs::write(&fname, data)
                    .map_err(|err| MmlError::WriteAttachmentError(err, fname.clone()))?;
            }

            let fname = fname.to_string_lossy();
            tpl = format!("<#part type={ctype} disposition=inline filename=\"{fname}\"><#/part>\n");
        }

        Ok(tpl)
    }

    fn interpret_text(&self, ctype: &str, text: &str) -> String {
        let mut tpl = String::new();

        if self.filter_parts.contains(ctype) {
            let text = text.replace('\r', "");
            let text = Self::escape_mml_markup(text);

            if !self.show_parts || self.filter_parts.only(ctype) {
                tpl.push_str(&text);
            } else {
                tpl.push_str(&format!("<#part type={ctype}>\n"));
                tpl.push_str(&text);
                tpl.push_str("<#/part>\n");
            }
        }

        tpl
    }

    fn interpret_text_plain(&self, part: &MessagePart, plain: &str) -> String {
        let mut tpl = String::new();

        if self.filter_parts.contains("text/plain") {
            let plain = plain.replace('\r', "");

            // NOTE: unflowed first, so the markup escaping and the
            // signature stripping below both see logical lines.
            let plain = match content_type_attribute(part, FORMAT) {
                Some(format) if format.eq_ignore_ascii_case(FLOWED) => {
                    let delsp = content_type_attribute(part, DELSP)
                        .is_some_and(|delsp| delsp.eq_ignore_ascii_case(YES));
                    decode(&plain, delsp)
                }
                _ => plain,
            };

            let mut plain = Self::escape_mml_markup(plain);

            if !self.show_plain_texts_signature {
                plain = plain
                    .rsplit_once("-- \n")
                    .map(|(body, _signature)| body.to_owned())
                    .unwrap_or(plain);
            }

            tpl.push_str(&plain);
        }

        tpl
    }

    fn interpret_text_html(&self, html: &str) -> String {
        let mut tpl = String::new();

        if self.filter_parts.contains("text/html") {
            if self.filter_parts.only("text/html") {
                let html = html.replace('\r', "");
                let html = Self::escape_mml_markup(html);
                tpl.push_str(&html);
            } else {
                let html = html2text(html);
                let html = Self::escape_mml_markup(html);

                if self.show_parts {
                    tpl.push_str("<#part type=text/html>\n");
                }

                tpl.push_str(&html);

                if self.show_parts {
                    tpl.push_str("<#/part>\n");
                }
            }
        }

        tpl
    }

    fn interpret_part(
        &self,
        msg: &Message<'_>,
        part: &MessagePart<'_>,
        pgp: &mut Vec<MmlMimePgpRequest>,
    ) -> Result<String> {
        let mut tpl = String::new();
        let ctype = get_ctype(part);

        match &part.body {
            PartType::Text(plain) if ctype == "text/plain" => {
                tpl.push_str(&self.interpret_text_plain(part, plain));
            }
            PartType::Text(text) => {
                tpl.push_str(&self.interpret_text(&ctype, text));
            }
            PartType::Html(html) => {
                tpl.push_str(&self.interpret_text_html(html));
            }
            PartType::Binary(data) => {
                tpl.push_str(&self.interpret_attachment(&ctype, part, data)?);
            }
            PartType::InlineBinary(data) => {
                tpl.push_str(&self.interpret_inline_attachment(&ctype, part, data)?);
            }
            PartType::Message(msg) => {
                tpl.push_str(&self.interpret_part(msg, msg.root_part(), pgp)?);
            }
            // NOTE: RFC 3156 section 4 pins the layout: a
            // `application/pgp-encrypted` version part, then the
            // OpenPGP message. Only the second one is decrypted.
            PartType::Multipart(ids) if ctype == "multipart/encrypted" => {
                let Some(encrypted) = ids.get(1).and_then(|id| msg.part(*id)) else {
                    debug!("malformed multipart/encrypted, skipping it");
                    return Ok(tpl);
                };

                tpl.push_str(&placeholder(pgp.len()));
                pgp.push(MmlMimePgpRequest::Decrypt(encrypted.contents().to_vec()));
            }
            // NOTE: RFC 3156 section 5 pins the layout: the signed
            // part, then its detached `application/pgp-signature`. The
            // signed part renders as usual; the placeholder only
            // carries whatever the caller makes of the verdict.
            PartType::Multipart(ids) if ctype == "multipart/signed" => {
                let signed = ids.first().and_then(|id| msg.part(*id));
                let signature = ids.get(1).and_then(|id| msg.part(*id));

                let (Some(signed), Some(signature)) = (signed, signature) else {
                    debug!("malformed multipart/signed, skipping it");
                    return Ok(tpl);
                };

                tpl.push_str(&placeholder(pgp.len()));
                pgp.push(MmlMimePgpRequest::Verify {
                    signed: msg
                        .raw_message()
                        .get(signed.offset_header as usize..signed.offset_end as usize)
                        .unwrap_or_default()
                        .to_vec(),
                    signature: signature.contents().to_vec(),
                });

                tpl.push_str(&self.interpret_part(msg, signed, pgp)?);
            }
            PartType::Multipart(ids) if ctype == "multipart/alternative" => {
                let mut parts = ids.iter().filter_map(|id| msg.part(*id));

                let part = match &self.filter_parts {
                    MmlFilterParts::All => {
                        let part = parts
                            .clone()
                            .find_map(|part| match &part.body {
                                PartType::Text(plain)
                                    if is_plain(part) && !plain.trim().is_empty() =>
                                {
                                    Some(Ok(self.interpret_text_plain(part, plain)))
                                }
                                _ => None,
                            })
                            .or_else(|| {
                                parts.clone().find_map(|part| match &part.body {
                                    PartType::Html(html) if !html.trim().is_empty() => {
                                        Some(Ok(self.interpret_text_html(html)))
                                    }
                                    _ => None,
                                })
                            })
                            .or_else(|| {
                                parts.clone().find_map(|part| {
                                    let ctype = get_ctype(part);
                                    match &part.body {
                                        PartType::Text(text) if !text.trim().is_empty() => {
                                            Some(Ok(self.interpret_text(&ctype, text)))
                                        }
                                        _ => None,
                                    }
                                })
                            });

                        match part {
                            Some(part) => Some(part),
                            None => parts.next().map(|part| self.interpret_part(msg, part, pgp)),
                        }
                    }
                    MmlFilterParts::Only(ctype) => parts
                        .clone()
                        .find(|part| get_ctype(part).starts_with(ctype))
                        .map(|part| self.interpret_part(msg, part, pgp)),
                    MmlFilterParts::Include(ctypes) => parts
                        .clone()
                        .find(|part| ctypes.contains(&get_ctype(part)))
                        .map(|part| self.interpret_part(msg, part, pgp)),
                    MmlFilterParts::Exclude(ctypes) => parts
                        .clone()
                        .find(|part| !ctypes.contains(&get_ctype(part)))
                        .map(|part| self.interpret_part(msg, part, pgp)),
                };

                if let Some(part) = part {
                    tpl.push_str(&part?);
                }
            }
            PartType::Multipart(ids) => {
                if self.show_multiparts {
                    let stype = part
                        .content_type()
                        .and_then(|p| p.subtype())
                        .unwrap_or("mixed");
                    tpl.push_str(&format!("<#multipart type={stype}>\n"));
                }

                for id in ids {
                    if let Some(part) = msg.part(*id) {
                        tpl.push_str(&self.interpret_part(msg, part, pgp)?);
                    } else {
                        debug!("cannot find part {id}, skipping it");
                    }
                }

                if self.show_multiparts {
                    tpl.push_str("<#/multipart>\n");
                }
            }
        }

        Ok(tpl)
    }

    /// Interpret the given MIME [Message] as a MML message string.
    pub fn interpret_msg<'a>(&self, msg: &Message<'a>) -> Result<String> {
        self.interpret_part(msg, msg.root_part(), &mut Vec::new())
    }

    /// Interprets a message, collecting the PGP requests its
    /// encrypted parts need served.
    pub(crate) fn interpret_msg_pgp<'a>(
        &self,
        msg: &Message<'a>,
        pgp: &mut Vec<MmlMimePgpRequest>,
    ) -> Result<String> {
        self.interpret_part(msg, msg.root_part(), pgp)
    }

    /// Interpret the given MIME message bytes as a MML message
    /// string.
    pub fn interpret_bytes<'a>(&self, bytes: impl AsRef<[u8]> + 'a) -> Result<String> {
        let msg = MessageParser::new()
            .parse(bytes.as_ref())
            .ok_or(MmlError::ParseMimeMessageError)?;
        self.interpret_msg(&msg)
    }

    /// Interpret the given MIME [MessageBuilder] as a MML message
    /// string.
    pub fn interpret_msg_builder<'a>(&self, builder: MessageBuilder<'a>) -> Result<String> {
        let bytes = builder
            .write_to_vec()
            .map_err(MmlError::WriteMessageError)?;
        self.interpret_bytes(&bytes)
    }
}

fn get_ctype(part: &MessagePart) -> String {
    part.content_type()
        .and_then(|ctype| {
            ctype
                .subtype()
                .map(|stype| format!("{}/{stype}", ctype.ctype()))
        })
        .unwrap_or_else(|| String::from("application/octet-stream"))
}

/// The given content type parameter of the part, if any.
fn content_type_attribute<'a>(part: &'a MessagePart, name: &str) -> Option<&'a str> {
    part.content_type()?.attribute(name)
}

fn is_plain(part: &MessagePart) -> bool {
    get_ctype(part) == "text/plain"
}

#[cfg(test)]
mod tests {
    use concat_with::concat_line;
    use mail_builder::{MessageBuilder, mime::MimePart};

    use crate::interpreter::body::{MmlFilterParts, MmlMimeBodyInterpreter};

    #[test]
    fn nested_multiparts() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/mixed",
            vec![
                MimePart::new("text/plain", "This is a plain text part.\n"),
                MimePart::new(
                    "multipart/related",
                    vec![
                        MimePart::new("text/plain", "\nThis is a second plain text part.\n\n"),
                        MimePart::new("text/plain", "This is a third plain text part.\n\n\n"),
                    ],
                ),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter::default()
            .interpret_msg_builder(builder.clone())
            .unwrap();

        let expected_tpl = concat_line!(
            "This is a plain text part.",
            "",
            "This is a second plain text part.",
            "",
            "This is a third plain text part.",
            "",
            "",
            "",
        );

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn nested_multiparts_with_markup() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/mixed",
            vec![
                MimePart::new("text/plain", "This is a plain text part.\n\n"),
                MimePart::new(
                    "multipart/related",
                    vec![
                        MimePart::new("text/plain", "This is a second plain text part.\n\n"),
                        MimePart::new("text/plain", "This is a third plain text part.\n\n"),
                    ],
                ),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter {
            show_multiparts: true,
            ..Default::default()
        }
        .interpret_msg_builder(builder.clone())
        .unwrap();

        let expected_tpl = concat_line!(
            "<#multipart type=mixed>",
            "This is a plain text part.",
            "",
            "<#multipart type=related>",
            "This is a second plain text part.",
            "",
            "This is a third plain text part.",
            "",
            "<#/multipart>",
            "<#/multipart>",
            "",
        );

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn all_text() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/mixed",
            vec![
                MimePart::new("text/plain", "This is a plain text part.\n\n"),
                MimePart::new("text/html", "<h1>This is a &lt;HTML&gt; text part.</h1>\n"),
                MimePart::new("text/json", "{\"type\": \"This is a JSON text part.\"}\n"),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter::default()
            .interpret_msg_builder(builder.clone())
            .unwrap();

        let expected_tpl = concat_line!(
            "This is a plain text part.",
            "",
            "<#part type=text/html>",
            "This is a <HTML> text part.\r",
            "\r",
            "<#/part>",
            "<#part type=text/json>",
            "{\"type\": \"This is a JSON text part.\"}",
            "<#/part>",
            "",
        );

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn only_text_plain() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/mixed",
            vec![
                MimePart::new("text/plain", "This is a plain text part.\n"),
                MimePart::new(
                    "text/html",
                    "<h1>This is a &lt;HTML&gt; text&nbsp;part.</h1>\n",
                ),
                MimePart::new("text/json", "{\"type\": \"This is a JSON text part.\"}\n"),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter {
            filter_parts: MmlFilterParts::Only("text/plain".into()),
            ..Default::default()
        }
        .interpret_msg_builder(builder.clone())
        .unwrap();

        let expected_tpl = concat_line!("This is a plain text part.", "");

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn only_text_html() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/mixed",
            vec![
                MimePart::new("text/plain", "This is a plain text part.\n"),
                MimePart::new(
                    "text/html",
                    "<h1>This is a &lt;HTML&gt; text&nbsp;part.</h1>\n",
                ),
                MimePart::new("text/json", "{\"type\": \"This is a JSON text part.\"}\n"),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter {
            filter_parts: MmlFilterParts::Only("text/html".into()),
            ..Default::default()
        }
        .interpret_msg_builder(builder.clone())
        .unwrap();

        let expected_tpl = concat_line!("<h1>This is a &lt;HTML&gt; text&nbsp;part.</h1>", "");

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn only_text_other() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/mixed",
            vec![
                MimePart::new("text/plain", "This is a plain text part.\n"),
                MimePart::new(
                    "text/html",
                    "<h1>This is a &lt;HTML&gt; text&nbsp;part.</h1>\n",
                ),
                MimePart::new("text/json", "{\"type\": \"This is a JSON text part.\"}\n"),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter {
            filter_parts: MmlFilterParts::Only("text/json".into()),
            ..Default::default()
        }
        .interpret_msg_builder(builder.clone())
        .unwrap();

        let expected_tpl = concat_line!("{\"type\": \"This is a JSON text part.\"}", "");

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn multipart_alternative_text_all_without_plain() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/alternative",
            vec![
                MimePart::new("text/html", "<h1>This is a &lt;HTML&gt; text part.</h1>\n"),
                MimePart::new("text/json", "{\"type\": \"This is a JSON text part.\"}\n"),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter::default()
            .interpret_msg_builder(builder.clone())
            .unwrap();

        let expected_tpl = concat_line!(
            "<#part type=text/html>",
            "This is a <HTML> text part.\r",
            "\r",
            "<#/part>",
            ""
        );

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn multipart_alternative_text_all_with_empty_plain() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/alternative",
            vec![
                MimePart::new("text/plain", "    \n\n"),
                MimePart::new("text/html", "<h1>This is a &lt;HTML&gt; text part.</h1>\n"),
                MimePart::new("text/json", "{\"type\": \"This is a JSON text part.\"}\n"),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter::default()
            .interpret_msg_builder(builder.clone())
            .unwrap();

        let expected_tpl = concat_line!(
            "<#part type=text/html>",
            "This is a <HTML> text part.\r",
            "\r",
            "<#/part>",
            ""
        );

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn multipart_alternative_text_all_without_plain_nor_html() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/alternative",
            vec![MimePart::new(
                "text/json",
                "{\"type\": \"This is a JSON text part.\"}\n",
            )],
        ));

        let tpl = MmlMimeBodyInterpreter::default()
            .interpret_msg_builder(builder.clone())
            .unwrap();

        let expected_tpl = concat_line!(
            "<#part type=text/json>",
            "{\"type\": \"This is a JSON text part.\"}",
            "<#/part>",
            ""
        );

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn multipart_alternative_text_all() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/alternative",
            vec![
                MimePart::new("text/plain", "This is a plain text part.\n"),
                MimePart::new(
                    "text/html",
                    "<h1>This is a &lt;HTML&gt; text&nbsp;part.</h1>\n",
                ),
                MimePart::new("text/json", "{\"type\": \"This is a JSON text part.\"}\n"),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter::default()
            .interpret_msg_builder(builder.clone())
            .unwrap();

        let expected_tpl = concat_line!("This is a plain text part.", "");

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn multipart_alternative_text_html_only() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/alternative",
            vec![
                MimePart::new("text/plain", "This is a plain text part.\n"),
                MimePart::new(
                    "text/html",
                    "<h1>This is a &lt;HTML&gt; text&nbsp;part.</h1>\n",
                ),
                MimePart::new("text/json", "{\"type\": \"This is a JSON text part.\"}\n"),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter {
            filter_parts: MmlFilterParts::Only("text/html".into()),
            ..Default::default()
        }
        .interpret_msg_builder(builder.clone())
        .unwrap();

        let expected_tpl = concat_line!("<h1>This is a &lt;HTML&gt; text&nbsp;part.</h1>", "");

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn attachment() {
        let builder = MessageBuilder::new().attachment(
            "application/octet-stream",
            "attachment.txt",
            "Hello, world!".as_bytes(),
        );

        let tpl = MmlMimeBodyInterpreter {
            save_attachments_dir: "~/Downloads".into(),
            ..Default::default()
        }
        .interpret_msg_builder(builder)
        .unwrap();

        let expected_tpl = concat_line!(
            "<#part type=application/octet-stream filename=\"~/Downloads/attachment.txt\"><#/part>",
            "",
        );

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn hide_parts_single_html() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "text/html",
            "<h1>This is a &lt;HTML&gt; text part.</h1>\n",
        ));

        let tpl = MmlMimeBodyInterpreter {
            show_parts: false,
            ..Default::default()
        }
        .interpret_msg_builder(builder.clone())
        .unwrap();

        let expected_tpl = concat_line!("This is a <HTML> text part.\r", "\r", "");

        assert_eq!(tpl, expected_tpl);
    }

    #[test]
    fn hide_parts_multipart_mixed() {
        let builder = MessageBuilder::new().body(MimePart::new(
            "multipart/mixed",
            vec![
                MimePart::new("text/plain", "This is a plain text part.\n"),
                MimePart::new("text/html", "<h1>This is a &lt;HTML&gt; text part.</h1>\n"),
                MimePart::new("text/json", "{\"type\": \"This is a JSON text part.\"}\n"),
            ],
        ));

        let tpl = MmlMimeBodyInterpreter {
            show_parts: false,
            filter_parts: MmlFilterParts::Include(vec!["text/plain".into(), "text/html".into()]),
            ..Default::default()
        }
        .interpret_msg_builder(builder.clone())
        .unwrap();

        let expected_tpl = concat_line!(
            "This is a plain text part.",
            "This is a <HTML> text part.\r",
            "\r",
            "",
        );

        assert_eq!(tpl, expected_tpl);
    }
}

/// Opens a placeholder marking where a decrypted layer goes.
///
/// NOTE: a control character, which MML markup never contains, so a
/// placeholder can never collide with rendered content.
pub(crate) const PLACEHOLDER_BEGIN: char = '\u{1}';

/// Closes a placeholder, see [`PLACEHOLDER_BEGIN`].
pub(crate) const PLACEHOLDER_END: char = '\u{2}';

/// Marks where the decrypted layer of the given request goes.
pub(crate) fn placeholder(index: usize) -> String {
    format!("{PLACEHOLDER_BEGIN}{index}{PLACEHOLDER_END}")
}

/// A PGP request the walk found, resolved by
/// [`crate::interpreter::layer`].
///
/// NOTE: internal. The public shape is the inlined `WantsPgp*`
/// variants of [`crate::coroutine::MmlYield`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MmlMimePgpRequest {
    /// An OpenPGP message to decrypt.
    Decrypt(Vec<u8>),
    /// A detached signature to check against the bytes it covers.
    Verify {
        /// The exact bytes covered by the signature.
        signed: Vec<u8>,
        /// The detached `application/pgp-signature` payload.
        signature: Vec<u8>,
    },
}

/// What an unresolved encrypted part renders as.
///
/// NOTE: shared by the coroutine answering `Unavailable` and by the
/// PGP-free path, so both degrade the same visible way.
pub(crate) const UNAVAILABLE_PART: &str = "<#part type=application/pgp-encrypted><#/part>\n";

/// Renders every placeholder as [`UNAVAILABLE_PART`].
///
/// NOTE: the PGP-free path collects no request, so its placeholders
/// would otherwise leak control characters into the output.
pub(crate) fn resolve_unavailable(mml: &str) -> String {
    let mut resolved = String::with_capacity(mml.len());

    for (i, chunk) in mml.split(PLACEHOLDER_BEGIN).enumerate() {
        if i == 0 {
            resolved.push_str(chunk);
            continue;
        }

        match chunk.split_once(PLACEHOLDER_END) {
            Some((_, rest)) => {
                resolved.push_str(UNAVAILABLE_PART);
                resolved.push_str(rest);
            }
            None => resolved.push_str(chunk),
        }
    }

    resolved
}
