# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Imported the MML compile/interpret library from the legacy pimalaya/core/mml repository.

  The library and the CLI now ship from the same crate, so downstream users depend on `mml` directly instead of going through a separate `mml-lib` crate.

- Imported the new, reply, and forward template builders from the legacy pimalaya/core/email repository, exposed as the `template` subcommand (alias `tpl`).

- Added editor-driven subcommands `compose`, `reply`, and `forward`.

  Each opens `$VISUAL` / `$EDITOR` on a pre-filled MML template, compiles the buffer on save, then prompts to validate, re-edit, view, or abort.

- Added a `read` visible alias on the `interpret` subcommand so himalaya v2's `[message.reader.mml]` slot can spawn `mml read` against a piped MIME message.

- Added TOML configuration via pimalaya-config.

  Loaded from $XDG_CONFIG_HOME/mml/config.toml, $HOME/.config/mml/config.toml, or $HOME/.mmlrc; the path can be overridden with `-c <PATH>` or `MML_CONFIG=<PATH>`. Identities live under `[accounts.<NAME>]` with per-command defaults under `[accounts.<NAME>.compose]`, `[accounts.<NAME>.reply]`, `[accounts.<NAME>.forward]`, and `[accounts.<NAME>.read]`. A documented config.sample.toml is shipped at the repository root.

- Added `completions` and `manuals` subcommands provided by pimalaya-cli.

- Added install.sh for installing the pre-built binary from the GitHub releases.

- Added dual-licensing under MIT and Apache-2.0, replacing the single MIT license.

### Changed

- Split functionality behind three opt-in cargo features `compiler`, `interpreter`, and `cli`, all enabled by default.

  Library consumers can now set `default-features = false` and pick only `compiler` and/or `interpreter` to drop clap, ariadne, edit, pimalaya-cli, and pimalaya-config from the dependency tree.

- Moved the repository from soywod/mml to pimalaya/mml; homepage and documentation links now point to pimalaya.org and docs.rs/mml.

- Adopted the pimalaya/nix flake module and the pimalaya-cli build helper for nix builds, GitHub release workflows, and the on-demand pre-release artifacts.

- Restructured the source tree into cli/, compiler/, interpreter/, template/, header.rs, and grammar.rs, replacing the previous mml/ and message/ layout.

- Bumped all dependencies, notably chumsky 0.13, mail-builder 0.4, mail-parser 0.11, and thiserror 2.

### Fixed

- Fixed missing `tokio` and `rustls` cargo features required by downstream pimalaya-tui consumers.

- Fixed the release and audit GitHub workflows after the pimalaya/nix migration.

- Fixed the cargo-deny configuration so license and advisory checks pass on CI.

## [1.0.0] - 2023-09-27

### Changed

- Bumped `mml-lib@v1.0.0`.

## [0.3.0] - 2023-09-20

### Added

- Added better diagnostic with `ariadne`.
- Added `mml interpret` options:
  - `--include-header` to include specific header to the interpreted message (cumulative)
  - `--exclude-header` to exclude specific header from the interpreted message (cumulative)
  - `--include-part` to include specific MIME types to the interpreted message (cumulative)
  - `--exclude-part` to exclude specific MIME types from the interpreted message (cumulative)
  - `--show-multiparts` to enable interpretation of multiparts
  - `--save-attachments` to automatically save attachments to directory defined by `--save-attachments-dir`
  - `--save-attachments-dir` to define directory attachments should point to
  - `--hide-attachments` to disable interpretation of all attachments
  - `--hide-inline-attachments` to disable interpretation of inline attachments only
  - `--hide-plain-texts-signature` to trim out signature from text plain parts

### Changed

- Bumped `mml-lib@v0.4.0`.

## [0.2.1] - 2023-08-30

### Changed

- Improved shell expansion.
- Use `clap` derive feature for parsing arguments.

## [0.2.0] - 2023-08-27

### Changed

- Bumped `mml-lib@v0.2.0`.
- Renamed cargo feature `pgp-cmds` to `pgp-commands`.

### Fixed

- Fixed wrong main command name.

## [0.1.1] - 2023-08-27

### Fixed

- Fixed missing angles when compiling MML containing one of those headers: Message-ID, References, In-Reply-To, Return-Path, Content-ID, Resent-Message-ID.
- Fixed windows build.

### Removed

- Removed `pgp-cmds` feature from default ones.

## [0.1.0] - 2023-08-23

### Added

- Added `compile` and `interpret` feature from [mml-lib](https://crates.io/crates/mml-lib).

[Unreleased]: https://github.com/soywod/mml/compare/v1.0.0...master
[1.0.0]: https://github.com/soywod/mml/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/soywod/mml/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/soywod/mml/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/soywod/mml/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/soywod/mml/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/soywod/mml/releases/tag/v0.1.0
