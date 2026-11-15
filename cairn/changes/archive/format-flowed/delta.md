---
cairn: change
change: format-flowed
---

# Delta

## ADDED Requirements

### Requirement: MML holds logical lines
A newline in an MML `text/plain` body SHALL be a hard break the author typed, one logical line per paragraph, and soft breaks SHALL exist only in the compiled MIME. Nothing in the MML text records that a body was flowed, the flowing being a property of the message rather than of what was written. Emacs distinguishes hard from soft newlines through a buffer text property that a `&str` cannot carry, so any other reading would have to guess which breaks the author meant.

### Requirement: The pipelines round trip
Interpreting a compiled body SHALL return the body it was compiled from, for any column and either format. The pair is one transformation and its inverse, which is what makes both halves belong to this crate rather than to its callers.

### Requirement: Format is a property of the part
The grammar SHALL carry a `format` property on `<#part>`, accepting the two values RFC 3676 defines, `flowed` and `fixed`. It applies to `text/plain` parts and is ignored elsewhere. The compile options SHALL carry the default for bodies with no part to hang a property on, since bare text compiles to a `Part::PlainText` that has no properties at all.

### Requirement: The column is configurable
The flowed column SHALL be an option defaulting to 72, which leaves room for the quote prefixes a reply accumulates under the 78 characters RFC 3676 section 4.2 recommends. A column of 998, the RFC 5322 line limit, breaks nothing and ships every paragraph whole, which is how a reader that ignores the format still reflows it.

### Requirement: Generating follows section 4
Compiling a flowed part SHALL space-stuff every line starting with a space, `>` or `From `, break the remaining lines at the column on a word boundary, end each broken line with the space that marks it soft, and repeat the quote prefix on every line a quoted paragraph breaks into. The Usenet signature separator `-- ` SHALL never be broken nor stuffed: it already ends in a space and would otherwise read as a flowed line.

### Requirement: Reading always unflows
Interpreting a part whose content type carries `format=flowed` SHALL count the quote depth, unstuff, and join the soft breaks back into logical lines, with no option to keep the raw form. A depth change and the `-- ` separator both end a paragraph, so a quote and a signature never absorb the text around them. Unflowing happens before the MML markup is escaped and before the signature is stripped, so both keep operating on logical lines.

### Requirement: DelSp is read, never written
A part carrying `DelSp=yes` SHALL be joined by deleting the marking space rather than keeping it. Compilation SHALL NOT emit the parameter: it exists for scripts that do not separate words with spaces, and every client that ignores it renders the joined words fused.

### Requirement: Crypto parts stay fixed
A part carrying `sign` or `encrypt` SHALL NOT be flowed, whatever its format property says, as `mml.el` does. A signed body stays byte-for-byte what its author wrote. Transport alone would allow it: mail-builder ships any body with a trailing space as quoted-printable.

### Requirement: Parameters in the type property are opaque
The compiler SHALL NOT inspect the `type` property for a `format` parameter. A `type="text/plain; format=flowed"` keeps reaching the wire as the opaque string mail-builder writes verbatim, and performs no encoding. Encoding on the strength of a substring would make the two spellings of the same intent behave differently by accident.

### Requirement: Format and column are configuration
The `[compose]`, `[reply]` and `[forward]` configuration sections SHALL carry `format` and `flowed-column`, and no flag SHALL duplicate them. They are preferences of how a composer writes, like the signature style, not choices made per invocation.

## MODIFIED Requirements

## REMOVED Requirements
