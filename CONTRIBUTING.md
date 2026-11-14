# Contributing guide

Thank you for investing your time in contributing to MML.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide (Nix environment, build and check commands, dependency overrides, commit style), which chains to the shared architecture and guidelines;
3. the inline header documentation in src/lib.rs: it is the architecture document of this crate, covering the compiler and interpreter pipelines, the template layer, and the optional CLI;
4. the cairn/ folder, which follows [Cairn](https://github.com/pimalaya/cairn) (spec/ is current truth, changes/ holds in-flight proposals, log/ is the dated history), activated by AGENTS.md.

Everything below documents only what differs from the Pimalaya standards.

## Where changes belong

MML is a single crate shipping both a library and the `mml` binary. Triage before patching:

- the MML grammar, the MML→MIME compiler and the MIME→MML interpreter live here, in the compiler and interpreter modules;
- MIME serialization and parsing come from [mail-builder](https://crates.io/crates/mail-builder) and [mail-parser](https://crates.io/crates/mail-parser), so wire-level MIME bugs usually belong upstream;
- the clap tree, printer and prompt primitives come from [pimalaya/cli](https://github.com/pimalaya/cli), and the TOML loader from [pimalaya/config](https://github.com/pimalaya/config);
- the editor-driven compose/reply/forward flow and the TOML configuration live in the cli module.

## Feature matrix

MML is a library and a binary in one crate. Its cargo features are `compiler` (MML→MIME) and `interpreter` (MIME→MML), both on by default, `gpg`, serving PGP through gpgme, plus `cli`, which gates the `mml` binary, turns both pipelines on and pulls clap, ariadne, the editor integration, the config loader and schemars. CI runs clippy with `-D warnings` over every set below, so check them when touching a gate:

```sh
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets --no-default-features --features compiler -- -D warnings
cargo clippy --all-targets --no-default-features --features interpreter -- -D warnings
cargo clippy --all-targets --no-default-features --features cli -- -D warnings
cargo clippy --all-targets --features gpg -- -D warnings
```
