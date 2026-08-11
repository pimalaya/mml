---
cairn: log
change: v2-api-reshape
landed: 2026-08-10
---

# Reshape the public API for 2.0

Took the breaking changes deferred on 2026-07-26 and landed them together, plus the CLI stream contract and the coroutine seam they enabled. Three spec capabilities appeared: coroutines, cli, and an amended layout.

The `Mml` prefix rename went through (naming-006/007): `Template` and friends, the `Template*Style` enums, `MimeInterpreter*`, and `FilterHeaders` / `FilterParts`. The shared template types left the banned `template::types` catch-all and moved up into `template.rs` (naming-005). himalaya-tui, the only consumer, was updated in step.

The builders went. `MmlCompilerBuilder` carried a single private field and zero setters, so the three-type chain to reach `compile` was pure ceremony; `MmlMimeInterpreterBuilder` carried 22 `with_*` setters, nine of them pure delegation to the body interpreter. Both collapsed into options structs of public fields, `MmlCompileOptions` and `MmlInterpretOptions`, which then became the second argument of the `MmlCompile` and `MmlInterpret` coroutines. The setters carrying real logic moved onto the filter as `MmlFilterHeaders::include` and `::extend`. Net across the two modules: 217 added, 449 removed.

The CLI gained the explicit stream contract: a global `-o, --output`, a per-command `-i, --input`, `-` naming the stream, and `--output` resolving into the printer's sink so it composes with `--json` rather than competing with it. The trailing positional `MESSAGE` was removed: it was reachable only after `--`, silently lost to `--input`, and swallowed any flag typed after it. `mailto` came back as a real subcommand, reusing the compose path rather than the 246 lines of parallel routing the pre-2.0 attempt carried. The seven template-building commands now share one identity arg group.

Two live defects surfaced and were fixed. `compose`, `reply` and `forward` hung when their output landed on a redirected stdout, since the editor inherits that stream; they now refuse to start with an error naming the fix. `interpret` and `read` printed raw control characters where an encrypted part belonged, because the PGP-free path collected no request and never resolved its placeholder.

Guidelines work rode along: `default-features = false` on every dependency, SECURITY.md, module headers on the four modules missing them, docs on every public item, `use crate::` in place of the banned `use super::`, and the library code snippets out of the README.
