//! End-to-end PGP over a throwaway GPG store.
//!
//! The store is generated per run rather than checked in: the legacy
//! fixture in pimalaya/core/mml carried keys that expired in 2025 and
//! silently rotted the suite. Generating costs a second and never
//! goes stale.
//!
//! Requires the `gpg` binary, which the nix dev shell provides.

#![cfg(feature = "pgp-gpg")]

use std::{fs, path::PathBuf, process::Command};

use mail_parser::MessageParser;
use mml::{
    compiler::message::{MmlCompile, MmlCompileOptions},
    coroutine::MmlYield,
    interpreter::{layer::MmlInterpret, message::MmlInterpretOptions},
    pgp::{MmlPgpAnswer, MmlPgpVerdict, gpg::MmlPgpGpg},
    template::reply::MmlTemplateReplyBuilder,
};

/// A throwaway GPG store holding Alice and Bob.
struct Store {
    home_dir: PathBuf,
}

impl Store {
    /// Generates a store with a secret key per identity.
    fn new(name: &str, identities: &[&str]) -> Self {
        let home_dir = std::env::temp_dir().join(format!("mml-pgp-{name}"));

        let _ = fs::remove_dir_all(&home_dir);
        fs::create_dir_all(&home_dir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&home_dir, fs::Permissions::from_mode(0o700)).unwrap();
        }

        for identity in identities {
            let status = Command::new("gpg")
                .env("GNUPGHOME", &home_dir)
                .args(["--batch", "--pinentry-mode", "loopback", "--passphrase", ""])
                .args(["--quick-gen-key", identity, "default", "default", "never"])
                .status()
                .expect("gpg must be on PATH");

            assert!(status.success(), "cannot generate a key for {identity}");
        }

        Self { home_dir }
    }

    /// The backend serving this store.
    fn gpg(&self) -> MmlPgpGpg {
        MmlPgpGpg {
            home_dir: Some(self.home_dir.clone()),
        }
    }

    /// Compiles MML to MIME, serving every PGP step.
    fn compile(&self, mml: &str) -> Result<Vec<u8>, String> {
        let gpg = self.gpg();
        let mut coroutine = MmlCompile::new(mml, MmlCompileOptions::default());

        drive(|answer| coroutine.resume(answer), &gpg)
    }

    /// Interprets MIME back to MML, serving every PGP step.
    fn interpret(&self, mime: &[u8]) -> Result<String, String> {
        let gpg = self.gpg();
        let mut coroutine = MmlInterpret::new(mime.to_vec().into(), MmlInterpretOptions::default());

        drive(|answer| coroutine.resume(answer), &gpg)
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        let _ = Command::new("gpgconf")
            .env("GNUPGHOME", &self.home_dir)
            .args(["--kill", "all"])
            .status();
        let _ = fs::remove_dir_all(&self.home_dir);
    }
}

/// Runs a coroutine to completion against the given backend.
fn drive<T>(
    mut resume: impl FnMut(Option<MmlPgpAnswer>) -> MmlYield<T>,
    gpg: &MmlPgpGpg,
) -> Result<T, String> {
    let mut answer = None;

    loop {
        let yielded = resume(answer.take());

        match gpg.serve(&yielded) {
            Some(Ok(served)) => answer = Some(served),
            Some(Err(err)) => return Err(err.to_string()),
            None => {
                return match yielded {
                    MmlYield::Ok(output) => Ok(output),
                    MmlYield::Err(err) => Err(err.to_string()),
                    _ => unreachable!("serve only returns None on a terminal yield"),
                };
            }
        }
    }
}

/// Wraps a body in a minimal MML message.
fn mml(body: &str) -> String {
    format!("From: alice@localhost\r\nTo: bob@localhost\r\nSubject: subject\r\n\r\n{body}")
}

#[test]
fn encrypt_then_decrypt() {
    let store = Store::new("roundtrip", &["Alice <alice@localhost>"]);

    let mime = store
        .compile(&mml(
            "<#part encrypt=pgpmime recipients=alice@localhost>secret<#/part>",
        ))
        .unwrap();
    let mime_str = String::from_utf8_lossy(&mime);

    assert!(mime_str.contains("multipart/encrypted"), "{mime_str}");
    assert!(mime_str.contains("BEGIN PGP MESSAGE"), "{mime_str}");
    assert!(
        !mime_str.contains("secret"),
        "the cleartext must not survive"
    );

    assert!(store.interpret(&mime).unwrap().contains("secret"));
}

#[test]
fn the_signer_defaults_to_the_message_from() {
    let store = Store::new("signer-default", &["Alice <alice@localhost>"]);

    // NOTE: no `sender` property, so the From header decides.
    let mime = store
        .compile(&mml("<#part sign=pgpmime>signed<#/part>"))
        .unwrap();

    assert!(String::from_utf8_lossy(&mime).contains("BEGIN PGP SIGNATURE"));
}

#[test]
fn sign_then_verify() {
    let store = Store::new("sign", &["Alice <alice@localhost>"]);

    let mime = store
        .compile(&mml(
            "<#part sign=pgpmime sender=alice@localhost>signed<#/part>",
        ))
        .unwrap();
    let mime_str = String::from_utf8_lossy(&mime);

    assert!(mime_str.contains("multipart/signed"), "{mime_str}");
    assert!(mime_str.contains("BEGIN PGP SIGNATURE"), "{mime_str}");
    assert!(
        mime_str.contains("signed"),
        "a signature does not hide the body"
    );

    assert!(store.interpret(&mime).unwrap().contains("signed"));
}

