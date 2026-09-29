//! The post-order worklist applying PGP during compilation.
//!
//! Signing or encrypting a part wraps children that are already
//! built, so the crypto order is strictly deepest-first. The parsed
//! tree is therefore flattened into a post-order list of slots, each
//! materialised in turn: by the time a slot needs crypto, every slot
//! it contains is final. No recursion, and no suspension inside one.
//!
//! Unlike [`crate::interpreter::layer`], nothing here is
//! self-referential: [`crate::compiler::body`] compiles to
//! `MimePart<'static>`, so a half-built tree survives a suspension on
//! its own.

use mail_builder::mime::{BodyPart, MimePart};

use crate::{
    compiler::{
        body::MmlBodyCompiler,
        parser::tokens::{Part, Props},
    },
    error::Result,
    grammar::{ALTERNATIVE, ENCRYPT, MIXED, PGP_MIME, RECIPIENTS, RELATED, SENDER, SIGN, TYPE},
};

/// The crypto a slot still owes, in application order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MmlCryptoStep {
    /// Sign as the given identity.
    Sign(String),
    /// Encrypt for the given recipients.
    Encrypt(Vec<String>),
}

/// One node of the flattened tree.
enum MmlCryptoSlot {
    /// A compiled leaf, ready as-is.
    Leaf(MimePart<'static>),
    /// A multipart, assembled once its children are materialised.
    Multi {
        ctype: &'static str,
        children: Vec<usize>,
    },
}

/// The parsed tree, flattened post-order with its pending crypto.
pub(crate) struct MmlCryptoPlan {
    slots: Vec<MmlCryptoSlot>,
    /// Crypto owed per slot, deepest slot first.
    steps: Vec<(usize, MmlCryptoStep)>,
    /// Slots of the root level, assembled last.
    roots: Vec<usize>,
    /// Who to sign as when a part carries no `sender` property.
    ///
    /// NOTE: the message `From`, since signing as someone other than
    /// the sender is the exception rather than the rule.
    signer: Option<String>,
}

impl MmlCryptoPlan {
    /// Flattens the parsed parts, compiling every leaf and recording
    /// the crypto each node owes.
    pub(crate) fn new(
        compiler: &MmlBodyCompiler,
        parts: Vec<Part<'_>>,
        signer: Option<String>,
    ) -> Result<Self> {
        let mut plan = Self {
            slots: Vec::new(),
            steps: Vec::new(),
            roots: Vec::new(),
            signer,
        };

        for part in parts {
            let slot = plan.push(compiler, part)?;
            plan.roots.push(slot);
        }

        Ok(plan)
    }

    /// The crypto steps to serve, deepest first.
    pub(crate) fn steps(&self) -> &[(usize, MmlCryptoStep)] {
        &self.steps
    }

    /// Pushes one part and its subtree, returning its slot.
    ///
    /// NOTE: children are pushed before their parent, which is what
    /// makes [`Self::steps`] post-order.
    fn push(&mut self, compiler: &MmlBodyCompiler, part: Part<'_>) -> Result<usize> {
        let slot = match part {
            Part::Multi(ref props, parts) => {
                let ctype = match props.get(TYPE) {
                    Some(&ALTERNATIVE) => "multipart/alternative",
                    Some(&RELATED) => "multipart/related",
                    Some(&MIXED) | None => "multipart/mixed",
                    Some(_) => "multipart/mixed",
                };

                let mut children = Vec::new();
                for part in parts {
                    children.push(self.push(compiler, part)?);
                }

                let slot = self.slots.len();
                self.slots.push(MmlCryptoSlot::Multi { ctype, children });
                self.record(slot, props);
                slot
            }
            Part::Single(ref props, _) => {
                let props = props.clone();
                let slot = self.slots.len();
                self.slots
                    .push(MmlCryptoSlot::Leaf(compiler.compile_part(part)?));
                self.record(slot, &props);
                slot
            }
            part => {
                let slot = self.slots.len();
                self.slots
                    .push(MmlCryptoSlot::Leaf(compiler.compile_part(part)?));
                slot
            }
        };

        Ok(slot)
    }

