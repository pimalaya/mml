---
cairn: tasks
change: the-family-standards
---

# Tasks

- [x] Cargo.toml: pimalaya-cli and pimalaya-config 0.4 with `[patch.crates-io]` path patches, schemars, clap without `color`, `env` and `wrap_help`, shellexpand without `tilde`, `cli` turning both pipelines on, pimalaya-config `sample` as a dev-dependency, `[[test]] cli`.
- [x] src/cli.rs: `ConfigFlags`, optional command, `meet_bare_invocation`, `load_config`, `take_account`, `welcome`, `json-schema`, root named `mml`, the dead pipeline gates removed.
- [x] src/main.rs: `pimalaya_cli::clap::parse`, the welcome on a missing configuration, `eval_coded`.
- [x] src/cli/config.rs: 0.4 `TomlConfig`, every field documented, the sample test.
- [x] src/cli/error.rs: `ErrorCode`, `coded`, `mml_invalid`.
- [x] src/cli/printer.rs: `event`, `out_bytes`, NDJSON, `defuse`, `rfc822`.
- [x] src/cli/json_schema.rs: the registry and its two checks.
- [x] `InterpretOutput`, `SignatureOutput`, `TemplateOutput`; invalid signature as `PartialOutcome`.
- [x] compile and the edit loop: coded parse failure, diagnostics on stderr, colour on a terminal only.
- [x] The forward headline and the `\n` unescape fixes.
- [x] config.sample.toml in the family shape; package.nix and install.sh ship it; package.nix writes share/schemas.
- [x] deny.toml targets and licences; rustix 1.1.6.
- [x] Clippy fixes in the library (no API change).
- [x] tests/cli.rs on the binary; the invalid signature exit in tests/pgp_gpg.rs; the forward headline test seen failing on the old code.
- [x] CI clippy matrix.
- [x] README, CONTRIBUTING, src/lib.rs and src/main.rs headers, CHANGELOG.
- [x] Fold into cairn/spec/cli.md, layout.md, packaging.md; log; abandon first-run-wizard.
