---
cairn: spec
capability: cli
status: current
---

# CLI

The `mml` binary is a thin shell driving the library coroutines. It owns the streams, the editor and the PGP backend; the library owns everything else.

### Requirement: Streams are explicit
Every command SHALL read its source from stdin and write its artifact to stdout by default. `-i, --input <PATH>` reads a file instead, `-o, --output <PATH>` writes one, and `-` names the stream explicitly in either. A file actually called `-` is addressed as `./-`. There is no positional message argument: a trailing var-arg was reachable only after `--`, silently lost to `--input`, and swallowed any flag typed after it.

### Requirement: Output and format are independent
`--output` SHALL pick where the payload lands and `--json` SHALL pick its format, so `--json --output result.json` writes JSON to that file. `--output` is global and resolves into the printer's sink, so one implementation serves every command.

### Requirement: Errors go to stdout
The error report SHALL always go to stdout, even when `--output` redirects the artifact to a file. A failure belongs on the stream the user is watching, not appended to a half-written artifact.

### Requirement: An editor needs a terminal
`compose`, `reply`, `forward` and `mailto` SHALL refuse to start when their artifact would land on a stdout that is not a terminal. The editor inherits that stream, so a pipe swallows its interface. Passing `--output <PATH>`, including a process substitution such as `>(consumer)`, keeps stdout free.

### Requirement: Identity flags are shared
The seven commands building a template SHALL share `--from`, `--from-name` and `--signature` through one flattened arg group, so their help text and behaviour cannot drift. The signature style stays per command, since reply accepts quote-relative placements the others do not.

### Requirement: mailto is the compose path
`mailto <URI>` SHALL parse an RFC 6068 URI into the same header set the `compose` flags produce, then run the compose path. Recipients come from the path and the `to` parameter; `subject`, `cc`, `bcc` and `in-reply-to` become headers; `body` pre-fills the draft. Every other key is dropped, including `from`, as RFC 6068 section 5 advises against honouring arbitrary headers from an untrusted URI.

### Requirement: The CLI serves PGP
Driving the coroutines and answering their PGP yields SHALL happen in one CLI module, gated once on the `gpg` feature at the item rather than inside a function body. Without the feature every step answers `Unavailable`, and the library decides what that means per direction.

### Requirement: An invalid signature is an exit code
`interpret` SHALL exit 0 when the message carries no signature or a valid one, 3 when a signature is invalid, and 1 on any other failure. The message renders either way: a bad signature is a reason to distrust it, not a reason to hide it. The verdict never enters the rendered body, where it would be indistinguishable from text the sender wrote.

### Requirement: Signatures ride the payload, not stderr
The verdicts SHALL be fields of the value `interpret` prints, whose text form stays the raw MML so stdout remains what a composer can consume, and whose `--json` form carries a `signatures` array of `{valid, fingerprint}`. Nothing about them goes to stderr, which carries logs and parse diagnostics alone.

### Requirement: Format and column are configuration
The `[compose]`, `[reply]` and `[forward]` configuration sections SHALL carry `format` and `flowed-column`, threaded into the compile options of the matching command, and no flag SHALL duplicate them. `compile` takes none and compiles fixed.

### Requirement: The configuration is optional
A command finding no configuration at the default locations SHALL run on its defaults and its flags. Every default location that exists SHALL be read, each a layer merged on top of the ones before. `-c <PATH>` SHALL replace them, take one path per flag, repeated to merge several files, the first one the base, and SHALL fail when its first path does not exist. No environment variable supplies a path. MML SHALL write no configuration and SHALL have no setup wizard.

### Requirement: A bare invocation shows the help or the welcome
A bare `mml` SHALL print the help when a configuration exists, a broken one included. With none, a person (no `--json`, stderr a terminal) SHALL meet pimalaya-cli's welcome on stderr, naming the missing file and the commands creating it from the shipped sample, or downloading it, then `mml compose`, and the process exits 1; a script or a JSON caller SHALL get the failure object. The same holds for a missing `-c` path.

### Requirement: Account resolution
The account SHALL be the global layer, then the account `-a` names or the one marked default on top. No account marked default SHALL leave the global layer alone. An unknown name SHALL fail listing the accounts the configuration holds, and several defaults SHALL fail naming them.

### Requirement: Unknown keys are refused
Every configuration table SHALL refuse a key it does not know, naming it.

### Requirement: Exit statuses
A command SHALL exit 0 on success, 1 after printing the failure object, 2 on a command line the grammar refuses, and 3 when `interpret` met an invalid signature. Under `--json`, a refused command line SHALL print the failure object coded `usage-invalid` on stdout.

### Requirement: Failures carry stable codes
The failure object SHALL carry `code`: `mml-invalid` for an MML source that does not parse, `mime-invalid` for a MIME source that does not, `from-missing` for a draft with no `From` address, `pgp-unavailable` for a part asking for PGP in a build without a backend, `pgp-failed` for a backend failure, `aborted` for a draft aborted from the post-edit prompt, `account-missing` for a name `-a` gives that no account has, `account-default-several` for several accounts marked default and none named, `usage-invalid` for a refused command line, and `null` otherwise. No account marked default is no failure: the global layer applies alone. The library knows none of them: the CLI maps them from `MmlError` and pimalaya-config's `AccountError`, through pimalaya-cli's `with_code`.

### Requirement: A source that does not parse fails
`compile` and the edit loop SHALL fail coded `mml-invalid` on an MML source that does not parse, each parse error a source of the failure. In text mode, each error SHALL first be drawn on stderr against the source, coloured only when stderr is a terminal.

### Requirement: Every output is a described value
`compile`, `compose`, `reply`, `forward` and `mailto` SHALL print the compiled message as bytes, raw in text mode and `{data, size, mime}` under `--json`, `data` in base64 and `mime` `message/rfc822`. `interpret` SHALL print `{mml, signatures}`, each signature `{valid, fingerprint}`, `fingerprint` `null` when invalid. A template command SHALL print `{content, cursor: {row, col}}`. The text form of these is the artifact alone. `json-schema` SHALL write their schemas, `mml-error` the failure object's, and a test SHALL check the registry against the command tree.

### Requirement: Piped JSON is one value per line
JSON SHALL be pretty on a terminal and compact elsewhere, each value followed by a newline.

### Requirement: Terminal text is defused
Text written to a terminal SHALL have its control and bidi characters replaced with U+FFFD, line feeds, CRLF and tabs kept. Text written to a file or a pipe SHALL be the artifact unchanged.

### Requirement: The command is mml
The clap root SHALL be named `mml`, so help, man pages and completions carry the binary's name rather than the package's.

### Requirement: The sample is tested
config.sample.toml SHALL hold one `[accounts.example]` table and no other, the defaults active and the alternatives commented, one comment per line. A test SHALL read it through the loader `-c` uses, as written and with each commented line uncommented through `pimalaya_config::sample::variants`.
