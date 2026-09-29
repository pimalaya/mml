---
cairn: spec
capability: flowed
status: current
---

# Flowed

RFC 3676 `format=flowed` plain text: the compiler flows, the interpreter unflows, and MML always holds logical lines.

### Requirement: MML holds logical lines
A newline in an MML `text/plain` body SHALL be a hard break, one logical line per paragraph. Soft breaks SHALL exist only in the compiled MIME, and nothing in the MML text records that a body was flowed.

### Requirement: The pipelines round trip
Interpreting a compiled body SHALL return the body it was compiled from, for any column and either format, trailing spaces of hard lines aside.

### Requirement: Format is a property of the part
The grammar SHALL carry a `format` property on `<#part>`, accepting `flowed` and `fixed`. It applies to a bare `text/plain` part, typed or guessed, and is ignored elsewhere. `MmlBodyCompiler::format` SHALL be the default for parts carrying none, bare text included, and SHALL default to `fixed`.

### Requirement: The column is configurable
`MmlBodyCompiler::flowed_column` SHALL bound every flowed line, quote marks and stuffing included, and SHALL default to 72. A word longer than the column SHALL be shipped whole. A column of 998 breaks nothing.

### Requirement: Generating follows section 4
Compiling a flowed part SHALL strip trailing spaces from hard lines, break the lines longer than the column on spaces with the space ending each soft line, repeat the quote marks on every line a quoted paragraph breaks into, and space-stuff every line starting with a space, `>` or `From `, a quoted line always carrying the space after its marks. The Usenet signature separator `-- ` SHALL be neither broken nor stripped. The content type SHALL carry `charset=utf-8` and `format=flowed`.

### Requirement: Reading always unflows
Interpreting a part whose content type carries `format=flowed` SHALL count the quote depth, unstuff, and join the soft breaks back into logical lines, with no option to keep the raw form. A depth change and the `-- ` separator both end a paragraph. Unflowing happens before the MML markup is escaped and before the signature is stripped.

### Requirement: DelSp is read, never written
A part carrying `DelSp=yes` SHALL be joined by deleting the marking space. Compilation SHALL NOT emit the parameter.

### Requirement: Crypto parts stay fixed
A part carrying `sign` or `encrypt` SHALL NOT be flowed, whatever its format property says.

### Requirement: Parameters in the type property are opaque
The compiler SHALL NOT inspect the `type` property for a `format` parameter: a `type` carrying parameters reaches the wire verbatim and performs no encoding.
