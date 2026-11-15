---
cairn: tasks
change: format-flowed
---

# Tasks

- [x] Add `FORMAT`, `FLOWED` and `FIXED` to `grammar.rs` and a `format()` property parser accepting the two RFC 3676 values, wired into the `<#part>` property `choice`.
- [x] Give `MmlBodyCompiler` its options: the default format and the flowed column, defaulting to `fixed` and 72.
- [x] Write the flowed encoder: space-stuff the lines starting with a space, `>` or `From `, break the rest at the column on word boundaries with a trailing space, repeat the quote prefix on every line a quoted paragraph breaks into, and leave `-- ` alone.
- [x] Apply it in `compile_part` to `Part::PlainText` and to `Part::Single` bodies typed `text/plain`, adding the `format=flowed` content type attribute, and skip parts carrying `sign` or `encrypt`.
- [x] Read the content type parameters on the interpret side: `get_ctype` keeps returning the bare type, and the `format` and `delsp` attributes reach `interpret_text_plain` separately.
- [x] Write the flowed decoder: count the quote depth, unstuff, join the soft breaks, drop the marking space under `DelSp=yes`, never join across a depth change or into `-- `.
- [x] Unflow before escaping the MML markup and before the signature is stripped, so both keep working on logical lines.
- [x] Add `format` and `flowed-column` to the `[compose]`, `[reply]` and `[forward]` configuration sections and thread them into the compile options.
- [x] Document the property in the README feature list and in `config.sample.toml`.
- [x] Tests: the round trip holds for a paragraph, a quoted reply and a signature block; a stuffed line survives both directions; a column of 998 breaks nothing; `DelSp=yes` decodes; a signed part is not flowed; a `fixed` part is untouched.
- [x] Build/test/fmt/clippy.
- [x] Fold into a new `cairn/spec/flowed.md` and into `cairn/spec/cli.md`; log; land.
