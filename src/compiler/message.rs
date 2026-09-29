//! MML → MIME message compilation: parses MML headers + body, hands
//! the body off to [`crate::compiler::body`], stitches the result
//! into MIME bytes.

use mail_builder::{MessageBuilder, headers::text::Text, mime::MimePart};
use mail_parser::MessageParser;

use crate::{
    compiler::{
        body::MmlBodyCompiler,
        crypto::{MmlCryptoPlan, MmlCryptoStep, encrypted_part, serialise, signed_part},
    },
    coroutine::MmlYield,
    error::{MmlError, Result},
    pgp::MmlPgpAnswer,
};

/// MML → MIME compilation options.
#[derive(Clone, Debug, Default)]
pub struct MmlCompileOptions {
    /// The body-level compiler, carrying the part options.
    pub body: MmlBodyCompiler,
}

impl MmlCompileOptions {
    /// Compiles an MML message into MIME bytes.
    ///
    /// Headers are copied over verbatim from the MML source, with a
    /// `MIME-Version` added.
    ///
    /// NOTE: drives [`MmlCompile`] with no PGP backend, so MML asking
    /// to sign or encrypt fails rather than compiling in the clear.
    /// Drive the coroutine yourself to serve those.
    pub fn compile(&self, mml: &str) -> Result<Vec<u8>> {
        let mut coroutine = MmlCompile::new(mml, self.clone());
        let mut answer = None;

        loop {
            match coroutine.resume(answer.take()) {
                MmlYield::Ok(mime) => return Ok(mime),
                MmlYield::Err(err) => return Err(err),
                _ => answer = Some(MmlPgpAnswer::Unavailable),
            }
        }
    }

    /// Flattens the MML body into the post-order crypto plan.
    fn plan(&self, mml: &str) -> Result<MmlCryptoPlan> {
        let mml_msg = MessageParser::new()
            .parse(mml.as_bytes())
            .ok_or(MmlError::ParseMessageError)?;

        let mml_body = mml_msg
            .text_bodies()
            .next()
            .ok_or(MmlError::ParseMmlEmptyBodyError)?
            .text_contents()
            .ok_or(MmlError::ParseMmlEmptyBodyContentError)?;

        let signer = mml_msg
            .from()
            .and_then(|from| from.first())
            .and_then(|from| from.address())
            .map(ToString::to_string);

        MmlCryptoPlan::new(&self.body, self.body.parse(mml_body)?, signer)
    }

    /// Stitches the compiled body under the MML source headers.
    fn assemble(&self, mml: &str, body: MimePart<'static>) -> Result<Vec<u8>> {
        let mml_msg = MessageParser::new()
            .parse(mml.as_bytes())
            .ok_or(MmlError::ParseMessageError)?;

        let mut mime_msg = MessageBuilder::new().body(body);

        mime_msg = mime_msg.header("MIME-Version", Text::new("1.0"));

        for header in mml_msg.headers() {
            let key = header.name.as_str();
            let val = crate::header::to_builder_val(header);
            mime_msg = mime_msg.header(key, val);
        }

        mime_msg
            .write_to_vec()
            .map_err(MmlError::CompileMmlMessageToVecError)
    }
}

/// I/O-free MML → MIME compilation coroutine.
///
/// Drive it by calling [`Self::resume`] with `None`, then once per
/// `WantsPgp*` yield with the matching answer, until it reports
/// [`MmlYield::Ok`] or [`MmlYield::Err`].
///
/// NOTE: suspends once per `sign` or `encrypt` property found,
/// deepest part first, sign before encrypt. See
/// [`crate::compiler::crypto`].
pub struct MmlCompile {
    mml: String,
    opts: MmlCompileOptions,
    plan: Option<MmlCryptoPlan>,
    served: usize,
}

impl MmlCompile {
    /// Builds a coroutine compiling the given MML message.
    pub fn new(mml: impl ToString, opts: MmlCompileOptions) -> Self {
        Self {
            mml: mml.to_string(),
            opts,
            plan: None,
            served: 0,
        }
    }

    /// Advances the coroutine, consuming the answer to the last yield.
    ///
    /// NOTE: the first call takes `None`, every later one takes the
    /// answer to the previous yield.
    pub fn resume(&mut self, answer: Option<MmlPgpAnswer>) -> MmlYield<Vec<u8>> {
        match self.step(answer) {
            Ok(yielded) => yielded,
            Err(err) => MmlYield::Err(err),
        }
    }

    /// The fallible body of [`Self::resume`].
    fn step(&mut self, answer: Option<MmlPgpAnswer>) -> Result<MmlYield<Vec<u8>>> {
        match answer {
            None => self.plan = Some(self.opts.plan(&self.mml)?),
            Some(answer) => self.absorb(answer)?,
        }

        let plan = self.plan.as_mut().ok_or(MmlError::ParseMessageError)?;

        if let Some((slot, step)) = plan.steps().get(self.served).cloned() {
            // NOTE: materialising takes the slot, so put the part back
            // for the wrapping that follows. Signing keeps it as the
            // cleartext half of the `multipart/signed`.
            let part = plan.materialise(slot)?;
            let plain = serialise(part.clone());
            plan.put(slot, part);

            return Ok(match step {
                MmlCryptoStep::Sign(signer) => MmlYield::WantsPgpSign { plain, signer },
                MmlCryptoStep::Encrypt(recipients) => {
                    MmlYield::WantsPgpEncrypt { plain, recipients }
                }
            });
        }

        let body = plan.finish()?;

        Ok(MmlYield::Ok(self.opts.assemble(&self.mml, body)?))
    }

