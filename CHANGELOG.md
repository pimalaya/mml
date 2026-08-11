# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added the `pgp-gpg` cargo feature, off by default, serving the `WantsPgp*` yields through gpgme. GPG owns key discovery, the agent and the passphrase prompt; `pgp::gpg::MmlPgpGpg` is the only place in the crate performing cryptography, and the coroutines never reach it.

  The signing identity comes from the part's `sender` property, falling back to the message `From`. Recipients come from the part's `recipients` property only: a config key could not know who a given message is addressed to.

  Recipient keys resolve through GPG's auto-key-locate, so WKD and keyserver lookups work as they do for the `gpg` command. Encryption uses `ALWAYS_TRUST`: a composer cannot demand every recipient key be signed first.

  The nix package pulls gpgme, libgpg-error and pkg-config only when the feature is on. The dev shell carries them unconditionally so the feature builds and tests there.

- Added the I/O-free coroutine seam. `coroutine::MmlYield` carries `Ok`, `Err` and the four inlined `WantsPgpEncrypt` / `WantsPgpDecrypt` / `WantsPgpSign` / `WantsPgpVerify` steps; `pgp::MmlPgpAnswer` and `pgp::MmlPgpVerdict` are what the caller resumes with. Data only, no trait: the caller matches on the yield and plugs its own backend.

- Added the `MmlCompile` and `MmlInterpret` coroutines, each built from the message and its options struct, each advanced with `resume`. `MmlCompiler` and `MmlMimeInterpreter` became those options, renamed `MmlCompileOptions` and `MmlInterpretOptions`, and keep a direct method for the PGP-free path.

- Added `interpreter::layer`, resolving encrypted parts without recursion. `MmlInterpret::resume` walks one MIME layer at a time, suspends on each `multipart/encrypted` it meets, and splices the decrypted layer back in once the caller answers. Nested encryption is a queue, not a call stack.

  Each layer owns its bytes through `MmlMimeLayer`, a `Cow<[u8]>` so the first layer borrows the caller's input at no cost, behind ouroboros so the parsed `mail_parser::Message` can borrow them.

- Added signature reporting. `MmlInterpret::verdicts` exposes every verdict a run collected. `mml interpret` turns them into a distinct exit code, 0 with no signature or a valid one, 2 with an invalid one, 1 on any other failure, and reports them under `--json` as a `signatures` array. The plain rendering stays the raw MML.

  The message renders either way, since a bad signature is a reason to distrust it rather than to hide it. The verdict never enters the rendered body, where it would be indistinguishable from text the sender wrote.

- Added `multipart/signed` interpretation (RFC 3156 section 5): the signed part renders as usual and the coroutine yields `WantsPgpVerify` with the exact bytes the signature covers, so the caller decides what to make of the verdict.

- Added the `sign`, `encrypt`, `sender` and `recipients` properties to the `<#part>` and `<#multipart>` parsers, restoring the grammar the legacy pimalaya/core/mml carried. Only `pgpmime` is accepted for `sign` and `encrypt`.

- Added compile-time PGP through `MmlCompile`, which now suspends once per `sign` or `encrypt` property, deepest part first and signing before encrypting per RFC 3156. Signed parts wrap in `multipart/signed`, encrypted ones in `multipart/encrypted`.

  Unlike the legacy implementation, an `Unavailable` or unusable answer aborts the compilation instead of falling back to the cleartext part. A message marked `encrypt=pgpmime` can no longer go out unencrypted.

- Added the `mailto` command, composing from an RFC 6068 `mailto:` URI. Recipients come from the path and the `to` query parameter, `subject`, `cc`, `bcc` and `in-reply-to` become headers, and `body` pre-fills the draft.

  Headers outside that set are dropped, as RFC 6068 section 5 advises against honouring arbitrary headers from an untrusted URI. Register `mml mailto "$1" --output <PATH>` as a desktop handler.

- Added the global `-o, --output <PATH>` flag, redirecting a command's artifact to a file. It defaults to stdout, and `-o -` names stdout explicitly.

  It is independent of `--json`: `--output` picks where the payload lands, `--json` picks its format, so `--json --output result.json` writes JSON to that file.

- Added the `-i, --input <PATH>` flag to every command reading a source, defaulting to stdin, with `-i -` naming stdin explicitly. An implicit stdin that is a terminal is rejected rather than left hanging.

### Changed

- Moved quote rendering out of the reply and forward template builders. `MmlTemplateReplyBuilder::build` and `MmlTemplateForwardBuilder::build` now take the rendered quote as a second argument, and each type exposes `quote_options()` describing how that quote should be rendered.

  This is what lets replying to or forwarding an encrypted message quote the plaintext: the builders interpreted the source themselves through the borrowed-message path, which cannot drive the PGP coroutine, so a quoted thread came out as `<#part type=application/pgp-encrypted>`. The caller now drives interpretation and hands the result in.

- Collapsed the compiler into a single `MmlCompiler` with public fields and one `compile` method returning MIME bytes, replacing the `MmlCompilerBuilder` / `MmlCompiler` / `MmlCompileResult` chain and its `into_vec` / `into_string` / `*_msg_builder` accessors.

  `MmlCompilerBuilder::new().build(&mml)?.compile()?.into_vec()?` becomes `MmlCompiler::default().compile(&mml)?`. The builder carried no options at all, so the three types were pure ceremony.