    /// Records the crypto the given props ask for.
    ///
    /// NOTE: sign before encrypt, per RFC 3156. Reversing it would
    /// leak who signed to anyone seeing the ciphertext.
    fn record(&mut self, slot: usize, props: &Props) {
        if let Some(&PGP_MIME) = props.get(SIGN) {
            let signer = props
                .get(SENDER)
                .map(ToString::to_string)
                .or_else(|| self.signer.clone())
                .unwrap_or_default();
            self.steps.push((slot, MmlCryptoStep::Sign(signer)));
        }

        if let Some(&PGP_MIME) = props.get(ENCRYPT) {
            let recipients = props
                .get(RECIPIENTS)
                .map(|recipients| {
                    recipients
                        .split(',')
                        .map(str::trim)
                        .filter(|recipient| !recipient.is_empty())
                        .map(ToOwned::to_owned)
                        .collect()
                })
                .unwrap_or_default();

            self.steps.push((slot, MmlCryptoStep::Encrypt(recipients)));
        }
    }

    /// Materialises the given slot, assembling multiparts from their
    /// already-materialised children.
    pub(crate) fn materialise(&mut self, slot: usize) -> Result<MimePart<'static>> {
        let taken = std::mem::replace(
            &mut self.slots[slot],
            MmlCryptoSlot::Leaf(MimePart::new("text/plain", "")),
        );

        match taken {
            MmlCryptoSlot::Leaf(part) => Ok(part),
            MmlCryptoSlot::Multi { ctype, children } => {
                let mut parts = Vec::new();
                for child in children {
                    parts.push(self.materialise(child)?);
                }

                Ok(MimePart::new(ctype, BodyPart::Multipart(parts)))
            }
        }
    }

    /// Replaces a slot with an already-built part.
    pub(crate) fn put(&mut self, slot: usize, part: MimePart<'static>) {
        self.slots[slot] = MmlCryptoSlot::Leaf(part);
    }

    /// Assembles the root, once every slot is final.
    pub(crate) fn finish(&mut self) -> Result<MimePart<'static>> {
        match self.roots.len() {
            0 => Ok(MimePart::new("text/plain", String::new())),
            1 => self.materialise(self.roots[0]),
            _ => {
                let roots = self.roots.clone();
                let mut parts = Vec::new();
                for slot in roots {
                    parts.push(self.materialise(slot)?);
                }

                Ok(MimePart::new("multipart/mixed", BodyPart::Multipart(parts)))
            }
        }
    }
}

/// Serialises a part for signing or encrypting.
pub(crate) fn serialise(part: MimePart<'static>) -> Vec<u8> {
    let mut bytes = Vec::new();
    part.write_part(&mut bytes);
    bytes
}

/// Wraps a signed part and its detached signature, RFC 3156 section 5.
pub(crate) fn signed_part(clear: MimePart<'static>, signature: Vec<u8>) -> MimePart<'static> {
    MimePart::new(
        "multipart/signed; protocol=\"application/pgp-signature\"; micalg=\"pgp-sha256\"",
        BodyPart::Multipart(vec![
            clear,
            MimePart::new("application/pgp-signature", crlf(signature)).transfer_encoding("7bit"),
        ]),
    )
}

/// Wraps an OpenPGP message, RFC 3156 section 4.
pub(crate) fn encrypted_part(ciphertext: Vec<u8>) -> MimePart<'static> {
    MimePart::new(
        "multipart/encrypted; protocol=\"application/pgp-encrypted\"",
        BodyPart::Multipart(vec![
            MimePart::new("application/pgp-encrypted", "Version: 1"),
            MimePart::new("application/octet-stream", crlf(ciphertext)).transfer_encoding("7bit"),
        ]),
    )
}

/// Normalises bare line feeds to CRLF, as MIME requires.
fn crlf(bytes: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut prev = 0u8;

    for byte in bytes {
        if byte == b'\n' && prev != b'\r' {
            out.push(b'\r');
        }

        out.push(byte);
        prev = byte;
    }

    out
}

#[cfg(test)]
mod tests {
    use crate::{
        compiler::message::{MmlCompile, MmlCompileOptions},
        coroutine::MmlYield,
        pgp::MmlPgpAnswer,
    };

    /// What the coroutine asked for, flattened for assertion.
    #[derive(Debug, Eq, PartialEq)]
    enum Asked {
        Sign(String),
        Encrypt(Vec<String>),
    }

    /// Runs a compilation to completion, answering every step with a
    /// marker derived from what was asked.
    fn drive(mml: &str, asked: &mut Vec<Asked>) -> Result<Vec<u8>, String> {
        let mut coroutine = MmlCompile::new(mml, MmlCompileOptions::default());
        let mut answer = None;

        loop {
            match coroutine.resume(answer.take()) {
                MmlYield::Ok(mime) => return Ok(mime),
                MmlYield::Err(err) => return Err(err.to_string()),
                MmlYield::WantsPgpSign { signer, .. } => {
                    asked.push(Asked::Sign(signer));
                    answer = Some(MmlPgpAnswer::Bytes(b"SIGNATURE".to_vec()));
                }
                MmlYield::WantsPgpEncrypt { recipients, .. } => {
                    asked.push(Asked::Encrypt(recipients));
                    answer = Some(MmlPgpAnswer::Bytes(b"CIPHERTEXT".to_vec()));
                }
                yielded => panic!("unexpected yield: {yielded:?}"),
            }
        }
    }

