---
cairn: change
change: first-run-wizard
---

# Delta

## ADDED Requirements

### Requirement: A named command generates an identity
A `configure` command (alias `wizard`) SHALL ask for the address messages are sent from, the display name paired with it, and an optional signature, then hand the resulting `[accounts.<name>]` table back as a file to create, a block to append, or a document on stdout. Nothing is discovered, an identity being typed rather than found. It refuses to run when stdin is not a terminal, naming the sample configuration to write by hand instead.

### Requirement: A bare invocation meets the newcomer
A bare `mml` SHALL raise an offer to generate a configuration when it finds none, behind a welcome naming the file that is missing, and show the help when it finds one. `--account` with no subcommand is a half-typed command and shows the help too. Nothing is offered when stdin is not a terminal or `--json` is set, and a declined offer falls back to the help.

### Requirement: The offer is not raised from commands
A command finding no configuration SHALL NOT offer the wizard, because it does not need one: the configuration is optional and an account only supplies identity defaults every flag can override. A command with no account uses the global section, which is correct rather than a failure. The offer therefore belongs to the bare invocation alone.

### Requirement: Generating never rewrites what a human wrote
The wizard SHALL write a configuration file that does not exist and append a plain text block to one that does, never parsing and re-serializing the document, so comments, ordering and formatting survive. The account name is derived from the address rather than prompted, and suffixed until the configuration does not already hold it, since a second `[accounts.<name>]` table makes the whole document fail to parse. The generated account claims `default` only when no other account does, since two defaults resolve to whichever the account map yields first.

### Requirement: The welcome names the missing path
The welcome SHALL name the configuration path that was looked for, which is the one `-c` or `MML_CONFIG` gave or the default location, so a mistyped path shows up as itself rather than as a generic first run. It frames the product, points at the documented sample, and names the command that runs the wizard again later.

### Requirement: A missing named account names the ones that exist
Selecting an account the configuration does not hold SHALL fail with a message listing the accounts it does hold. There is no matching failure for a missing default: no default is the global section, not an error.

## MODIFIED Requirements

## REMOVED Requirements
