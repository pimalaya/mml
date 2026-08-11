//! The layered MIME walk driving PGP without recursion.
//!
//! A message is one layer. Decrypting a `multipart/encrypted` part
//! yields plaintext that parses into another layer, which may itself
//! carry encrypted parts. [`MmlInterpret`] iterates a queue of
//! layers, so suspension only happens between them and the pure walk
//! inside a layer stays a plain function.
//!
//! Each layer owns its bytes through [`MmlMimeLayer`]: `Cow` so the
//! first layer borrows the caller's input at no cost while decrypted
//! layers own their plaintext, and ouroboros because the parsed
//! [`mail_parser::Message`] borrows those very bytes.

use std::borrow::Cow;

use mail_parser::{Message, MessageParser};
use ouroboros::self_referencing;

use crate::{
    coroutine::MmlYield,
    error::{MmlError, Result},
    interpreter::{
        body::{MmlMimePgpRequest, PLACEHOLDER_BEGIN, PLACEHOLDER_END, UNAVAILABLE_PART},
        message::MmlInterpretOptions,
    },
    pgp::{MmlPgpAnswer, MmlPgpVerdict},
};

/// One parsed MIME layer, owning the bytes it borrows from.
#[self_referencing]
pub struct MmlMimeLayer<'a> {
    bytes: Cow<'a, [u8]>,
    #[borrows(bytes)]
    #[covariant]
    parsed: Option<Message<'this>>,
}

impl<'a> MmlMimeLayer<'a> {
    /// Parses the given bytes into a layer that keeps them alive.
    pub fn parse(bytes: Cow<'a, [u8]>) -> Self {
        MmlMimeLayerBuilder {
            bytes,
            parsed_builder: |bytes| MessageParser::new().parse(bytes.as_ref()),
        }
        .build()
    }

    /// Borrows the parsed message, failing when the bytes were not
    /// valid MIME.
    pub fn message(&self) -> Result<&Message<'_>> {
        self.borrow_parsed()
            .as_ref()
            .ok_or(MmlError::ParseRawEmailError)
    }
}

/// A piece of rendered output, resolved or still awaiting a layer.
#[derive(Clone, Debug, Eq, PartialEq)]
enum MmlMimeFragment {
    /// Rendered MML, ready to concatenate.
    Text(String),
    /// A placeholder, keyed by the id of the request that fills it.
    ///
    /// NOTE: an id rather than a position, because resolving one
    /// placeholder splices several fragments in its place and shifts
    /// every position after it.
    Pending(usize),
}

/// A PGP request and the placeholder its answer resolves.
struct MmlMimePending {
    request: MmlMimePgpRequest,
    id: usize,
}

/// I/O-free MIME → MML interpretation coroutine.
///
/// Drive it by calling [`Self::resume`] with `None`, then once per
/// `WantsPgp*` yield with the matching answer, until it reports
/// [`MmlYield::Ok`] or [`MmlYield::Err`].
pub struct MmlInterpret<'a> {
    opts: MmlInterpretOptions,
    layers: Vec<MmlMimeLayer<'a>>,
    fragments: Vec<MmlMimeFragment>,
    pending: Vec<MmlMimePending>,
    served: usize,
    next_id: usize,
    verdicts: Vec<MmlPgpVerdict>,
}

impl<'a> MmlInterpret<'a> {
    /// Builds a coroutine interpreting the given MIME bytes.
    pub fn new(mime: Cow<'a, [u8]>, opts: MmlInterpretOptions) -> Self {
        Self {
            opts,
            layers: vec![MmlMimeLayer::parse(mime)],
            fragments: Vec::new(),
            pending: Vec::new(),
            served: 0,
            next_id: 0,
            verdicts: Vec::new(),
        }
    }

    /// Advances the coroutine, consuming the answer to the last yield.
    ///
    /// NOTE: the first call takes `None`, every later one takes the
    /// answer to the previous yield.
    pub fn resume(&mut self, answer: Option<MmlPgpAnswer>) -> MmlYield<String> {
        match self.step(answer) {
            Ok(yielded) => yielded,
            Err(err) => MmlYield::Err(err),
        }
    }

