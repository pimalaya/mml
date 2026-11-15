# Migration guide

## From v1 to v2

v2 brings MML to the Pimalaya family standards: one configuration path per `-c`, sources and destinations as flags, the family JSON output and exit codes, and a library reshaped around I/O-free coroutines. The CHANGELOG lists every change; these are the ones to act on.

### Configuration

- **`MML_CONFIG` is no longer read.** Pass `-c "$MML_CONFIG"` for one path, and one `-c` per path for what was a `:`-separated list.
- **`-c` takes one path per flag.** A value is no longer split on `:`. Repeat the flag to merge several files, the first one the base: `-c public.toml -c private.toml`.
- **A `-c` path that does not exist is refused**, where v1 ran on the defaults. With no `-c`, every default location that exists is read, each merged on top of the previous one, where v1 read the first one found.
- **Several `default = true` accounts are refused**, naming them, where v1 took one at random. Keep it on one account across the merged files.
- **An unknown key fails the configuration**, naming it: a typo no longer passes silently. The shipped config.sample.toml (share/doc/mml/config.sample.toml in releases and packages) lists every key MML reads.

### Command line

- **Sources come from `-i, --input`.** The trailing positional `MESSAGE` is gone: `--input <PATH>` reads a file, stdin stays the default, and `--input -` names it. An implicit stdin that is a terminal is refused rather than left waiting.
- **Destinations go through `-o, --output`.** The leading positional output path of `compose`, `reply` and `forward` is gone. `mml reply <OUTPUT> -- <SOURCE>` becomes `mml reply --output <OUTPUT> --input <SOURCE>`.
- **Commands are singular.** `templates` is `template`, `completions` is `completion`, `manuals` is `manual`. The plurals (and `tpl`) keep working as hidden aliases.
- **Man pages and completions are named `mml`**, where v1 named them `mime-meta-language`. Remove the old files from a packaging script or a shell setup that installed them by that name.

### Output and exit codes

- **`compile`, `compose`, `reply`, `forward` and `mailto` print `{data, size, mime}` under `--json`**, `data` the message in base64 and `mime` `message/rfc822`, where v1 printed a JSON string. Decode it with `jq -r .data | base64 -d`. Text output stays the raw message.
- **`compile` fails on MML that does not parse**, exiting 1 with the failure object coded `mml-invalid`, where v1 exited 0. Its diagnostics go to stderr, and only in text mode.
- **`interpret` exits 3 on an invalid signature**, 0 with no signature or a valid one, 1 on any other failure. Under `--json` it prints `{mml, signatures}`, each signature `{valid, fingerprint}`.
- **Every failure prints the failure object** under `--json`, with a stable `code` (`mml-invalid`, `mime-invalid`, `from-missing`, `pgp-unavailable`, `pgp-failed`, `aborted`, `usage-invalid`, or `null`). Read `code` rather than matching the message. A command line the grammar refuses still exits 2, now printing that object coded `usage-invalid` on stdout under `--json`.
- **Text written to a terminal is sanitized**: control and bidi characters become U+FFFD, line breaks and tabs kept. A file or a pipe still gets the bytes unchanged.
- `json-schema` writes the schema of what each command prints; packages install them under share/schemas.

### Library

- **Every public item carries the `Mml` prefix**: `Template` is `MmlTemplate`, `TemplateCursor` is `MmlTemplateCursor`, `MimeInterpreter` is `MmlMimeInterpreter`, `FilterHeaders` and `FilterParts` are `MmlFilterHeaders` and `MmlFilterParts`, and so on. The shared template types moved from `template::types` up to `template`.
- **The compiler is one call.** `MmlCompilerBuilder::new().build(&mml)?.compile()?.into_vec()?` becomes `MmlCompileOptions::default().compile(&mml)?`, returning the MIME bytes.
- **The interpreter takes options by value.** The builder and its `with_*` setters are gone: build `MmlInterpretOptions` with a struct literal and `..Default::default()`, then call `interpret`, `interpret_builder` or `interpret_msg` (the old `from_*`). `with_show_only_headers` is `MmlFilterHeaders::include`, `with_show_additional_headers` is `MmlFilterHeaders::extend`.
- **`MmlBodyCompiler` is no longer a unit struct**: build it with `MmlBodyCompiler::default()` or a literal setting `format` and `flowed_column`.
- **Reply and forward builders take the rendered quote.** `MmlTemplateReplyBuilder::build` and `MmlTemplateForwardBuilder::build` take it as a second argument; render it with the options their `quote_options()` returns.
- **PGP goes through the coroutines.** `MmlCompile` and `MmlInterpret` yield `WantsPgp*` steps and resume with an `MmlPgpAnswer`; the `gpg` cargo feature provides `pgp::gpg::MmlPgpGpg` to answer them. The direct `compile` and `interpret` methods drive them with no backend, so a part marked `sign` or `encrypt` fails to compile rather than ship in the clear.
- **The binary needs the `cli` feature**, which turns on `compiler` and `interpreter`, so `--no-default-features --features cli` builds.
