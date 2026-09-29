//! RFC 3676 `format=flowed` decoding of a `text/plain` body.
//!
//! Soft breaks are joined back into logical lines, one per paragraph,
//! which is what the compiler takes as input.

/// The Usenet signature separator, the one line allowed to end in a
/// space without being soft (RFC 3676 section 4.3).
const SIGNATURE_SEPARATOR: &str = "-- ";

/// Decodes the given `format=flowed` body, deleting the marking space
/// of each soft break when `delsp` is set (section 4.2).
pub(crate) fn decode(text: &str, delsp: bool) -> String {
    let mut lines = Vec::new();
    let mut pending: Option<(usize, String)> = None;

    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let depth = line.chars().take_while(|c| *c == '>').count();
        let content = &line[depth..];
        let content = content.strip_prefix(' ').unwrap_or(content);
        let separator = content == SIGNATURE_SEPARATOR;

        // NOTE: a depth change and the signature separator both end
        // the paragraph, so a quote or a signature never absorbs the
        // text around it (section 4.5).
        let (depth, mut joined) = match pending.take() {
            Some((d, mut joined)) if d == depth && !separator => {
                joined.push_str(content);
                (depth, joined)
            }
            Some((d, joined)) => {
                lines.push(unquote(d, &joined));
                (depth, content.to_owned())
            }
            None => (depth, content.to_owned()),
        };

        if content.ends_with(' ') && !separator {
            if delsp {
                joined.pop();
            }

            pending = Some((depth, joined));
        } else {
            lines.push(unquote(depth, &joined));
        }
    }

    if let Some((depth, joined)) = pending {
        lines.push(unquote(depth, &joined));
    }

    lines.join("\n")
}

/// Renders a logical line with its quote marks, a space separating
/// them from any text.
fn unquote(depth: usize, content: &str) -> String {
    let mut line = ">".repeat(depth);

    if depth > 0 && !content.is_empty() {
        line.push(' ');
    }

    line.push_str(content);
    line
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn soft_breaks_are_joined() {
        assert_eq!(
            decode("aaaa bbbb \ncccc dddd\n", false),
            "aaaa bbbb cccc dddd\n"
        );
    }

    #[test]
    fn hard_breaks_are_kept() {
        let text = "Hello,\n\nA short line.\n";
        assert_eq!(decode(text, false), text);
    }

    #[test]
    fn delsp_deletes_the_marking_space() {
        assert_eq!(decode("aaaa \nbbbb", true), "aaaabbbb");
    }

    #[test]
    fn stuffing_is_removed() {
        assert_eq!(
            decode("  indented\n From here", false),
            " indented\nFrom here"
        );
    }

    #[test]
    fn quoted_paragraph_is_joined_at_its_depth() {
        let text = ">> aaaa \n>> bbbb\n> cccc";
        assert_eq!(decode(text, false), ">> aaaa bbbb\n> cccc");
    }

    #[test]
    fn depth_change_ends_the_paragraph() {
        assert_eq!(decode("> aaaa \nbbbb", false), "> aaaa \nbbbb");
    }

    #[test]
    fn signature_separator_stays_hard() {
        let text = "body \n-- \nAlice";
        assert_eq!(decode(text, false), "body \n-- \nAlice");
    }

    #[test]
    fn crlf_input_is_normalised() {
        assert_eq!(decode("aaaa \r\nbbbb\r\n", false), "aaaa bbbb\n");
    }
}
