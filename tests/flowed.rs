//! RFC 3676 `format=flowed` across both pipelines: what the compiler
//! flows, the interpreter unflows back into the body it came from.

#![cfg(all(feature = "compiler", feature = "interpreter"))]

use mml::{
    compiler::{
        body::{MmlBodyCompiler, MmlBodyFormat},
        message::MmlCompileOptions,
    },
    interpreter::message::{MmlFilterHeaders, MmlInterpretOptions},
};

/// Compiles the body as flowed at the given column.
fn compile(body: &str, column: usize) -> String {
    let opts = MmlCompileOptions {
        body: MmlBodyCompiler {
            format: MmlBodyFormat::Flowed,
            flowed_column: column,
        },
    };

    let mml = format!("From: alice@localhost\nTo: bob@localhost\nSubject: test\n\n{body}");
    String::from_utf8(opts.compile(&mml).unwrap()).unwrap()
}

/// Interprets the MIME back into its body alone.
fn interpret(mime: &str) -> String {
    let opts = MmlInterpretOptions {
        show_headers: MmlFilterHeaders::Include(Vec::new()),
        ..Default::default()
    };

    opts.interpret(mime).unwrap()
}

#[test]
fn paragraph_round_trips() {
    let body = format!(
        "Hello Bob,\n\n{}\n",
        "a long paragraph ".repeat(20).trim_end()
    );
    let mime = compile(&body, 72);

    assert!(mime.contains("format=\"flowed\""));
    assert!(mime.contains("charset=\"utf-8\""));
    assert!(mime.contains("Content-Transfer-Encoding: quoted-printable"));
    assert!(mime.contains("=20\r\n"));
    assert_eq!(interpret(&mime).trim_start(), body);
}

#[test]
fn quoted_reply_round_trips() {
    let quote = "a quoted line that goes well beyond the column ".repeat(3);
    let body = format!("> {}\n>> nested\n\nMy answer.\n", quote.trim_end());

    assert_eq!(interpret(&compile(&body, 40)).trim_start(), body);
}

#[test]
fn signature_round_trips() {
    let body = "Body text.\n\n-- \nAlice\n";
    assert_eq!(interpret(&compile(body, 72)).trim_start(), body);
}

#[test]
fn stuffed_lines_round_trip() {
    let body = "From here on\n indented\n";
    let mime = compile(body, 72);

    assert!(mime.contains("\r\n From here on\r\n"));
    assert_eq!(interpret(&mime).trim_start(), body);
}

#[test]
fn column_998_ships_paragraphs_whole() {
    let paragraph = "word ".repeat(150);
    let body = format!("{}\n", paragraph.trim_end());
    let mime = compile(&body, 998);

    assert!(!mime.contains("=20\r\n"));
    assert_eq!(interpret(&mime).trim_start(), body);
}

#[test]
fn fixed_part_is_untouched() {
    let long = "a long line ".repeat(10);
    let body = format!(
        "<#part type=text/plain format=fixed>{}\n<#/part>\n",
        long.trim_end()
    );
    let mime = compile(&body, 20);

    assert!(!mime.contains("flowed"));
}

#[test]
fn signed_part_is_not_flowed() {
    let opts = MmlCompileOptions {
        body: MmlBodyCompiler {
            format: MmlBodyFormat::Flowed,
            flowed_column: 20,
        },
    };

    let long = "a long line ".repeat(10);
    let body = format!(
        "<#part type=text/plain sign=pgpmime>{}\n<#/part>\n",
        long.trim_end()
    );
    let parts = opts.body.compile(&body).unwrap();
    let mime = String::from_utf8(parts.write_to_vec().unwrap()).unwrap();

    assert!(!mime.contains("flowed"));
}

#[test]
fn flowed_part_overrides_a_fixed_default() {
    let long = "a long line ".repeat(10);
    let body = format!(
        "<#part type=text/plain format=flowed>{}\n<#/part>\n",
        long.trim_end()
    );
    let parts = MmlBodyCompiler::default().compile(&body).unwrap();
    let mime = String::from_utf8(parts.write_to_vec().unwrap()).unwrap();

    assert!(mime.contains("format=\"flowed\""));
}

#[test]
fn delsp_is_honoured() {
    let mime = concat!(
        "Content-Type: text/plain; charset=utf-8; format=flowed; delsp=yes\r\n",
        "\r\n",
        "abc \r\n",
        "def\r\n",
    );

    assert_eq!(interpret(mime).trim(), "abcdef");
}
