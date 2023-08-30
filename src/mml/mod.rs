use std::io::{self, BufRead, BufReader};

#[cfg(feature = "compiler")]
pub mod compiler;
#[cfg(feature = "interpreter")]
pub mod interpreter;

pub(crate) fn format_str(input: &str) -> String {
    let input = input.replace("\\r", "").replace("\\n", "\n");
    let mut lines = input.lines();
    let mut output = String::new();

    while let Some(ref line) = lines.next() {
        output.push_str(line);
        output.push('\r');
        output.push('\n');
    }

    output
}

pub(crate) fn format_stdin() -> String {
    let mut lines = BufReader::new(io::stdin()).lines();
    let mut output = String::new();

    while let Some(Ok(ref line)) = lines.next() {
        output.push_str(line);
        output.push('\r');
        output.push('\n');
    }

    output
}
