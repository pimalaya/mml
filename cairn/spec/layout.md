---
cairn: spec
capability: layout
status: current
---

# Layout

MML is a single crate shipping both a library and the `mml` binary. The library holds all the domain logic; the binary is a thin clap shell over it, gated behind the `cli` feature and excluded from the library contract.

### Requirement: Two pipelines
The crate SHALL expose two feature-gated pipelines. The `compiler` feature turns MML markup into a MIME message: its parser submodule tokenizes and parses the `<#part>` / `<#multipart>` grammar, and its message submodule assembles the parts into MIME via mail-builder. The `interpreter` feature walks a parsed MIME message (mail-parser) back into MML or plain text, applying the header and part filters. Both are on by default.

### Requirement: Template layer
The crate SHALL expose a template layer building compose, reply and forward drafts, the starting point both pipelines share. It carries its own shared types (`MmlTemplate`, `MmlTemplateCursor`, `MmlTemplateBody`) in `template.rs` next to its `template/` folder, not in a `types` catch-all. The grammar and header helpers shared by the compiler and interpreter stay private.

### Requirement: PGP seam
The crate SHALL expose the PGP data types in `pgp` and the coroutine contract in `coroutine`, both unconditional. Only the gpg backend, `pgp::gpg`, sits behind a feature. The machinery each pipeline needs to suspend lives next to it: `compiler::crypto` holds the post-order worklist, `interpreter::layer` the layer queue.

### Requirement: CLI is optional
The `mml` binary SHALL live behind the `cli` feature and require it. The editor-driven `compose`, `reply`, `forward` and `mailto` subcommands require the `compiler` and `interpreter` features together. The CLI module carries the clap tree, the TOML configuration, the editor-driven flow and the PGP driver, none of it part of the library API. Its contract is the cli capability.

### Requirement: File layout
The source tree SHALL follow the Pimalaya naming rules: snake_case files, no `#[path]`, and no `foo/mod.rs` plus `foo/foo.rs` inception. A module carrying its own code is the sibling `foo.rs` next to its `foo/` folder; a folder whose `mod.rs` only aggregates stays a pure aggregator.
