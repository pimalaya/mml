---
cairn: log
change: adopt-cairn-and-align-guidelines
landed: 2026-07-26
---

# Adopt Cairn and align with the Pimalaya guidelines

Seeded the Cairn root and brought the repository in line with the Pimalaya documentation and naming guidelines, without any breaking public-API change.

The current, landed truth became one spec capability: layout (the two feature-gated pipelines, the template layer, the optional CLI, and the file-layout rules). It was seeded from the src/lib.rs header.

Code changes: replaced the README-as-rustdoc include in src/lib.rs with a real architecture header (header-003), and collapsed two `foo/mod.rs` plus `foo/foo.rs` inception layouts into sibling modules, src/cli.rs and src/cli/template.rs (naming-002). Public module paths are unchanged.

Documentation changes: reordered the README so AI disclosure precedes License, added a Contributing section, and tightened the AI disclosure block to the standard shape (no blank lines between items, no pinned model version). Added a repo-local CONTRIBUTING.md documenting the lib+CLI feature matrix, and this AGENTS.md activation stanza.

The breaking `Mml`-prefix rename (naming-006/007) is deferred: it changes the public API consumed by himalaya and himalaya-tui and belongs to a coordinated major release.

This is a documentation and layout reorganisation with no behaviour change.
