---
cairn: change
id: the-family-standards
status: landed
created: 2026-11-14
---

# MML takes the family standards

## Why

MML releases as 1.2.0 with the rest of the Pimalaya family, which moved between 2026-11-08 and 2026-11-14 onto one standard (ortie, himalaya and comodoro cairn/log; pimalaya-cli cairn/spec output.md, error.md and terminal.md; pimalaya-config 0.4). MML stopped on 2026-09-29 (master 6055ed9):

- pimalaya-cli 0.2 and pimalaya-config 0.2, whose `TomlConfig` and `Printer` changed shape in 0.4;
- `-c` read `MML_CONFIG` and split on `:`, and a mistyped `-c` path silently ran on the defaults;
- several `default = true` accounts resolved to whichever the map yielded first;
- the first-run-wizard change (proposed 2026-08-14) was never built, and the family dropped wizards since;
- a bare `mml` was a usage error;
- clap's own parse, so a refused command line under `--json` printed text on stderr;
- `interpret` exited 2 on an invalid signature, the status the family now keeps for a refused command line;
- `compile` drew a parse error on stdout (ariadne's `print`) and exited 0;
- the failure object's `code` was always `null`;
- `--json` printed a compiled message as a JSON string, a signature verdict as `{"valid": "<fingerprint>"}` or `"invalid"`, and no command had a schema;
- piped JSON carried no trailing newline;
- text rendered from a message reached the terminal raw, escape sequences included;
- the clap root was named `mime-meta-language`, the package name, so man pages and completions came out under that name;
- a prose config.sample.toml, neither tested nor shipped; deny.toml without targets; clippy failing under Rust 1.96; `--no-default-features --features cli` not building; a rustix advisory in the lockfile.

## What

One change, its pieces in the order Comodoro landed them:

1. The wizard leaves: first-run-wizard is abandoned. The configuration stays optional, as cairn/spec/cli.md always said. A bare `mml` shows the help when a configuration exists and the shared welcome otherwise.
2. The shared kit: pimalaya-cli and pimalaya-config 0.4 (local path patches until the releases); `ConfigFlags` (one path per `-c`, no environment variable); `TomlConfig::load` for an explicit `-c`, so a missing one is reported; `take_account`, no default account leaving the global layer alone; `pimalaya_cli::clap::parse`; `json-schema` with a registry checked against the command tree.
3. The outputs become data: a compiled message prints through `out_bytes` as `message/rfc822` `Bytes`; `InterpretOutput` carries `signatures` as `{valid, fingerprint}`; `TemplateOutput` keeps its `{content, cursor}` shape. Text written to a terminal is defused; a file or a pipe gets the artifact unchanged.
4. Stable error codes: `mml-invalid`, `mime-invalid`, `from-missing`, `pgp-unavailable`, `pgp-failed`, `aborted`, mapped from `MmlError` in the CLI so the library stays free of pimalaya-cli.
5. Exit statuses: 0, 1 with the failure object, 2 for a refused line, 3 for an invalid signature (a `PartialOutcome`).
6. The sample is tested and shipped: config.sample.toml in Ortie's shape, read through `pimalaya_config::sample::variants`, installed as share/doc/mml/config.sample.toml by package.nix and install.sh, beside share/schemas.
7. Every feature set builds clean: `cli` turns `compiler` and `interpreter` on, clippy `-D warnings` over seven sets in CI, `cargo deny check` clean.

Found on the way and fixed: `template forward -Q` set the reply headline; `reply -Q` and `forward -Q` did not unescape `\n` as their help says; `--save-attachments-dir` claimed `~/Downloads` as its default where the code uses the system temporary directory.

## Scope / non-goals

- No REPL and no MCP server: a session over one account brings nothing to a filter whose commands each read one source.
- `--output` keeps its spec'd meaning (the sink of the whole printer, `--json` included) rather than the family's write receipt `{path, size}`; changing it is the owner's call.
- mail-builder and mail-parser stay; moving to mime-rs is a later change.
- The library API is untouched: himalaya-tui pins master.
- No `SimdCpu::install_detected`: the base64 of a compiled message is small, the portable codec suffices.
- No version bump, no release.
