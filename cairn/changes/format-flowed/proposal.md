---
cairn: change
id: format-flowed
status: active
created: 2026-08-15
---

# Let plain text be reflowed by the reader rather than by the writer

## Why

Issue [#7](https://github.com/pimalaya/mml/issues/7) asks for RFC 3676 `format=flowed`, with a configurable line length as described in [Format=Flowed with long lines](https://vxlabs.com/2019/08/25/format-flowed-with-long-lines/). The request lands here rather than in Himalaya because flowing is a transformation of a `text/plain` body during MML to MIME compilation, and unflowing is the same transformation during MIME to MML interpretation. Himalaya only assembles what this crate produces, so handling it there would mean parsing the compiled MIME back apart to find the text parts.

The reference implementation already does exactly this in the same place: `mml.el` calls `fill-flowed-encode` inside `mml-generate-mime-1`, guarded by `mml-enable-flowed`, by the type being `text/plain`, by the part not being signed, and by the tag carrying no `format` property or carrying `format=flowed`. `mml-insert-mime-headers` then appends `; format=flowed`. So `format` is a part property of the language this crate implements, not an extension invented here.

Nothing of that exists today. `Part::PlainText` builds a bare `MimePart::new("text/plain", body)` with no properties at all, and the parser has no `format` property. A body written as one long line per paragraph therefore ships as one long line, and a body wrapped by the composer ships hard-wrapped at whatever column the editor chose, which is the failure the RFC exists to remove: the reader's window is not the writer's window.

The read direction is worse and the issue does not mention it. `get_ctype` rebuilds `type/subtype` and drops every Content-Type parameter, so an incoming `format=flowed` message is invisible to the interpreter and its body is rendered raw, trailing spaces and all, either hard-wrapped at 72 columns or as unbroken 998 character lines depending on who sent it.

One accident is worth naming: `<#part type="text/plain; format=flowed">` already reaches the wire, because the quoted value parser accepts `;` and mail-builder writes the content type verbatim before its own attributes. That produces the header and none of the encoding, which is the worst of the two.

## What

MML holds logical text and the wire holds the flowed form. A newline in an MML body is a paragraph break the author typed, one logical line per paragraph, and the compiler is what breaks those lines to fit a column, appending the space that marks each break as soft. The interpreter removes them again. Neither direction needs to record anything in the MML text, since the flowing is a property of the message and not of what was written, and the pair holds the invariant that interpreting a compiled body returns the body.

That convention is forced rather than chosen. Emacs decides which newlines are soft by reading the `hard` text property off the buffer; a `&str` carries no such thing, so the only unambiguous reading is that every newline the author typed is hard.

A `format` property joins the grammar on `<#part>`, taking the two values RFC 3676 defines, `flowed` and `fixed`, and the compile options carry the default for bodies that have no part to hang a property on, which is the common case since bare text compiles to `Part::PlainText`. The column is configurable, defaults to 72, and setting it to 998 is what the linked article does: no line is ever long enough to break, so the paragraphs ship whole and the clients that ignore the RFC reflow them anyway because they are too long to display otherwise.

Generating a flowed body means space-stuffing the lines that would otherwise be misread (section 4.4), breaking the rest at the column with a trailing space, and carrying the quote prefix onto every line a quoted paragraph breaks into. Reading one means counting the quote depth, unstuffing, joining the soft breaks, and honouring `DelSp=yes` by dropping the space it marks. Reading always unflows, with no option to turn it off: rendering a flowed body as raw text is not a preference, it is a bug.

Signed and encrypted parts are not flowed, following `mml.el`. Trailing spaces are load-bearing and mail-builder picks `7bit` for an ASCII body, where an intermediary is free to strip them; RFC 3156 verification needs the signed bytes to arrive verbatim, so a stripped space is an invalid signature rather than a cosmetic loss.

## Scope / non-goals

**No new flags.** The format and the column are preferences that belong in the `[compose]`, `[reply]` and `[forward]` sections of the configuration, like the signature style. A per-invocation override can be added the day someone wants one.

**`DelSp=yes` is read, never written.** It exists for scripts that do not separate words with spaces, and emitting it costs correctness with every client that ignores it.

**The `type` property is not inspected for parameters.** A `format` smuggled into `type="text/plain; format=flowed"` keeps reaching the wire as the opaque string it already is and performs no encoding. The property is the supported spelling, and quietly encoding on the strength of a substring match would make the two spellings differ in behaviour by accident.

**Quoted-printable is not forced on flowed parts.** It would protect the trailing spaces against every intermediary rather than only against the signing case, at the cost of making every plain ASCII message unreadable in its raw form. Emacs does not, so neither does this, and the signing case is covered by not flowing at all.