    #[test]
    fn no_directive_never_suspends() {
        let mut asked = Vec::new();
        let mime = drive("Content-Type: text/plain\r\n\r\nhello\r\n", &mut asked).unwrap();

        assert!(asked.is_empty());
        assert!(String::from_utf8_lossy(&mime).contains("hello"));
    }

    #[test]
    fn encrypt_wraps_in_the_rfc3156_envelope() {
        let mml = "To: a@b.c\r\n\r\n<#part encrypt=pgpmime recipients=a@b.c>secret<#/part>";
        let mut asked = Vec::new();

        let mime = drive(mml, &mut asked).unwrap();
        let mime = String::from_utf8_lossy(&mime);

        assert_eq!(asked, [Asked::Encrypt(vec![String::from("a@b.c")])]);
        assert!(mime.contains("multipart/encrypted"), "{mime}");
        assert!(mime.contains("application/pgp-encrypted"), "{mime}");
        assert!(mime.contains("CIPHERTEXT"), "{mime}");
        assert!(!mime.contains("secret"), "the cleartext must not survive");
    }

    #[test]
    fn sign_wraps_and_keeps_the_cleartext() {
        let mml = "To: a@b.c\r\n\r\n<#part sign=pgpmime sender=me@x.y>hello<#/part>";
        let mut asked = Vec::new();

        let mime = drive(mml, &mut asked).unwrap();
        let mime = String::from_utf8_lossy(&mime);

        assert_eq!(asked, [Asked::Sign(String::from("me@x.y"))]);
        assert!(mime.contains("multipart/signed"), "{mime}");
        assert!(mime.contains("application/pgp-signature"), "{mime}");
        assert!(mime.contains("hello"), "a signature does not hide the body");
    }

    #[test]
    fn sign_runs_before_encrypt() {
        let mml = "To: a@b.c\r\n\r\n<#part sign=pgpmime encrypt=pgpmime sender=me@x.y recipients=a@b.c>secret<#/part>";
        let mut asked = Vec::new();

        drive(mml, &mut asked).unwrap();

        assert_eq!(
            asked,
            [
                Asked::Sign(String::from("me@x.y")),
                Asked::Encrypt(vec![String::from("a@b.c")]),
            ],
            "RFC 3156 signs then encrypts, never the reverse",
        );
    }

    #[test]
    fn nested_crypto_runs_deepest_first() {
        let mml = concat!(
            "To: a@b.c\r\n\r\n",
            "<#multipart sign=pgpmime sender=outer@x.y>",
            "<#part encrypt=pgpmime recipients=inner@x.y>secret<#/part>",
            "<#/multipart>",
        );
        let mut asked = Vec::new();

        drive(mml, &mut asked).unwrap();

        assert_eq!(
            asked,
            [
                Asked::Encrypt(vec![String::from("inner@x.y")]),
                Asked::Sign(String::from("outer@x.y")),
            ],
            "the inner part is final before the outer one is signed",
        );
    }

    #[test]
    fn the_pgp_free_path_refuses_too() {
        let mml = "To: a@b.c\r\n\r\n<#part encrypt=pgpmime recipients=a@b.c>secret<#/part>";

        let err = MmlCompileOptions::default().compile(mml).unwrap_err();

        assert!(
            err.to_string().contains("no PGP backend"),
            "compiling without a backend must fail, not ship cleartext: {err}",
        );
    }

    #[test]
    fn unavailable_backend_aborts_rather_than_leaking() {
        let mml = "To: a@b.c\r\n\r\n<#part encrypt=pgpmime recipients=a@b.c>secret<#/part>";
        let mut coroutine = MmlCompile::new(mml, MmlCompileOptions::default());

        assert!(matches!(
            coroutine.resume(None),
            MmlYield::WantsPgpEncrypt { .. }
        ));

        let yielded = coroutine.resume(Some(MmlPgpAnswer::Unavailable));

        assert!(
            matches!(yielded, MmlYield::Err(_)),
            "a message marked encrypt must never compile in the clear",
        );
    }
}