    /// Wraps the part the served step covered with what came back.
    ///
    /// NOTE: an unavailable or unusable answer aborts. A message
    /// marked `encrypt=pgpmime` must never go out in the clear.
    fn absorb(&mut self, answer: MmlPgpAnswer) -> Result<()> {
        let plan = self.plan.as_mut().ok_or(MmlError::ParseMessageError)?;

        let (slot, step) = plan
            .steps()
            .get(self.served)
            .cloned()
            .ok_or(MmlError::ParseMessageError)?;

        self.served += 1;

        let MmlPgpAnswer::Bytes(bytes) = answer else {
            return Err(MmlError::PgpUnavailableError);
        };

        let part = match step {
            MmlCryptoStep::Sign(_) => signed_part(plan.materialise(slot)?, bytes),
            MmlCryptoStep::Encrypt(_) => encrypted_part(bytes),
        };

        plan.put(slot, part);

        Ok(())
    }
}

#[cfg(all(test, feature = "interpreter"))]
mod tests {
    use concat_with::concat_line;

    use crate::{
        compiler::message::MmlCompileOptions,
        interpreter::message::{MmlFilterHeaders, MmlInterpretOptions},
    };

    #[test]
    fn non_ascii_headers() {
        let mml = concat_line!(
            "Message-ID: <id@localhost>",
            "Date: Thu, 1 Jan 1970 00:00:00 +0000",
            "From: Frȯm <from@localhost>",
            "To: Tó <to@localhost>",
            "Subject: Subjêct",
            "",
            "Hello, world!",
            "",
        );

        let mime = MmlCompileOptions::default().compile(mml).unwrap();

        let mml_msg = MmlInterpretOptions {
            show_headers: MmlFilterHeaders::include(["From", "To", "Subject"]),
            ..Default::default()
        }
        .interpret(&mime)
        .unwrap();

        let expected_mml_msg = concat_line!(
            "From: Frȯm <from@localhost>",
            "To: Tó <to@localhost>",
            "Subject: Subjêct",
            "",
            "Hello, world!",
            "",
        );

        assert_eq!(mml_msg, expected_mml_msg);
    }

    #[test]
    fn message_id_with_angles() {
        let mml = concat_line!(
            "From: Hugo Osvaldo Barrera <hugo@localhost>",
            "To: Hugo Osvaldo Barrera <hugo@localhost>",
            "Cc:",
            "Subject: Blah",
            "Message-ID: <bfb64e12-b7d4-474c-a658-8a221365f8ca@localhost>",
            "",
            "Test message",
            "",
        );

        let mime = MmlCompileOptions::default().compile(mml).unwrap();

        let mml_msg = MmlInterpretOptions {
            show_headers: MmlFilterHeaders::include(["Message-ID"]),
            ..Default::default()
        }
        .interpret(&mime)
        .unwrap();

        let expected_mml_msg = concat_line!(
            "Message-ID: <bfb64e12-b7d4-474c-a658-8a221365f8ca@localhost>",
            "",
            "Test message",
            "",
        );

        assert_eq!(mml_msg, expected_mml_msg);
    }

    #[test]
    fn message_id_without_angles() {
        let mml = concat_line!(
            "From: Hugo Osvaldo Barrera <hugo@localhost>",
            "To: Hugo Osvaldo Barrera <hugo@localhost>",
            "Cc:",
            "Subject: Blah",
            "Message-ID: bfb64e12-b7d4-474c-a658-8a221365f8ca@localhost",
            "",
            "Test message",
            "",
        );

        let mime = MmlCompileOptions::default().compile(mml).unwrap();

        let mml_msg = MmlInterpretOptions {
            show_headers: MmlFilterHeaders::include(["Message-ID"]),
            ..Default::default()
        }
        .interpret(&mime)
        .unwrap();

        let expected_mml_msg = concat_line!(
            "Message-ID: <bfb64e12-b7d4-474c-a658-8a221365f8ca@localhost>",
            "",
            "Test message",
            "",
        );

        assert_eq!(mml_msg, expected_mml_msg);
    }

    #[test]
    fn mml_markup_unescaped() {
        let mml = concat_line!(
            "Message-ID: <id@localhost>",
            "Date: Thu, 1 Jan 1970 00:00:00 +0000",
            "From: from@localhost",
            "To: to@localhost",
            "Subject: subject",
            "",
            "<#!part>This should be unescaped<#!/part>",
            "",
        );

        let mime = MmlCompileOptions::default().compile(mml).unwrap();
        let mime_msg_str = String::from_utf8(mime.clone()).unwrap();

        let mml_msg = MmlInterpretOptions {
            show_headers: MmlFilterHeaders::include(["From", "To", "Subject"]),
            ..Default::default()
        }
        .interpret(&mime)
        .unwrap();

        let expected_mml_msg = concat_line!(
            "From: from@localhost",
            "To: to@localhost",
            "Subject: subject",
            "",
            "<#!part>This should be unescaped<#!/part>",
            "",
        );

        assert!(!mime_msg_str.contains("<#!part>"));
        assert!(mime_msg_str.contains("<#part>"));

        assert!(!mime_msg_str.contains("<#!/part>"));
        assert!(mime_msg_str.contains("<#/part>"));

        assert_eq!(mml_msg, expected_mml_msg);
    }
}
