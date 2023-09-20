# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Use `clap` derive feature for parsing arguments. [patch#44147]

## [0.2.0] - 2023-08-27

### Changed

- Bumped `mml-lib@v0.2.0`.
- Renamed cargo feature `pgp-cmds` to `pgp-commands`.

### Fixed

- Fixed wrong main command name [patch#44036].

## [0.1.1] - 2023-08-27

### Fixed

- Fixed missing angles when compiling MML containing one of those headers: Message-ID, References, In-Reply-To, Return-Path, Content-ID, Resent-Message-ID.
- Fixed windows build.

### Removed

- Removed `pgp-cmds` feature from default ones.

## [0.1.0] - 2023-08-23

### Added

- Added `compile` and `interpret` feature from [mml-lib].

[mml-lib]: https://crates.io/crates/mml-lib

[patch#44036]: https://lists.sr.ht/~soywod/pimalaya/patches/44036
[patch#44147]: https://lists.sr.ht/~soywod/pimalaya/patches/44147

[Unreleased]: https://github.com/soywod/mml/compare/v0.2.1...master
[0.2.1]: https://github.com/soywod/mml/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/soywod/mml/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/soywod/mml/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/soywod/mml/releases/tag/v0.1.0
