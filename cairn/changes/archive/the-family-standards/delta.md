---
cairn: delta
change: the-family-standards
---

# Delta

## ADDED Requirements

### Requirement: The configuration is optional (cli)
A command finding no configuration at the default locations SHALL run on its defaults and its flags. Every default location that exists SHALL be read, each a layer merged on top of the ones before. `-c <PATH>` SHALL replace them, take one path per flag, repeated to merge several files, the first one the base, and SHALL fail when its first path does not exist. No environment variable supplies a path. MML SHALL write no configuration and SHALL have no setup wizard.

### Requirement: A bare invocation shows the help or the welcome (cli)
A bare `mml` SHALL print the help when a configuration exists, a broken one included. With none, a person (no `--json`, stderr a terminal) SHALL meet pimalaya-cli's welcome on stderr, naming the missing file and the commands creating it from the shipped sample, or downloading it, then `mml compose`, and the process exits 1; a script or a JSON caller SHALL get the failure object. The same holds for a missing `-c` path.

### Requirement: Account resolution (cli)
The account SHALL be the global layer, then the account `-a` names or the one marked default on top. No account marked default SHALL leave the global layer alone. An unknown name SHALL fail listing the accounts the configuration holds, and several defaults SHALL fail naming them.

### Requirement: Unknown keys are refused (cli)
Every configuration table SHALL refuse a key it does not know, naming it.

### Requirement: Exit statuses (cli)
A command SHALL exit 0 on success, 1 after printing the failure object, 2 on a command line the grammar refuses, and 3 when `interpret` met an invalid signature. Under `--json`, a refused command line SHALL print the failure object coded `usage-invalid` on stdout.

### Requirement: Failures carry stable codes (cli)
The failure object SHALL carry `code`: `mml-invalid` for an MML source that does not parse, `mime-invalid` for a MIME source that does not, `from-missing` for a draft with no `From` address, `pgp-unavailable` for a part asking for PGP in a build without a backend, `pgp-failed` for a backend failure, `aborted` for a draft aborted from the post-edit prompt, `usage-invalid` for a refused command line, and `null` otherwise. The library knows none of them: the CLI maps them from `MmlError`.

### Requirement: A source that does not parse fails (cli)
`compile` and the edit loop SHALL fail coded `mml-invalid` on an MML source that does not parse, each parse error a source of the failure. In text mode, each error SHALL first be drawn on stderr against the source, coloured only when stderr is a terminal.

### Requirement: Every output is a described value (cli)
`compile`, `compose`, `reply`, `forward` and `mailto` SHALL print the compiled message as bytes, raw in text mode and `{data, size, mime}` under `--json`, `data` in base64 and `mime` `message/rfc822`. `interpret` SHALL print `{mml, signatures}`, each signature `{valid, fingerprint}`, `fingerprint` `null` when invalid. A template command SHALL print `{content, cursor: {row, col}}`. The text form of these is the artifact alone. `json-schema` SHALL write their schemas, `mml-error` the failure object's, and a test SHALL check the registry against the command tree.

### Requirement: Piped JSON is one value per line (cli)
JSON SHALL be pretty on a terminal and compact elsewhere, each value followed by a newline.

### Requirement: Terminal text is defused (cli)
Text written to a terminal SHALL have its control and bidi characters replaced with U+FFFD, line feeds, CRLF and tabs kept. Text written to a file or a pipe SHALL be the artifact unchanged.

### Requirement: The command is mml (cli)
The clap root SHALL be named `mml`, so help, man pages and completions carry the binary's name rather than the package's.

### Requirement: The sample is tested (cli)
config.sample.toml SHALL hold one `[accounts.example]` table and no other, the defaults active and the alternatives commented, one comment per line. A test SHALL read it through the loader `-c` uses, as written and with each commented line uncommented through `pimalaya_config::sample::variants`.

### Requirement: The sample ships beside the binary (packaging)
The Nix package SHALL install config.sample.toml as share/doc/mml/config.sample.toml and the JSON Schemas under share/schemas, and install.sh SHALL copy the sample to the same path under its prefix when the downloaded archive holds it.

### Requirement: Every feature set builds clean (packaging)
Clippy with `-D warnings` SHALL pass over the default set, `--all-features`, `--no-default-features`, `compiler` alone, `interpreter` alone, `cli` alone and `gpg`, checked in CI.

### Requirement: Dependency checks pass clean (packaging)
Every dependency SHALL be declared with `default-features = false` and the features MML uses. `cargo deny check` SHALL pass over the targets deny.toml lists. A duplicate the tree cannot avoid stays a visible warning: today hashbrown (chumsky and petgraph), syn (ouroboros' macros) and windows-sys (rustix against stacker).

## MODIFIED Requirements

### Requirement: An invalid signature is an exit code (cli)
`interpret` SHALL exit 0 when the message carries no signature or a valid one, 3 when a signature is invalid, and 1 on any other failure. The message renders either way: a bad signature is a reason to distrust it, not a reason to hide it. The verdict never enters the rendered body, where it would be indistinguishable from text the sender wrote.

### Requirement: Signatures ride the payload, not stderr (cli)
The verdicts SHALL be fields of the value `interpret` prints, whose text form stays the raw MML so stdout remains what a composer can consume, and whose `--json` form carries a `signatures` array of `{valid, fingerprint}`. Nothing about them goes to stderr, which carries logs and parse diagnostics alone.

### Requirement: CLI is optional (layout)
The `mml` binary SHALL live behind the `cli` feature and require it, and `cli` SHALL turn `compiler` and `interpreter` on. The CLI module carries the clap tree, the TOML configuration, the editor-driven flow, the PGP driver, the error codes and the JSON Schemas, none of it part of the library API. Its contract is the cli capability.

## REMOVED Requirements
