---
cairn: log
change: account-codes-and-with-code
landed: 2026-11-15
---

# Account codes, through the shared with_code

Owner rulings of 2026-11-15, items 13 and 24 of the merged list.

## What landed

- src/cli/error.rs: `coded` goes through pimalaya-cli's `with_code` (51ce8b1) in place of its own chain rebuild, reading the code of an `MmlError` or of pimalaya-config's `AccountError`.
- `account-missing` for a name `-a` gives that no account has, `account-default-several` for several accounts marked default. `NoDefault` stays uncoded: MML runs on the global layer alone then, as before.
- pimalaya-config pinned to d5a33cb, which reads `default` as a name like any other: `-a default` names an account called `default`.

## Capabilities moved

- [cli](../spec/cli.md): the failure codes list the two account codes.