    /// Every signature verdict the run collected, in order.
    ///
    /// NOTE: read it once the coroutine reports [`MmlYield::Ok`]. An
    /// invalid signature is a verdict rather than a failure, so the
    /// interpretation still succeeds and the caller decides what to
    /// make of it.
    pub fn verdicts(&self) -> &[MmlPgpVerdict] {
        &self.verdicts
    }

    /// The fallible body of [`Self::resume`].
    fn step(&mut self, answer: Option<MmlPgpAnswer>) -> Result<MmlYield<String>> {
        match answer {
            None => {
                let layer = self.layers.first().ok_or(MmlError::ParseRawEmailError)?;
                let mut requests = Vec::new();
                let mml = self
                    .opts
                    .interpret_msg_pgp(layer.message()?, &mut requests)?;

                self.fragments = self.render(mml, requests);
            }
            Some(answer) => self.absorb(answer)?,
        }

        // NOTE: a layer is walked in full before the next one starts,
        // so the requests it discovers queue behind those already
        // found. Depth is bounded by the message, not by the stack.
        if let Some(pending) = self.pending.get(self.served) {
            return Ok(match &pending.request {
                MmlMimePgpRequest::Decrypt(ciphertext) => {
                    MmlYield::WantsPgpDecrypt(ciphertext.clone())
                }
                MmlMimePgpRequest::Verify { signed, signature } => MmlYield::WantsPgpVerify {
                    signed: signed.clone(),
                    signature: signature.clone(),
                },
            });
        }

        let mml = self
            .fragments
            .iter()
            .map(|fragment| match fragment {
                MmlMimeFragment::Text(text) => text.as_str(),
                MmlMimeFragment::Pending(_) => "",
            })
            .collect();

        Ok(MmlYield::Ok(mml))
    }

    /// Files an answer against its request, walking the layer it
    /// carries when it carries plaintext.
    fn absorb(&mut self, answer: MmlPgpAnswer) -> Result<()> {
        let id = self
            .pending
            .get(self.served)
            .ok_or(MmlError::ParseRawEmailError)?
            .id;

        self.served += 1;

        let rendered = match answer {
            // NOTE: the part stays visible rather than silently
            // vanishing, so a reader without a backend still sees that
            // something was there.
            MmlPgpAnswer::Unavailable => {
                vec![MmlMimeFragment::Text(String::from(UNAVAILABLE_PART))]
            }
            MmlPgpAnswer::Verdict(verdict) => {
                self.verdicts.push(verdict);
                vec![MmlMimeFragment::Text(String::new())]
            }
            MmlPgpAnswer::Bytes(plain) => {
                let layer = MmlMimeLayer::parse(Cow::Owned(plain));
                let mut requests = Vec::new();
                let mml = self
                    .opts
                    .interpret_msg_pgp(layer.message()?, &mut requests)?;

                self.layers.push(layer);
                self.render(mml, requests)
            }
        };

        self.resolve(id, rendered);

        Ok(())
    }

    /// Splits rendered MML on its placeholders, queueing one request
    /// per placeholder found.
    fn render(&mut self, mml: String, requests: Vec<MmlMimePgpRequest>) -> Vec<MmlMimeFragment> {
        let mut fragments = Vec::new();

        for (i, chunk) in mml.split(PLACEHOLDER_BEGIN).enumerate() {
            if i == 0 {
                fragments.push(MmlMimeFragment::Text(chunk.to_owned()));
                continue;
            }

            let Some((index, rest)) = chunk.split_once(PLACEHOLDER_END) else {
                fragments.push(MmlMimeFragment::Text(chunk.to_owned()));
                continue;
            };

            let id = self.next_id;
            self.next_id += 1;

            fragments.push(MmlMimeFragment::Pending(id));
            fragments.push(MmlMimeFragment::Text(rest.to_owned()));

            if let Some(request) = index.parse().ok().and_then(|i: usize| requests.get(i)) {
                self.pending.push(MmlMimePending {
                    request: request.clone(),
                    id,
                });
            }
        }

        fragments
    }