#[test]
fn sign_and_encrypt_nest_in_order() {
    let store = Store::new("both", &["Alice <alice@localhost>"]);

    let mime = store
        .compile(&mml(
            "<#part sign=pgpmime encrypt=pgpmime sender=alice@localhost recipients=alice@localhost>both<#/part>",
        ))
        .unwrap();
    let mime_str = String::from_utf8_lossy(&mime);

    let encrypted = mime_str.find("multipart/encrypted");
    let signed = mime_str.find("multipart/signed");

    assert!(encrypted.is_some(), "{mime_str}");
    assert!(
        signed.is_none(),
        "the signature layer belongs inside the ciphertext, not beside it",
    );

    let mml = store.interpret(&mime).unwrap();

    assert!(mml.contains("both"), "{mml}");
    assert!(
        mml.contains("multipart/signed"),
        "decrypting reveals the signature layer: {mml}",
    );
}

#[test]
fn nested_encrypted_layers() {
    let store = Store::new("nested", &["Alice <alice@localhost>"]);

    let inner = store
        .compile(&mml(
            "<#part encrypt=pgpmime recipients=alice@localhost>deep secret<#/part>",
        ))
        .unwrap();

    let outer = store
        .compile(&format!(
            "From: alice@localhost\r\nTo: alice@localhost\r\nSubject: outer\r\n\r\n<#part encrypt=pgpmime recipients=alice@localhost type=message/rfc822>{}<#/part>",
            String::from_utf8_lossy(&inner),
        ))
        .unwrap();

    let mml = store.interpret(&outer).unwrap();

    assert!(mml.contains("deep secret"), "both layers decrypt: {mml}");
}

#[test]
fn encrypting_to_an_unknown_recipient_fails() {
    let store = Store::new("unknown", &["Alice <alice@localhost>"]);

    let err = store
        .compile(&mml(
            "<#part encrypt=pgpmime recipients=nobody@localhost>secret<#/part>",
        ))
        .unwrap_err();

    assert!(
        err.to_lowercase().contains("gpg") || err.to_lowercase().contains("key"),
        "an unresolvable recipient must abort with a key error: {err}",
    );
}

#[test]
fn encrypting_with_no_recipient_fails() {
    let store = Store::new("norcpt", &["Alice <alice@localhost>"]);

    let err = store
        .compile(&mml("<#part encrypt=pgpmime>secret<#/part>"))
        .unwrap_err();

    assert!(
        err.contains("recipient"),
        "an empty recipient list must abort, not encrypt to nobody: {err}",
    );
}

#[test]
fn decrypting_without_the_secret_key_fails() {
    let alice = Store::new("alice-only", &["Alice <alice@localhost>"]);
    let bob = Store::new("bob-only", &["Bob <bob@localhost>"]);

    let mime = alice
        .compile(&mml(
            "<#part encrypt=pgpmime recipients=alice@localhost>secret<#/part>",
        ))
        .unwrap();

    let err = bob.interpret(&mime).unwrap_err();

    assert!(
        err.to_lowercase().contains("gpg") || err.to_lowercase().contains("decrypt"),
        "Bob has no key for Alice's ciphertext: {err}",
    );
}

#[test]
fn replying_to_an_encrypted_message_quotes_the_plaintext() {
    let store = Store::new("reply", &["Alice <alice@localhost>"]);

    let mime = store
        .compile(&mml(
            "<#part encrypt=pgpmime recipients=alice@localhost>quoted secret<#/part>",
        ))
        .unwrap();

    // NOTE: the builder no longer interprets the source, so the
    // caller drives the coroutine and hands it decrypted text.
    let gpg = store.gpg();
    let mut coroutine = MmlInterpret::new(
        mime.clone().into(),
        MmlTemplateReplyBuilder::quote_options(),
    );
    let quote = drive(|answer| coroutine.resume(answer), &gpg).unwrap();

    let msg = MessageParser::new().parse(&mime).unwrap();
    let tpl = MmlTemplateReplyBuilder {
        from: "alice@localhost".into(),
        ..Default::default()
    }
    .build(&msg, &quote)
    .unwrap();

    assert!(
        tpl.content.contains("> quoted secret"),
        "the quote must be decrypted, not a ciphertext marker: {}",
        tpl.content,
    );
}

#[test]
fn a_tampered_signature_is_reported_as_a_verdict() {
    let store = Store::new("verdicts", &["Alice <alice@localhost>"]);

    let mime = store
        .compile(&mml(
            "<#part sign=pgpmime sender=alice@localhost>original<#/part>",
        ))
        .unwrap();
    let tampered = String::from_utf8_lossy(&mime).replace("original", "tampered");

    let gpg = store.gpg();
    let mut coroutine = MmlInterpret::new(
        tampered.as_bytes().to_vec().into(),
        MmlInterpretOptions::default(),
    );

    // NOTE: the interpretation still succeeds. The verdict is what
    // tells the caller not to trust it, which is what the CLI turns
    // into a distinct exit code.
    drive(|answer| coroutine.resume(answer), &gpg).unwrap();

    assert_eq!(coroutine.verdicts(), [MmlPgpVerdict::Invalid]);
}

#[test]
fn a_valid_signature_is_reported_as_a_verdict() {
    let store = Store::new("verdicts-ok", &["Alice <alice@localhost>"]);

    let mime = store
        .compile(&mml(
            "<#part sign=pgpmime sender=alice@localhost>original<#/part>",
        ))
        .unwrap();

    let gpg = store.gpg();
    let mut coroutine = MmlInterpret::new(mime.into(), MmlInterpretOptions::default());

    drive(|answer| coroutine.resume(answer), &gpg).unwrap();

    assert!(
        matches!(coroutine.verdicts(), [MmlPgpVerdict::Valid(_)]),
        "{:?}",
        coroutine.verdicts(),
    );
}
