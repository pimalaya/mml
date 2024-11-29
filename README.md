# 📫 MML CLI [![GitHub release](https://img.shields.io/github/v/release/pimalaya/mml?color=success)](https://github.com/pimalaya/mml/releases/latest) [![Matrix](https://img.shields.io/matrix/pimalaya:matrix.org?color=success&label=chat)](https://matrix.to/#/#pimalaya:matrix.org)

CLI to convert [MIME](https://www.rfc-editor.org/rfc/rfc2045) messages into/from Emacs [MIME Meta Language](https://www.gnu.org/software/emacs/manual/html_node/emacs-mime/MML-Definition.html), based on [`mml-lib`](https://crates.io/crates/mml-lib)

## Features

- MML to MIME messages compilation (`mml compile --help`)
- MIME to MML messages interpretation (`mml interpret --help`)

*MML CLI is written in [Rust](https://www.rust-lang.org/), and relies on [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to enable or disable functionalities. Default features can be found in the `features` section of the [`Cargo.toml`](https://github.com/pimalaya/mml/blob/master/Cargo.toml#L18).*

## Installation

### From release

MML CLI can be installed with a pre-built binary from the latest [GitHub release](https://github.com/pimalaya/mml/releases):

```bash
# As root:
$ curl -sSL https://raw.githubusercontent.com/pimalaya/mml/master/install.sh | sudo sh

# As a regular user:
$ curl -sSL https://raw.githubusercontent.com/pimalaya/mml/master/install.sh | PREFIX=~/.local sh
```

### From CI

MML CLI can be installed with a pre-built binary from the [`pre-releases` GitHub workflow](https://github.com/pimalaya/mml/actions/workflows/pre-releases.yml): take the latest build, find the *Artifacts* section, you should find a pre-built binary matching your OS.

This workflow is triggered everytime a new commit is pushed on `master`. Such binary is usually more up-to-date than a release, but also less stable.

*Pre-built binaries are built with [default](https://github.com/pimalaya/mml/blob/master/Cargo.toml#L18) cargo features. If you want to enable or disable a feature, please use another installation method.*

### Other

<details>
  <summary>Cargo</summary>

  MML CLI can be installed with [cargo](https://doc.rust-lang.org/cargo/):

  ```bash
  $ cargo install mml

  # With only IMAP support:
  $ cargo install mml --no-default-features --features imap
  ```

  You can also use the git repository for a more up-to-date (but less stable) version:

  ```bash
  $ cargo install --git https://github.com/pimalaya/mml.git mml
  ```
</details>

<details>
  <summary>Nix</summary>

  MML CLI can be installed with [Nix](https://serokell.io/blog/what-is-nix):

  ```bash
  $ nix-env -i mml
  ```

  You can also use the git repository for a more up-to-date (but less stable) version:

  ```bash
  $ nix-env -if https://github.com/pimalaya/mml/archive/master.tar.gz

  # or, from within the source tree checkout
  $ nix-env -if .
  ```

  If you have the [Flakes](https://nixos.wiki/wiki/Flakes) feature enabled:

  ```bash
  $ nix profile install mml

  # or, from within the source tree checkout
  $ nix profile install

  # you can also run MML directly without installing it:
  $ nix run mml
  ```
</details>

<details>
  <summary>Sources</summary>

  MML CLI can be installed from sources.

  First you need to install the Rust development environment (see the [rust installation documentation](https://doc.rust-lang.org/cargo/getting-started/installation.html)):

  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

  Then, you need to clone the repository and install dependencies:

  ```bash
  $ git clone https://github.com/pimalaya/mml.git
  $ cd mml
  $ cargo check
  ```

  Now, you can build MML:

  ```bash
  $ cargo build --release
  ```

  *Binaries are available under the `target/release` folder.*
</details>

## FAQ

<details>
  <summary>How to debug MML CLI?</summary>

  The simplest way is to use `--debug` and `--trace` arguments.

  The advanced way is based on environment variables:

  - `RUST_LOG=<level>`: determines the log level filter, can be one of `off`, `error`, `warn`, `info`, `debug` and `trace`.
  - `RUST_SPANTRACE=1`: enables the spantrace (a span represent periods of time in which a program was executing in a particular context).
  - `RUST_BACKTRACE=1`: enables the error backtrace.
  - `RUST_BACKTRACE=full`: enables the full error backtrace, which include source lines where the error originated from.

  Logs are written to the `stderr`, which means that you can redirect them easily to a file:

  ```
  RUST_LOG=debug mml 2>/tmp/mml.log
  ```
</details>

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that helped the project to receive financial support from various programs:

- [NGI Assure](https://nlnet.nl/project/Himalaya/) in 2022
- [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/) in 2023
- [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/) in 2024 *(still ongoing)*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
