//! # Message body module
//!
//! A MML body can be compiled into a MIME body using the
//! [MmlBodyCompiler] builder. A MIME body can be interpreted as a MML
//! body using the [MimeBodyInterpreter] builder.

#[cfg(feature = "compiler")]
pub mod compiler;
#[cfg(feature = "interpreter")]
pub mod interpreter;

pub(crate) const PART_BEGIN: &str = "<#part";
pub(crate) const PART_BEGIN_ESCAPED: &str = "<#!part";
pub(crate) const PART_END: &str = "<#/part>";
pub(crate) const PART_END_ESCAPED: &str = "<#!/part>";

pub(crate) const MULTIPART_BEGIN: &str = "<#multipart";
pub(crate) const MULTIPART_BEGIN_ESCAPED: &str = "<#!multipart";
pub(crate) const MULTIPART_END: &str = "<#/multipart>";
pub(crate) const MULTIPART_END_ESCAPED: &str = "<#!/multipart>";

pub(crate) const ALTERNATIVE: &str = "alternative";
pub(crate) const ATTACHMENT: &str = "attachment";
pub(crate) const CREATION_DATE: &str = "creation-date";
pub(crate) const DATA_ENCODING: &str = "data-encoding";
pub(crate) const DESCRIPTION: &str = "description";
pub(crate) const DISPOSITION: &str = "disposition";
pub(crate) const ENCODING: &str = "encoding";
pub(crate) const ENCODING_7BIT: &str = "7bit";
pub(crate) const ENCODING_8BIT: &str = "8bit";
pub(crate) const ENCODING_BASE64: &str = "base64";
pub(crate) const ENCODING_QUOTED_PRINTABLE: &str = "quoted-printable";
pub(crate) const FILENAME: &str = "filename";
pub(crate) const INLINE: &str = "inline";
pub(crate) const MIXED: &str = "mixed";
pub(crate) const MODIFICATION_DATE: &str = "modification-date";
pub(crate) const NAME: &str = "name";
pub(crate) const READ_DATE: &str = "read-date";
pub(crate) const RECIPIENT_FILENAME: &str = "recipient-filename";
pub(crate) const RELATED: &str = "related";
pub(crate) const TYPE: &str = "type";

pub(crate) const BACKSLASH: char = '\\';
pub(crate) const DOUBLE_QUOTE: char = '"';
pub(crate) const GREATER_THAN: char = '>';
pub(crate) const NEW_LINE: char = '\n';
pub(crate) const SPACE: char = ' ';
