//! RFC 3676 `format=flowed` encoding of a `text/plain` body.
//!
//! Every newline of the input is a hard break the author typed. Lines
//! longer than the column are broken on spaces, each broken line
//! ending with the space that marks it soft, so a reader can join
//! them back. Nothing shorter is ever touched.

/// The Usenet signature separator, the one line allowed to end in a
/// space without being soft (RFC 3676 section 4.3).
const SIGNATURE_SEPARATOR: &str = "-- ";

/// Encodes the given body as `format=flowed`, breaking lines longer
/// than `column` characters, quote prefix included.
pub(crate) fn encode(text: &str, column: usize) -> String {
    let mut lines = Vec::new();

    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        encode_line(line, column, &mut lines);
    }

    lines.join("\n")
}

/// Encodes one logical line into one or more flowed lines.
fn encode_line(line: &str, column: usize, lines: &mut Vec<String>) {
    let depth = line.chars().take_while(|c| *c == '>').count();
    let content = &line[depth..];

    // NOTE: the space after the quote marks separates them from the
    // text, it is not part of it (section 4.5).
    let content = match depth {
        0 => content,
        _ => content.strip_prefix(' ').unwrap_or(content),
    };

    if content == SIGNATURE_SEPARATOR {
        lines.push(quote(depth, content));
        return;
    }

    // NOTE: a trailing space would turn this hard break into a soft
    // one, gluing the next line on at the reader's end.
    let mut rest = content.trim_end_matches(' ');

    loop {
        // NOTE: the next line starts like the rest, so it gets the
        // same quote marks and stuffing, which count in the column.
        let prefix = depth + usize::from(is_stuffed(depth, rest));
        let budget = column.saturating_sub(prefix);

        if rest.chars().count() <= budget {
            break;
        }

        let Some(cut) = find_cut(rest, budget) else {
            break;
        };

        lines.push(quote(depth, &rest[..=cut]));
        rest = &rest[cut + 1..];
    }

    lines.push(quote(depth, rest));
}

/// Finds the byte index of the space ending the longest soft line
/// fitting in `budget` characters, or ending the first word when
/// even that one overflows.
///
/// A space preceded by spaces only is never a cut, since it would
/// emit a line carrying nothing but its soft marker.
fn find_cut(rest: &str, budget: usize) -> Option<usize> {
    let is_cut =
        |&(i, c): &(usize, char)| c == ' ' && !rest[..i].trim_start_matches(' ').is_empty();

    let fitting = rest.char_indices().take(budget).filter(is_cut).last();

    fitting
        .or_else(|| rest.char_indices().find(is_cut))
        .map(|(i, _)| i)
}

/// Prefixes the content with its quote marks, space-stuffing it when
/// needed.
fn quote(depth: usize, content: &str) -> String {
    let mut line = ">".repeat(depth);

    if is_stuffed(depth, content) {
        line.push(' ');
    }

    line.push_str(content);
    line
}

/// Whether the content gets a space after its quote marks (section
/// 4.4).
///
/// A quoted line always does, the space both separating the marks
/// from the text and stuffing it. An unquoted one does only when it
/// would otherwise be misread: a leading space, a `>` taken for a
/// quote, or a `From ` mangled by mbox.
fn is_stuffed(depth: usize, content: &str) -> bool {
    match depth {
        0 => content.starts_with([' ', '>']) || content.starts_with("From "),
        _ => !content.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::encode;

    #[test]
    fn short_lines_are_untouched() {
        let text = "Hello,\n\nA short line.\n";
        assert_eq!(encode(text, 72), text);
    }

    #[test]
    fn long_line_breaks_on_spaces() {
        let text = "aaaa bbbb cccc dddd";
        assert_eq!(encode(text, 10), "aaaa bbbb \ncccc dddd");
    }

    #[test]
    fn trailing_spaces_are_stripped() {
        assert_eq!(encode("hard   \nnext", 72), "hard\nnext");
    }

    #[test]
    fn signature_separator_is_kept() {
        let text = "body\n-- \nAlice";
        assert_eq!(encode(text, 72), text);
    }

    #[test]
    fn quoted_line_repeats_its_prefix() {
        let text = ">> aaaa bbbb cccc";
        assert_eq!(encode(text, 12), ">> aaaa \n>> bbbb cccc");
    }

    #[test]
    fn empty_quoted_line_gets_no_space() {
        assert_eq!(encode(">\n> text", 72), ">\n> text");
    }

    #[test]
    fn misreadable_lines_are_stuffed() {
        let text = " indented\nFrom here\nplain";
        assert_eq!(encode(text, 72), "  indented\n From here\nplain");
    }

    #[test]
    fn stuffing_counts_in_the_column() {
        assert_eq!(encode("From aaaa bbbb", 10), " From \naaaa bbbb");
    }

    #[test]
    fn broken_piece_starting_with_space_is_stuffed() {
        assert_eq!(encode("aaaa  bbbb", 5), "aaaa \n  bbbb");
    }

    #[test]
    fn overlong_word_is_kept_whole() {
        let url = "https://example.org/a/very/long/path";
        let text = format!("see {url} now");
        assert_eq!(encode(&text, 10), format!("see \n{url} \nnow"));
    }

    #[test]
    fn crlf_input_is_normalised() {
        assert_eq!(encode("one\r\ntwo\r\n", 72), "one\ntwo\n");
    }

    #[test]
    fn column_998_breaks_nothing() {
        let text = "word ".repeat(150);
        assert_eq!(encode(&text, 998), text.trim_end());
    }
}
