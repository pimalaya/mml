//! Shared MML grammar primitives: directive markers, attribute
//! keys/values, and character literals used by both the
//! [`crate::compiler`] and [`crate::interpreter`] pipelines.

pub(crate) const PART_BEGIN: &str = "<#part";
pub(crate) const PART_BEGIN_ESCAPED: &str = "<#!part";
pub(crate) const PART_END: &str = "<#/part>";
pub(crate) const PART_END_ESCAPED: &str = "<#!/part>";

pub(crate) const MULTIPART_BEGIN: &str = "<#multipart";
pub(crate) const MULTIPART_BEGIN_ESCAPED: &str = "<#!multipart";
pub(crate) const MULTIPART_END: &str = "<#/multipart>";
pub(crate) const MULTIPART_END_ESCAPED: &str = "<#!/multipart>";

#[cfg(feature = "compiler")]
pub(crate) const ALTERNATIVE: &str = "alternative";
#[cfg(feature = "compiler")]
pub(crate) const ATTACHMENT: &str = "attachment";
#[cfg(feature = "compiler")]
pub(crate) const CREATION_DATE: &str = "creation-date";
#[cfg(feature = "compiler")]
pub(crate) const DATA_ENCODING: &str = "data-encoding";
#[cfg(feature = "compiler")]
pub(crate) const DESCRIPTION: &str = "description";
#[cfg(feature = "compiler")]
pub(crate) const DISPOSITION: &str = "disposition";
#[cfg(feature = "compiler")]
pub(crate) const ENCODING: &str = "encoding";
#[cfg(feature = "compiler")]
pub(crate) const ENCODING_7BIT: &str = "7bit";
#[cfg(feature = "compiler")]
pub(crate) const ENCODING_8BIT: &str = "8bit";
#[cfg(feature = "compiler")]
pub(crate) const ENCODING_BASE64: &str = "base64";
#[cfg(feature = "compiler")]
pub(crate) const ENCODING_QUOTED_PRINTABLE: &str = "quoted-printable";
#[cfg(feature = "compiler")]
pub(crate) const FILENAME: &str = "filename";
#[cfg(feature = "compiler")]
pub(crate) const INLINE: &str = "inline";
#[cfg(feature = "compiler")]
pub(crate) const MIXED: &str = "mixed";
#[cfg(feature = "compiler")]
pub(crate) const MODIFICATION_DATE: &str = "modification-date";
#[cfg(feature = "compiler")]
pub(crate) const NAME: &str = "name";
#[cfg(feature = "compiler")]
pub(crate) const READ_DATE: &str = "read-date";
#[cfg(feature = "compiler")]
pub(crate) const RECIPIENT_FILENAME: &str = "recipient-filename";
#[cfg(feature = "compiler")]
pub(crate) const RELATED: &str = "related";
#[cfg(feature = "compiler")]
pub(crate) const TYPE: &str = "type";

#[cfg(feature = "compiler")]
pub(crate) const BACKSLASH: char = '\\';
#[cfg(feature = "compiler")]
pub(crate) const DOUBLE_QUOTE: char = '"';
#[cfg(feature = "compiler")]
pub(crate) const GREATER_THAN: char = '>';
#[cfg(feature = "compiler")]
pub(crate) const NEW_LINE: char = '\n';
#[cfg(feature = "compiler")]
pub(crate) const SPACE: char = ' ';
