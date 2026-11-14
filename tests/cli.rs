//! The `mml` binary against the family output contract: exit statuses,
//! the failure object and its code, the printed shapes.
//!
//! Each run gets a scratch home, so no configuration of the machine
//! running the suite is read.

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

const MESSAGE: &str =
    "From: alice@example.org\r\nTo: bob@example.org\r\nSubject: Hi\r\n\r\nHello\r\n";

/// Runs the binary with `args` and `stdin` under the scratch `home`.
fn mml(home: &Path, args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mml"))
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("RUST_LOG")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();

    child.wait_with_output().unwrap()
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn a_refused_line_under_json_prints_the_coded_failure_and_exits_2() {
    let home = TempDir::new().unwrap();
    let output = mml(home.path(), &["--json", "compile", "--nope"], "");

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json(&output)["code"], "usage-invalid");
    assert!(output.stderr.is_empty());
}

#[test]
fn an_mml_source_that_does_not_parse_fails_coded() {
    let home = TempDir::new().unwrap();
    let mml_src = "Subject: Hi\n\n<#part type=text/plain disposition=nope>Hello<#/part>\n";
    let output = mml(home.path(), &["--json", "compile"], mml_src);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json(&output)["code"], "mml-invalid");
    assert!(output.stderr.is_empty());
}

#[test]
fn a_compiled_message_prints_as_bytes_under_json() {
    let home = TempDir::new().unwrap();
    let output = mml(
        home.path(),
        &["--json", "compile"],
        "Subject: Hi\n\nHello\n",
    );
    let value = json(&output);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(value["mime"], "message/rfc822");
    assert!(
        value["data"].is_string() && value["size"].is_u64(),
        "{value}"
    );
}

#[test]
fn an_interpreted_message_prints_its_mml_and_signatures() {
    let home = TempDir::new().unwrap();
    let output = mml(home.path(), &["--json", "interpret"], MESSAGE);
    let value = json(&output);

    assert_eq!(output.status.code(), Some(0));
    assert!(value["mml"].as_str().unwrap().contains("Hello"), "{value}");
    assert_eq!(value["signatures"], Value::Array(Vec::new()));
}

#[test]
fn no_configuration_is_the_empty_one_for_a_command() {
    let home = TempDir::new().unwrap();
    let args = ["template", "compose", "--from", "alice@example.org"];
    let output = mml(home.path(), &args, "");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("From: alice@example.org"), "{stdout}");
}

#[test]
fn a_missing_config_path_is_refused() {
    let home = TempDir::new().unwrap();
    let missing = home.path().join("missing.toml");
    let args = [
        "--json",
        "-c",
        missing.to_str().unwrap(),
        "template",
        "compose",
    ];
    let output = mml(home.path(), &args, "");

    assert_eq!(output.status.code(), Some(1));
    let error = json(&output)["error"].as_str().unwrap().to_owned();
    assert!(error.starts_with("No configuration found at"), "{error}");
}

#[test]
fn a_missing_from_fails_coded() {
    let home = TempDir::new().unwrap();
    let output = mml(home.path(), &["--json", "template", "compose"], "");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json(&output)["code"], "from-missing");
}

#[test]
fn a_forward_template_takes_its_quote_headline() {
    let home = TempDir::new().unwrap();
    let config = home.path().join("config.toml");
    fs::write(&config, "from = \"bob@example.org\"\n").unwrap();

    let args = [
        "-c",
        config.to_str().unwrap(),
        "template",
        "forward",
        "-Q",
        "Forwarded:\\n",
    ];
    let output = mml(home.path(), &args, MESSAGE);

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\nForwarded:\n"), "{stdout}");
}
