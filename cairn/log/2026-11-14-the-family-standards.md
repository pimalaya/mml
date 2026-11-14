---
cairn: log
change: the-family-standards
landed: 2026-11-14
---

# MML took the family standards

pimalaya-cli and pimalaya-config moved from 0.2 to 0.4, behind `[patch.crates-io]` path patches on ../cli and ../config: MML uses `pimalaya_cli::clap::parse` and `EXIT_PARTIAL` 3, which pimalaya-cli 0.4.0 lacks, so the patches leave once 0.4.1 is released. The lockfile took rustix 1.1.6 for RUSTSEC-2026-0486.

src/cli.rs flattens `ConfigFlags`, makes the command optional and meets a bare `mml` in `meet_bare_invocation`; `load_config` keeps a missing default configuration the empty one and loads an explicit `-c` through `TomlConfig::load`; `take_account` turns `AccountError::NoDefault` into the global layer alone. src/main.rs parses with `pimalaya_cli::clap::parse`, prints the shared welcome on a `ConfigMissingError` when `welcome::is_wanted`, and evaluates with `ErrorReport::<ErrorCode>::eval_coded`. Checked under `script` with an empty `HOME`: bare `mml` prints the welcome with the `curl` command, and the help once the sample is copied in.

src/cli/error.rs holds `ErrorCode` and `coded`, which `Command::execute` applies to every result, rebuilding the chain with a `CodedError` at its root when it carries an `MmlError` with a code; `mml_invalid` builds the parse failure, drawing the ariadne diagnostics on stderr (they went to stdout through `print` in `compile`) and colouring them only on a terminal. src/cli/printer.rs implements the 0.4 `Printer` (`event`, `out_bytes`), ends every JSON value with a newline and defuses text bound for a terminal through `sanitize::text`, line breaks and tabs kept. src/cli/json_schema.rs registers nine command keys and `mml-error`, beside the three the toolkit registers; `json-schema -d` writes the thirteen.

The registry test found the clap root named `mime-meta-language`, the package name since 1.1.0, so man pages and completions were generated under it while package.nix installs `mml.bash`: the root is `mml` now.

`interpret` returns a `PartialOutcome` after printing on an invalid signature, exiting 3 instead of 2; `SignatureOutput` replaces the serialized `MmlPgpVerdict`, whose `Serialize` derive stays in the library untouched.

config.sample.toml took Ortie's shape, one `[accounts.example]` table with dotted keys, tab-aligned comments at column 40 or the tab stop past a block's longest line. The global layer and the editor note left the sample; README covers them. `the_sample_configuration_parses_with_each_variant` reads every variant through `Config::load`.

`cli` turns `compiler` and `interpreter` on: `--no-default-features --features cli` never built before, its modules needing both. The pipeline gates inside src/cli/ went with it. shellexpand lost `tilde`, the attachment path expanding `~` through `std::env::home_dir`, which drops dirs 6 and its windows-sys. Clippy under Rust 1.96 flagged the library (needless borrows, manual `Option::map`, collapsible `if let`, doc quote continuations), fixed without API change.

Fixed on the way: `template forward -Q` set the reply headline (tests/cli.rs `a_forward_template_takes_its_quote_headline`, seen failing on the old code); `reply -Q` and `forward -Q` now unescape `\n` as the template commands and the help do; the `--save-attachments-dir` help named `~/Downloads` where the default is the system temporary directory.

first-run-wizard is abandoned, superseded by this change.

Checks: `cargo fmt`; `cargo clippy --all-targets -- -D warnings` over the default set, `--all-features`, `--no-default-features`, `compiler`, `interpreter`, `cli` alone and `gpg`; `cargo test --all-features`; `cargo deny check`, three named duplicate warnings.

Capabilities: cli (An invalid signature is an exit code, Signatures ride the payload MODIFIED; The configuration is optional, A bare invocation shows the help or the welcome, Account resolution, Unknown keys are refused, Exit statuses, Failures carry stable codes, A source that does not parse fails, Every output is a described value, Piped JSON is one value per line, Terminal text is defused, The command is mml, The sample is tested ADDED); layout (CLI is optional MODIFIED); packaging created.
