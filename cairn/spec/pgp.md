---
cairn: spec
capability: pgp
status: current
---

# PGP

PGP is a property of a part, not a command. Signing and encrypting happen during compilation, decryption and verification during interpretation, and the cryptography itself happens outside the pipelines, in whatever backend the caller serves the yields with.

### Requirement: Properties on the part
The grammar SHALL carry four properties on `<#part>` and `<#multipart>`: `sign` and `encrypt`, whose only accepted value is `pgpmime`, plus `sender` and `recipients`. There is no `<#secure>` directive.

### Requirement: Signer defaults to the sender
The signing identity SHALL come from the part's `sender` property, falling back to the message `From`. Recipients SHALL come from the part's `recipients` property only: configuration cannot know who a given message is addressed to, so there is no recipient default.

### Requirement: Sign before encrypt
When a part asks for both, signing SHALL run first and encrypting second, per RFC 3156. The reverse order leaks who signed to anyone holding the ciphertext.

### Requirement: Deepest part first
Crypto SHALL apply bottom-up: a part is signed or encrypted only once every part it contains is final. The compiler flattens the parsed tree into a post-order worklist rather than recursing, so a suspension never happens inside a walk.

### Requirement: RFC 3156 envelopes
A signed part SHALL wrap in `multipart/signed; protocol="application/pgp-signature"; micalg="pgp-sha256"`, cleartext first and detached signature second. An encrypted part SHALL wrap in `multipart/encrypted; protocol="application/pgp-encrypted"`, a `Version: 1` part first and the OpenPGP message second. Backend output has its bare line feeds normalised to CRLF before embedding.

### Requirement: Interpretation resolves layers, not a stack
Decrypting yields plaintext that parses into another layer, which may carry encrypted parts of its own. The interpreter SHALL iterate a queue of layers, suspending only between them, so nesting depth is bounded by the message rather than the call stack. Each layer owns its bytes through a `Cow` behind ouroboros, since the parsed `mail_parser::Message` borrows them.

### Requirement: Verdicts reach the caller
`MmlInterpret` SHALL collect every signature verdict of a run and expose them once it completes. An invalid signature is a verdict rather than a failure, so the interpretation succeeds and the caller decides what to do with it.

### Requirement: Reading degrades, writing refuses
An `MmlPgpAnswer::Unavailable` on a read path SHALL render the part visibly as `<#part type=application/pgp-encrypted>` rather than dropping it. On a write path it SHALL abort the compilation. Degrading while reading loses nothing; degrading while writing sends a message the user asked to encrypt in the clear, which is a security failure rather than a lesser experience.

### Requirement: Quotes are rendered by the caller
The reply and forward template builders SHALL NOT interpret the source message. They expose `quote_options()` describing how the quote should be rendered and take the rendered quote as an argument, so the caller can drive the coroutine and decrypt an encrypted thread before it is quoted.

### Requirement: GPG is one optional backend
The `gpg` feature, off by default, SHALL provide `pgp::gpg::MmlPgpGpg` serving the four yields through gpgme. It carries an optional `home_dir` so a caller isolates a keyring without touching the process-wide `GNUPGHOME`. Recipient keys resolve through GPG's auto-key-locate, and encryption sets `ALWAYS_TRUST`: a composer cannot require every recipient key to be signed first. Any gpgme failure is an error, never a fallback to cleartext.

### Requirement: End-to-end coverage over a generated store
The PGP suite SHALL generate its GPG store per run rather than checking one in. The pre-2.0 fixture carried keys that expired in 2025 and silently rotted the suite. Coverage spans the four actions, sign-and-encrypt nesting, nested encrypted layers, quoting an encrypted thread, and the failure paths: unknown recipient, empty recipient list, missing secret key, and a tampered signed body reported as an invalid verdict.
