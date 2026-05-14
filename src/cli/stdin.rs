//! CRLF normalization helpers — RFC 5322 expects `\r\n` line endings,
//! but stdin and inline arguments usually arrive with raw `\n`.

use std::io::{stdin, BufRead, BufReader};

/// Normalize an inline argument into CRLF-terminated lines, matching
/// the on-disk RFC 5322 shape so it can be fed straight into a MIME
/// parser. `\r` is stripped and `\n` becomes `\r\n`; the previous CLI
/// also expanded the escape sequences `\\r` / `\\n` (kept here for
/// backward compatibility with the inline argument form).
pub fn format_str(input: &str) -> String {
    let input = input.replace("\\r", "").replace("\\n", "\n");
    let mut output = String::new();

    for line in input.lines() {
        output.push_str(line);
        output.push('\r');
        output.push('\n');
    }

    output
}

/// Read all of stdin as a string, normalizing line endings to CRLF
/// (same shape as [`format_str`]).
pub fn format_stdin() -> String {
    let mut output = String::new();

    for line in BufReader::new(stdin()).lines().map_while(Result::ok) {
        output.push_str(&line);
        output.push('\r');
        output.push('\n');
    }

    output
}
