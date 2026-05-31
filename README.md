# MML [![Documentation](https://img.shields.io/docsrs/mml?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/mml/latest/mml) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

CLI and lib for the Emacs MIME message Meta Language ([MML](https://www.gnu.org/software/emacs/manual/html_node/emacs-mime/MML-Definition.html)), written in Rust.

This repository ships:

- A **library** exposing two pipelines (MML→MIME compiler, MIME→MML interpreter) and a template builder for compose/reply/forward drafts.
- A **CLI** wrapping the library, plus three editor-driven commands (`compose`, `reply`, `forward`) that bundle "template → `$EDITOR` → compile" with a post-edit choice prompt, and `interpret` (aliased `read`) for the inverse MIME→MML flow.

## Table of contents

- [Features](#features)
- [Installation](#installation)
  - [Pre-built binary](#pre-built-binary)
  - [Cargo](#cargo)
  - [Nix](#nix)
  - [Sources](#sources)
- [Configuration](#configuration)
- [Usage](#usage)
  - [Library](#library)
  - [CLI](#cli)
- [FAQ](#faq)
- [License](#license)
- [AI disclosure](#ai-disclosure)
- [Social](#social)
- [Sponsoring](#sponsoring)

## Features

- **MML → MIME compilation** (requires `compiler` feature):
  - `<#part>` / `<#multipart>` directives with `type`, `filename`, `disposition`, `encoding`, `description`, `name`, `recipient-filename`, dates, etc.
  - Inline parts, attached parts, nested multiparts (`alternative`, `mixed`, `related`)
  - File-path expansion via [`shellexpand`](https://crates.io/crates/shellexpand)
  - MIME-type detection via [`tree_magic_mini`](https://crates.io/crates/tree_magic_mini)
  - Parse-error reporting via [`ariadne`](https://crates.io/crates/ariadne) (CLI)
- **MIME → MML interpretation** (requires `interpreter` feature):
  - Header include / exclude filters
  - Part include / exclude filters
  - HTML → text rendering via [`nanohtml2text`](https://crates.io/crates/nanohtml2text)
  - Attachment save-to-disk
  - `mml interpret` (aliased `mml read`): MIME on stdin, MML/text on stdout (himalaya `read-with` slot)
- **Editor-driven flow** (requires `cli` + `compiler` + `interpreter`):
  - `mml compose` / `mml reply` / `mml forward`: open `$EDITOR`, compile on save, prompt to validate / re-edit / view / abort
- **TOML configuration** with per-account identities and per-section defaults (`[compose]`, `[reply]`, `[forward]`, `[read]`)

> [!TIP]
> MML is written in [Rust](https://www.rust-lang.org/) and uses [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to gate functionality. The default feature set is declared in [Cargo.toml](./Cargo.toml).

## Installation

### Pre-built binary

The CLI binary `mml` can be installed from the latest [GitHub release](https://github.com/pimalaya/mml/releases) using the install script:

*As root:*

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/mml/master/install.sh | sudo sh
```

*As a regular user:*

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/mml/master/install.sh | PREFIX=~/.local sh
```

For a more up-to-date version, check out the [pre-releases](https://github.com/pimalaya/mml/actions/workflows/pre-releases.yml) GitHub workflow: pick the latest run and grab the artifact matching your OS. These are built from the `master` branch.

> [!NOTE]
> Pre-built binaries are built with the default cargo features. If you need a different feature set, use another installation method.

### Cargo

```sh
cargo install mml --locked
```

You can also use the git repository for a more up-to-date (but less stable) version:

```sh
cargo install --locked --git https://github.com/pimalaya/mml.git
```

To use `mml` as a library, add it to your `Cargo.toml`:

```toml
[dependencies]
mml = { version = "1.0", default-features = false, features = ["compiler", "interpreter"] }
```

Drop `cli` (and pick only `compiler` and/or `interpreter`) for a slim library build with no clap, no ariadne, no editor integration.

### Nix

If you have the [Flakes](https://nixos.wiki/wiki/Flakes) feature enabled:

```sh
nix profile install github:pimalaya/mml
```

Or run without installing:

```sh
nix run github:pimalaya/mml -- compile <<<'<#part>Hello, world!<#/part>'
```

### Sources

```sh
git clone https://github.com/pimalaya/mml
cd mml
nix run
```

## Configuration

A sample [config.sample.toml](./config.sample.toml) is shipped at the repository root. Drop it into one of:

- `$XDG_CONFIG_HOME/mml/config.toml`
- `$HOME/.config/mml/config.toml`
- `$HOME/.mmlrc`

Override the path with `-c <PATH>` or `MML_CONFIG=<PATH>`.

CLI flags always win; config values fill in the blanks. Pick an account with `-a <NAME>`, or flag one entry `default = true`.

## Usage

### Library

Compile MML to MIME:

```rust,ignore
use mml::compiler::message::MmlCompilerBuilder;

let mml = "<#part>Hello, world!<#/part>";
let mime = MmlCompilerBuilder::new()
    .build(mml)?
    .compile()?
    .into_string()?;

println!("{mime}");
```

Interpret MIME back to MML:

```rust,ignore
use mml::interpreter::message::MimeInterpreterBuilder;

let mime = b"From: a@b\r\nTo: c@d\r\nSubject: Hi\r\n\r\nHello!\r\n";
let mml = MimeInterpreterBuilder::new()
    .with_show_only_headers(["From", "To", "Subject"])
    .build()
    .from_bytes(mime)?;

println!("{mml}");
```

### CLI

Compile MML on stdin, emit MIME on stdout:

```sh
mml compile <<< '<#part>Hello, world!<#/part>'
```

Interpret MIME back to MML/text:

```sh
mml interpret < message.eml
```

Open the editor on a fresh compose draft, then emit the compiled MIME message on stdout:

```sh
mml compose --from me@example.org
```

Reply / forward from a piped MIME message:

```sh
cat message.eml | mml reply --all
cat message.eml | mml forward
```

Read (MIME → text) for himalaya's `read-with`:

```sh
cat message.eml | mml read --exclude-header Received,DKIM-Signature
```

Generate a draft template without opening the editor:

```sh
mml template compose --from me@example.org
mml template reply --all < message.eml
mml template forward < message.eml
```

Plug `mml` into [himalaya](https://github.com/pimalaya/himalaya) v2:

```toml
[message.composer.mml]
command = "mml compose"
default = true

[message.reader.mml]
command = "mml read"
default = true
```

## FAQ

<details>
  <summary>How to debug the CLI?</summary>

  Use `--log <level>` where `<level>` is one of `off`, `error`, `warn`, `info`, `debug`, `trace`:

  ```sh
  mml --log trace compile < message.mml
  ```

  The `RUST_LOG` environment variable, when set, overrides `--log` and supports per-target filters (see the [env_logger](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) documentation). `RUST_BACKTRACE=1` enables full error backtraces, including source lines where the error originated from.

  Logs are written to `stderr`, so they can be redirected easily to a file:

  ```sh
  mml --log trace compile < message.mml 2>/tmp/mml.log
  ```
</details>

<details>
  <summary>How does `mml compose` pick the editor?</summary>

  The [edit](https://crates.io/crates/edit) crate resolves `$VISUAL` first, then `$EDITOR`, then an OS default. `mml` does not expose a config knob on top: set `VISUAL` / `EDITOR` in your shell rc file.
</details>

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## AI disclosure

This project is developed with AI assistance. This section documents how, so users and downstream packagers can make informed decisions.

- **Tools**: Claude Code (Anthropic), Opus 4.7, invoked locally with a persistent project-scoped memory and a small set of repo-specific rules.

- **Used for**: Refactors, mechanical multi-file edits, boilerplate (feature gates, error enums, derive macros, trait impls), test scaffolding, doc polish, exploratory design conversations.

- **Not used for**: Engineering, critical code, git manipulation (commit, merge, rebase…), real-world tests.

- **Verification**: Every AI-assisted change is read, compiled, tested, and formatted before commit (`nix develop --command cargo check / cargo test / cargo fmt`). Behavioural correctness is verified against the relevant RFC or upstream spec, not assumed from the model output. Tests are never adjusted to fit AI-generated code; the code is adjusted to fit correct behaviour.

- **Limitations**: AI models occasionally produce code that compiles and passes tests but is subtly wrong: off-by-one errors, missed edge cases, plausible but nonexistent APIs, stale RFC references. The verification workflow catches most of this; it does not catch all of it. Bug reports are welcome and taken seriously.

- **Last reviewed**: 31/05/2026

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- *2027 in preparation…*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
