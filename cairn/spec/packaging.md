---
cairn: spec
capability: packaging
status: current
---

# Packaging

How MML is built and shipped.

### Requirement: The sample ships beside the binary
The Nix package SHALL install config.sample.toml as share/doc/mml/config.sample.toml and the JSON Schemas under share/schemas, and install.sh SHALL copy the sample to the same path under its prefix when the downloaded archive holds it.

### Requirement: Every feature set builds clean
Clippy with `-D warnings` SHALL pass over the default set, `--all-features`, `--no-default-features`, `compiler` alone, `interpreter` alone, `cli` alone and `gpg`, checked in CI.

### Requirement: Dependency checks pass clean
Every dependency SHALL be declared with `default-features = false` and the features MML uses. `cargo deny check` SHALL pass over the targets deny.toml lists. A duplicate the tree cannot avoid stays a visible warning: today hashbrown (chumsky and petgraph), syn (ouroboros' macros) and windows-sys (rustix against stacker).