- Collapsed the interpreter the same way. `MmlMimeInterpreterBuilder` is gone, and `MmlInterpretOptions` exposes `show_headers` plus a `body` of public options, built with a struct literal and `..Default::default()`. The 22 `with_*` setters and the `build` step went with it, and the three `from_*` entry points became `interpret`, `interpret_builder` and `interpret_msg`.

  The setters that carried logic moved onto the filter: `with_show_only_headers` is `MmlFilterHeaders::include` (still deduplicating), and `with_show_additional_headers` is `MmlFilterHeaders::extend`. `with_show_all_headers`, `with_hide_all_headers` and `with_save_some_attachments_dir` become plain values (`All`, `Include(Vec::new())`, and an `unwrap_or_else` at the call site).

- Shared `--from`, `--from-name` and `--signature` across the seven commands building a template, so their help text and behaviour no longer drift. `templates compose` and `templates forward` lose their custom `<EMAIL>` and `<NAME>` value names in favour of the common ones.

- Renamed every public library item to carry the `Mml` domain prefix, per the Pimalaya naming guidelines: `Template` became `MmlTemplate`, `TemplateCursor` became `MmlTemplateCursor`, `TemplateBody` became `MmlTemplateBody`, `TemplateBuilderCompose` became `MmlTemplateComposeBuilder` (same for reply and forward), the `Template*Style` enums gained the prefix, `MimeInterpreter` became `MmlMimeInterpreter`, `MimeBodyInterpreter` became `MmlMimeBodyInterpreter`, and `FilterHeaders` / `FilterParts` became `MmlFilterHeaders` / `MmlFilterParts`.

- Moved the shared template types out of the `template::types` catch-all module and up into `template`, so they are reached as `mml::template::MmlTemplate` instead of `mml::template::types::Template`.

- Declared `default-features = false` on every dependency, enabling only the features actually used.

### Removed

- Removed the optional leading positional output path on `compose`, `reply` and `forward`. Use `--output <PATH>` instead, which no longer depends on positional order.

- Removed the trailing positional `MESSAGE` argument from every command. Sources now come from `--input <PATH>` or stdin.

  It was reachable only after a `--` separator, it silently lost to `--input`, and being a trailing var-arg it swallowed any flag typed after it. Its three roles are covered: a path is `--input PATH`, stdin is the default or `--input -`, and an inline message is a pipe or `--input <(printf ...)`.

  `mml reply <OUTPUT> -- <SOURCE>` becomes `mml reply --output <OUTPUT> --input <SOURCE>`.

### Fixed

- Fixed `compile` silently ignoring `sign` and `encrypt`, so a part marked `encrypt=pgpmime` compiled and shipped in the clear. `MmlCompileOptions::compile` now drives `MmlCompile` with no backend, which aborts on any crypto request. `MmlInterpretOptions::interpret` drives `MmlInterpret` the same way, so both paths and the coroutines share one implementation.

- Fixed `interpret` and `read` printing control characters instead of the encrypted part. The PGP-free path collects no request, so the placeholder marking where a decrypted layer belongs was never resolved. It now renders as `<#part type=application/pgp-encrypted>`, the same as an `Unavailable` answer.

- Fixed `compose`, `reply` and `forward` hanging when their output would land on a redirected stdout. The editor inherits that stream, so its interface never reaches the terminal. The commands now refuse to start, with an error naming the fix.

## [1.1.1] - 2026-06-07

### Fixed

- Fixed CI release build.

## [1.1.0] - 2026-06-07

### Added

- Imported the MML compile/interpret library from the legacy pimalaya/core/mml repository.

  The library and the CLI now ship from the same crate, so downstream users depend on `mml` directly instead of going through a separate `mml-lib` crate.

- Imported the new, reply, and forward template builders from the legacy pimalaya/core/email repository, exposed as the `template` subcommand (alias `tpl`).

- Added editor-driven subcommands `compose`, `reply`, and `forward`.

  Each opens `$VISUAL` / `$EDITOR` on a pre-filled MML template, compiles the buffer on save, then prompts to validate, re-edit, view, or abort.

  Each subcommand also takes an optional leading positional `<OUTPUT>` path. When given, the compiled MIME bytes are written to that file instead of stdout, keeping mml's stdout connected to the terminal so the spawned editor sees a real tty. This is the form expected by shell pipelines into a consumer that itself opens an editor (`mml compose /tmp/x.eml && himalaya messages send /tmp/x.eml`) and by bash/zsh process substitution (`mml compose >(himalaya messages send)`). For `reply` / `forward`, the source MIME is the trailing positional (after `--`) or stdin.

- Added a `read` visible alias on the `interpret` subcommand for the inverse MIME→text flow.

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

[Unreleased]: https://github.com/pimalaya/mml/compare/v1.1.1...master
[1.1.1]: https://github.com/pimalaya/mml/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/pimalaya/mml/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/pimalaya/mml/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/pimalaya/mml/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/pimalaya/mml/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/pimalaya/mml/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/pimalaya/mml/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/pimalaya/mml/releases/tag/v0.1.0