    /// Swaps a placeholder for the fragments that replace it.
    fn resolve(&mut self, id: usize, rendered: Vec<MmlMimeFragment>) {
        let placeholder = self
            .fragments
            .iter()
            .position(|fragment| matches!(fragment, MmlMimeFragment::Pending(this) if *this == id));

        match placeholder {
            Some(at) => {
                self.fragments.splice(at..=at, rendered);
            }
            None => self.fragments.extend(rendered),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::{
        coroutine::MmlYield,
        interpreter::{
            layer::{MmlInterpret, MmlMimeLayer},
            message::MmlInterpretOptions,
        },
        pgp::{MmlPgpAnswer, MmlPgpVerdict},
    };

    /// Wraps a ciphertext in the RFC 3156 section 4 layout.
    fn encrypted(ciphertext: &str) -> Vec<u8> {
        format!(
            "Content-Type: multipart/encrypted; boundary=\"b\"; \
             protocol=\"application/pgp-encrypted\"\r\n\r\n\
             --b\r\nContent-Type: application/pgp-encrypted\r\n\r\nVersion: 1\r\n\
             --b\r\nContent-Type: application/octet-stream\r\n\r\n{ciphertext}\r\n\
             --b--\r\n"
        )
        .into_bytes()
    }

    /// Wraps a body and its detached signature in the RFC 3156
    /// section 5 layout.
    fn signed(signature: &str, body: &str) -> Vec<u8> {
        format!(
            "Content-Type: multipart/signed; boundary=\"b\"; \
             protocol=\"application/pgp-signature\"\r\n\r\n\
             --b\r\nContent-Type: text/plain\r\n\r\n{body}\r\n\
             --b\r\nContent-Type: application/pgp-signature\r\n\r\n{signature}\r\n\
             --b--\r\n"
        )
        .into_bytes()
    }

    /// Builds a plain text message.
    fn plain(body: &str) -> Vec<u8> {
        format!("Content-Type: text/plain\r\n\r\n{body}\r\n").into_bytes()
    }

    /// Runs a coroutine to completion, answering each decrypt from
    /// the given ciphertext to plaintext table and recording what was
    /// asked.
    fn drive(mime: Vec<u8>, keyring: &[(&[u8], Vec<u8>)], asked: &mut Vec<Vec<u8>>) -> String {
        let mut coroutine = MmlInterpret::new(Cow::Owned(mime), MmlInterpretOptions::default());
        let mut answer = None;

        loop {
            let ciphertext = match coroutine.resume(answer.take()) {
                MmlYield::Ok(mml) => return mml,
                MmlYield::Err(err) => panic!("{err}"),
                MmlYield::WantsPgpDecrypt(ciphertext) => ciphertext,
                yielded => panic!("unexpected yield: {yielded:?}"),
            };

            asked.push(ciphertext.clone());

            answer = Some(
                keyring
                    .iter()
                    .find(|(from, _)| *from == ciphertext)
                    .map(|(_, plain)| MmlPgpAnswer::Bytes(plain.clone()))
                    .unwrap_or(MmlPgpAnswer::Unavailable),
            );
        }
    }

    #[test]
    fn plain_message_never_suspends() {
        let mut coroutine = MmlInterpret::new(
            Cow::Owned(plain("hello, world!")),
            MmlInterpretOptions::default(),
        );

        let yielded = coroutine.resume(None);

        assert!(matches!(yielded, MmlYield::Ok(mml) if mml.contains("hello, world!")));
    }

    #[test]
    fn one_encrypted_layer_resolves() {
        let keyring = [(b"CIPHER".as_slice(), plain("secret"))];
        let mut asked = Vec::new();

        let mml = drive(encrypted("CIPHER"), &keyring, &mut asked);

        assert!(mml.contains("secret"), "{mml:?}");
        assert_eq!(asked, [b"CIPHER".to_vec()]);
    }

    #[test]
    fn nested_encrypted_layers_resolve() {
        let keyring = [
            (b"OUTER".as_slice(), encrypted("INNER")),
            (b"INNER".as_slice(), plain("deep secret")),
        ];
        let mut asked = Vec::new();

        let mml = drive(encrypted("OUTER"), &keyring, &mut asked);

        assert!(mml.contains("deep secret"), "{mml:?}");
        assert_eq!(
            asked,
            [b"OUTER".to_vec(), b"INNER".to_vec(),],
            "one request per layer, outer first",
        );
    }

    #[test]
    fn unavailable_backend_keeps_the_part_visible() {
        let mut asked = Vec::new();

        let mml = drive(encrypted("CIPHER"), &[], &mut asked);

        assert!(mml.contains("application/pgp-encrypted"), "{mml:?}");
        assert_eq!(asked.len(), 1);
    }

    #[test]
    fn signed_message_asks_to_verify_and_still_renders() {
        let mime = signed("SIGNATURE", "signed body");
        let mut coroutine = MmlInterpret::new(Cow::Owned(mime), MmlInterpretOptions::default());

        let MmlYield::WantsPgpVerify { signed, signature } = coroutine.resume(None) else {
            panic!("expected a verify request");
        };

        assert_eq!(signature, b"SIGNATURE");
        assert!(
            String::from_utf8_lossy(&signed).contains("signed body"),
            "the signed bytes cover the part, headers included",
        );

        let answer = MmlPgpAnswer::Verdict(MmlPgpVerdict::Valid(String::from("key")));

        let MmlYield::Ok(mml) = coroutine.resume(Some(answer)) else {
            panic!("expected completion");
        };

        assert!(mml.contains("signed body"), "{mml:?}");
        assert!(!mml.contains('\u{1}'), "{mml:?}");
    }

    #[test]
    fn pgp_free_path_never_leaks_a_placeholder() {
        let mml = MmlInterpretOptions::default()
            .interpret(encrypted("CIPHER"))
            .unwrap();

        assert!(!mml.contains('\u{1}'), "{mml:?}");
        assert!(!mml.contains('\u{2}'), "{mml:?}");
        assert!(
            mml.contains("<#part type=application/pgp-encrypted>"),
            "the part stays visible, like an Unavailable answer: {mml:?}",
        );
    }

    #[test]
    fn no_placeholder_leaks_into_the_output() {
        let keyring = [(b"CIPHER".as_slice(), plain("secret"))];

        let mml = drive(encrypted("CIPHER"), &keyring, &mut Vec::new());

        assert!(!mml.contains('\u{1}'), "{mml:?}");
        assert!(!mml.contains('\u{2}'), "{mml:?}");
    }

    #[test]
    fn borrowed_layer_parses() {
        let mime = b"From: a@b.c\r\nSubject: hi\r\n\r\nhello\r\n";
        let layer = MmlMimeLayer::parse(Cow::Borrowed(mime.as_slice()));

        assert_eq!(layer.message().unwrap().subject(), Some("hi"));
    }

    #[test]
    fn owned_layer_outlives_its_source() {
        let layer = {
            let mime = Vec::from(*b"From: a@b.c\r\nSubject: owned\r\n\r\nhello\r\n");
            MmlMimeLayer::parse(Cow::Owned(mime))
        };

        assert_eq!(layer.message().unwrap().subject(), Some("owned"));
    }

    #[test]
    fn layers_survive_the_queue_growing() {
        let mut layers = Vec::new();

        for i in 0..64 {
            let mime = format!("From: a@b.c\r\nSubject: {i}\r\n\r\nhello\r\n");
            layers.push(MmlMimeLayer::parse(Cow::Owned(mime.into_bytes())));
        }

        for (i, layer) in layers.iter().enumerate() {
            assert_eq!(
                layer.message().unwrap().subject(),
                Some(i.to_string().as_str())
            );
        }
    }

    #[test]
    fn invalid_bytes_report_a_parse_error() {
        let layer = MmlMimeLayer::parse(Cow::Borrowed(b"".as_slice()));

        assert!(layer.message().is_err());
    }
}
