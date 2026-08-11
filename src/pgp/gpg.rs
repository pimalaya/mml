//! GPG backend serving the `WantsPgp*` yields.
//!
//! A thin adapter over gpgme, which owns key discovery, the agent and
//! the passphrase prompt. It is the only place in the crate that
//! performs cryptography, and the coroutines never reach it: the
//! caller does, with a yield in hand.

use std::path::PathBuf;

use gpgme::{Context, Data, EncryptFlags, Protocol, SignMode};

use crate::{
    coroutine::MmlYield,
    error::{MmlError, Result},
    pgp::{MmlPgpAnswer, MmlPgpVerdict},
};

/// The GPG backend.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MmlPgpGpg {
    /// The GPG home directory, defaulting to GPG's own.
    ///
    /// NOTE: an explicit directory is how a caller isolates a keyring
    /// without touching `GNUPGHOME`, which is process-wide.
    pub home_dir: Option<PathBuf>,
}

impl MmlPgpGpg {
    /// Serves one `WantsPgp*` yield through GPG.
    ///
    /// Returns `None` for a terminal yield, which the caller handles
    /// itself. Any gpgme failure is an error, never a silent
    /// fallback: a part marked `encrypt=pgpmime` must not compile in
    /// the clear.
    pub fn serve<T>(&self, yielded: &MmlYield<T>) -> Option<Result<MmlPgpAnswer>> {
        match yielded {
            MmlYield::Ok(_) | MmlYield::Err(_) => None,
            MmlYield::WantsPgpEncrypt { plain, recipients } => {
                Some(self.encrypt(plain, recipients))
            }
            MmlYield::WantsPgpDecrypt(ciphertext) => Some(self.decrypt(ciphertext)),
            MmlYield::WantsPgpSign { plain, signer } => Some(self.sign(plain, signer)),
            MmlYield::WantsPgpVerify { signed, signature } => Some(self.verify(signed, signature)),
        }
    }

    /// Opens a gpgme context on the OpenPGP protocol, in armor mode.
    fn context(&self) -> Result<Context> {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp).map_err(MmlError::PgpGpgError)?;
        ctx.set_armor(true);

        if let Some(home_dir) = &self.home_dir {
            let home_dir = home_dir.to_string_lossy();
            ctx.set_engine_home_dir(home_dir.as_bytes())
                .map_err(MmlError::PgpGpgError)?;
        }

        Ok(ctx)
    }

    /// Encrypts for the given recipients, who must all resolve to a key.
    ///
    /// NOTE: `locate_key` follows GPG's auto-key-locate, so a recipient
    /// missing from the keyring can still be found through WKD or a
    /// keyserver, exactly as the `gpg` command would.
    ///
    /// NOTE: `ALWAYS_TRUST` because a mail composer cannot demand that
    /// every recipient key be signed first. GPG's web of trust is the
    /// user's to curate, not a precondition for sending.
    fn encrypt(&self, plain: &[u8], recipients: &[String]) -> Result<MmlPgpAnswer> {
        if recipients.is_empty() {
            return Err(MmlError::PgpMissingRecipientsError);
        }

        let mut ctx = self.context()?;

        let keys = recipients
            .iter()
            .map(|recipient| ctx.locate_key(recipient).map_err(MmlError::PgpGpgError))
            .collect::<Result<Vec<_>>>()?;

        let mut ciphertext = Vec::new();
        ctx.encrypt_with_flags(&keys, plain, &mut ciphertext, EncryptFlags::ALWAYS_TRUST)
            .map_err(MmlError::PgpGpgError)?;

        Ok(MmlPgpAnswer::Bytes(ciphertext))
    }

    /// Decrypts an OpenPGP message with whatever secret key matches.
    fn decrypt(&self, ciphertext: &[u8]) -> Result<MmlPgpAnswer> {
        let mut ctx = self.context()?;
        let mut plain = Vec::new();

        ctx.decrypt(ciphertext, &mut plain)
            .map_err(MmlError::PgpGpgError)?;

        Ok(MmlPgpAnswer::Bytes(plain))
    }

    /// Produces a detached signature as the given identity.
    fn sign(&self, plain: &[u8], signer: &str) -> Result<MmlPgpAnswer> {
        let mut ctx = self.context()?;

        if !signer.is_empty() {
            let key = ctx.get_secret_key(signer).map_err(MmlError::PgpGpgError)?;
            ctx.add_signer(&key).map_err(MmlError::PgpGpgError)?;
        }

        let mut signature = Vec::new();
        ctx.sign(SignMode::Detached, plain, &mut signature)
            .map_err(MmlError::PgpGpgError)?;

        Ok(MmlPgpAnswer::Bytes(signature))
    }

    /// Checks a detached signature over the bytes it covers.
    ///
    /// NOTE: an unreadable or unmatched signature is a verdict, not an
    /// error. Only gpgme itself failing is.
    fn verify(&self, signed: &[u8], signature: &[u8]) -> Result<MmlPgpAnswer> {
        let mut ctx = self.context()?;

        let signature = Data::from_bytes(signature).map_err(MmlError::PgpGpgError)?;
        let signed = Data::from_bytes(signed).map_err(MmlError::PgpGpgError)?;

        let result = ctx
            .verify_detached(signature, signed)
            .map_err(MmlError::PgpGpgError)?;

        let verdict = match result.signatures().next() {
            Some(signature) if signature.status().is_ok() => {
                MmlPgpVerdict::Valid(signature.fingerprint().unwrap_or("unknown").to_owned())
            }
            _ => MmlPgpVerdict::Invalid,
        };

        Ok(MmlPgpAnswer::Verdict(verdict))
    }
}
