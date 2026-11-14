---
cairn: tasks
change: first-run-wizard
---

# Tasks

- [ ] Add a `configure` command (alias `wizard`) asking for the from address, the display name and an optional signature, with a non-interactive bail.
- [ ] Derive the account name from the address, suffixed until the configuration does not already hold it; claim `default` only when no other account does.
- [ ] Write a configuration file that does not exist, append a plain text block to one that does, print when neither is possible.
- [ ] Render the account by hand, one key per line, so a generated document reads in a deliberate order.
- [ ] Add `print_welcome`, naming the configuration path that was looked for, the sample URL and the `mml configure` pointer.
- [ ] Make `command` optional, and meet a bare `mml`: offer when unconfigured, help otherwise, help when `--account` names one.
- [ ] Add `offer_configuration`, raised by the bare invocation alone, returning whether the wizard ran.
- [ ] Guard both entry points on `stdin().is_terminal()` and `printer.is_json()`.
- [ ] Name the missing named account, listing the accounts the configuration holds.
- [ ] Report where the account landed, under which name, and what to run next.
- [ ] Tests: a generated account parses back, an appended one keeps the existing account and its comments, a taken name gets a suffix, a missing configuration constrains nothing.
- [ ] Build/test/fmt/clippy.
- [ ] Fold into cairn/spec/cli.md; log; land.
