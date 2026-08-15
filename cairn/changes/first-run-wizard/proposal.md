---
cairn: change
id: first-run-wizard
status: active
created: 2026-08-14
---

# Meet the newcomer with a wizard rather than with nothing

## Why

MML has no wizard, and unlike the other four products it does not obviously need one: the configuration is optional. `Config::from_paths_or_default(...)?.unwrap_or_default()` means every command runs with no configuration at all, and an account only supplies identity defaults every flag can override. Nothing is a dead end, so there is nothing to rescue.

What there is instead is a discoverability gap. An identity is `from`, `from_name` and a signature, and nothing in the tool says so: a first run composes a message with no From header and no signature, silently, and the way to fix that is to find config.sample.toml. Comodoro was in the same position, with nothing to discover and a promise to keep, and answered it with two questions and a file.

The one real failure is unhelpful. `mml -a nope compose` fails with `Get account \`nope\` error`, the shared default, which names neither the accounts that do exist nor how to pick one.

`mml` with no subcommand is a clap usage error rather than anything useful, so the invocation a newcomer types first tells them the least.

## What

An `mml configure` command (alias `wizard`) asks the few questions an identity is made of: the email address it sends from, the display name to pair with it, and an optional signature. Nothing is discovered, because nothing here is discoverable.

The account name comes from the address rather than being prompted, suffixed until the configuration does not already hold it. The generated account claims `default` only when no other one does.

The wizard writes a configuration file that does not exist, appends a plain text block to one that does, and prints the account on stdout whenever it cannot write, so `mml configure > config.toml` works.

`command` becomes optional, so a bare `mml` has somewhere to land: with no configuration it raises the offer, behind a welcome naming the file that is missing; with one it shows the help. `--account` with no subcommand is a half-typed command and shows the help too.

Nothing prompts when stdin is not a terminal or `--json` is set.

The named-account failure lists the accounts the configuration holds.

## Scope / non-goals

**The offer is not raised from commands.** In the other four products it is, because nothing can happen without a configuration; here everything can. A command that finds no account uses the global defaults, which is correct behaviour and stays. So the offer belongs to the bare invocation alone, and `unwrap_or_default` is left exactly as it is. Raising it from every command would be inventing a dead end in order to rescue the user from it.

For the same reason there is no "missing default" failure to name: no default is not an error, it is the global section.

`MML_CONFIG` already exists and already splits on `:`, so nothing to do there.

The wizard covers the identity and stops. The compose, reply and forward sections, the PGP block and the per-account overrides stay hand-written against the documented sample: they are preferences with defaults, not the thing standing between a newcomer and a working first run.
