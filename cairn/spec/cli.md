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
Driving the coroutines and answering their PGP yields SHALL happen in one CLI module, gated once on the `pgp-gpg` feature at the item rather than inside a function body. Without the feature every step answers `Unavailable`, and the library decides what that means per direction.

### Requirement: An invalid signature is an exit code
`interpret` SHALL exit 0 when the message carries no signature or a valid one, 2 when a signature is invalid, and 1 on any other failure. The message renders either way: a bad signature is a reason to distrust it, not a reason to hide it. The verdict never enters the rendered body, where it would be indistinguishable from text the sender wrote.

### Requirement: Signatures ride the payload, not stderr
The verdicts SHALL be fields of the value `interpret` prints, whose plain rendering stays the raw MML so stdout remains what a composer can consume, and whose `--json` form carries a `signatures` array. Nothing about them goes to stderr, which carries logs alone.
