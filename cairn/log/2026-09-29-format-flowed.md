---
cairn: log
change: format-flowed
landed: 2026-09-29
---

# Plain text reflowed by the reader

Issue #7. RFC 3676 `format=flowed` in both pipelines, since mail-parser decodes none (checked through 0.11.9, the latest).

The encoder (src/compiler/flowed.rs) only ever splits: hard lines keep their breaks, lines over the column break on spaces with the soft marker, quote marks repeat on every piece, stuffing counts in the column, and `-- ` is left alone. `compile_part` applies it to `Part::PlainText` and to a bare `text/plain` `Part::Single`, unless the part carries `sign` or `encrypt`. `MmlBodyCompiler` stopped being a unit struct to carry `format` and `flowed_column`, and `get_or_guess_content_type` returns a `String` so the type can be compared.

The decoder (src/interpreter/flowed.rs) joins soft breaks at equal depth and honours `DelSp=yes`; `interpret_text_plain` now takes the part to read `format` and `delsp` from its content type.

Two facts corrected the proposal. mail-builder adds `charset=utf-8` only to a content type with no attribute, so the flowed type sets both. And a flowed part never ships `7bit`: mail-builder picks quoted-printable as soon as a line ends in a space, so soft breaks travel as `=20`. The crypto skip stays, for parity with `mml.el`, not for transport.

The CLI reads `format` and `flowed-column` from `[compose]`, `[reply]` and `[forward]` through `editor::compile_options`.

## Capabilities moved

- [flowed](../spec/flowed.md): new.
- [cli](../spec/cli.md): *Format and column are configuration* added.
