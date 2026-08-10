//! `mml mailto`: RFC 6068 `mailto:` URI composer. Parses the URI into
//! the header set [`crate::cli::compose`] already accepts, then runs
//! that command, so a desktop handler is one `mml mailto "$1"` away.

use anyhow::{Result, bail};
use clap::Parser;
use log::debug;
use pimalaya_cli::printer::Printer;

use crate::{
    cli::{
        account::Account,
        args::{HeaderRawArgs, IdentityArgs, OutputArg},
        compose::ComposeCommand,
    },
    template::compose::MmlTemplateComposeSignatureStyle,
};

/// Headers a `mailto:` URI is allowed to set.
///
/// NOTE: RFC 6068 section 5 warns against honouring arbitrary headers
/// from an untrusted URI, so anything outside this list is dropped.
const SAFE_HEADERS: [&str; 5] = ["to", "cc", "bcc", "subject", "in-reply-to"];

/// Compose a message from a mailto: URI, using $EDITOR.
#[derive(Debug, Parser)]
pub struct MailtoCommand {
    /// The RFC 6068 URI, for example `mailto:you@example.org?subject=Hi`.
    #[arg(value_name = "URI")]
    pub uri: String,

    #[command(flatten)]
    pub identity: IdentityArgs,
    /// How to attach the signature (`inlined`, `attached`,
    /// `hidden`).
    #[arg(long, short = 'S')]
    pub signature_style: Option<MmlTemplateComposeSignatureStyle>,
}

impl MailtoCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        output: &OutputArg,
        account: Account,
    ) -> Result<()> {
        let (headers, body) = parse_uri(&self.uri)?;

        ComposeCommand {
            identity: self.identity,
            signature_style: self.signature_style,
            headers: HeaderRawArgs { raw: headers },
            body,
        }
        .execute(printer, output, account)
    }
}

/// Splits a `mailto:` URI into headers and body.
///
/// The path holds a comma-separated recipient list, merged into any
/// `to` from the query. Both sides are percent-decoded.
fn parse_uri(uri: &str) -> Result<(Vec<(String, String)>, String)> {
    let Some(rest) = uri.strip_prefix("mailto:").or(uri.strip_prefix("MAILTO:")) else {
        bail!("Cannot parse {uri:?}: expected a mailto: URI");
    };

    let (path, query) = match rest.split_once('?') {
        Some((path, query)) => (path, query),
        None => (rest, ""),
    };

    let mut recipients: Vec<String> = decode(path)?
        .split(',')
        .map(str::trim)
        .filter(|to| !to.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    let mut headers = Vec::new();
    let mut body = String::new();

    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, val) = pair.split_once('=').unwrap_or((pair, ""));
        let key = decode(key)?.to_lowercase();
        let val = decode(val)?;

        match key.as_str() {
            "body" => body = val,
            "to" => recipients.extend(
                val.split(',')
                    .map(str::trim)
                    .filter(|to| !to.is_empty())
                    .map(ToOwned::to_owned),
            ),
            key if SAFE_HEADERS.contains(&key) => headers.push((header_case(key), val)),
            key => debug!("skipping unsafe mailto header {key:?}"),
        }
    }

    if !recipients.is_empty() {
        headers.insert(0, (String::from("To"), recipients.join(", ")));
    }

    Ok((headers, body))
}

/// Percent-decodes a URI component.
///
/// NOTE: RFC 6068 has no form encoding, so `+` stays a literal plus.
fn decode(raw: &str) -> Result<String> {
    let mut bytes = Vec::with_capacity(raw.len());
    let mut chars = raw.bytes();

    while let Some(byte) = chars.next() {
        if byte != b'%' {
            bytes.push(byte);
            continue;
        }

        let hex: Vec<u8> = chars.by_ref().take(2).collect();
        let hex = str::from_utf8(&hex)?;

        let Ok(byte) = u8::from_str_radix(hex, 16) else {
            bail!("Cannot decode percent-escape %{hex} in {raw:?}");
        };

        bytes.push(byte);
    }

    Ok(String::from_utf8(bytes)?)
}

/// Capitalises a lowercase header name, `in-reply-to` becoming
/// `In-Reply-To`.
fn header_case(key: &str) -> String {
    key.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::parse_uri;

    #[test]
    fn recipients_from_path_and_query() {
        let (headers, body) = parse_uri("mailto:a@b.c,d@e.f?to=g@h.i&subject=Hi").unwrap();

        assert_eq!(
            headers,
            [
                (String::from("To"), String::from("a@b.c, d@e.f, g@h.i")),
                (String::from("Subject"), String::from("Hi")),
            ]
        );
        assert!(body.is_empty());
    }

    #[test]
    fn percent_decoding() {
        let (headers, body) = parse_uri("mailto:?subject=A%20%26%20B&body=line%0Aline").unwrap();

        assert_eq!(headers, [(String::from("Subject"), String::from("A & B"))]);
        assert_eq!(body, "line\nline");
    }

    #[test]
    fn plus_stays_literal() {
        let (headers, _) = parse_uri("mailto:a+tag@b.c").unwrap();

        assert_eq!(headers, [(String::from("To"), String::from("a+tag@b.c"))]);
    }

    #[test]
    fn unsafe_headers_are_dropped() {
        let (headers, _) = parse_uri("mailto:a@b.c?from=evil@z.z&cc=d@e.f").unwrap();

        assert_eq!(
            headers,
            [
                (String::from("To"), String::from("a@b.c")),
                (String::from("Cc"), String::from("d@e.f")),
            ]
        );
    }

    #[test]
    fn in_reply_to_is_header_cased() {
        let (headers, _) = parse_uri("mailto:a@b.c?in-reply-to=%3Cid%40host%3E").unwrap();

        assert_eq!(
            headers[1],
            (String::from("In-Reply-To"), String::from("<id@host>"))
        );
    }

    #[test]
    fn non_mailto_uri_fails() {
        assert!(parse_uri("https://example.org").is_err());
    }
}
