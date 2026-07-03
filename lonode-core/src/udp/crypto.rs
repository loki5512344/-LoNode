//! XSalsa20-Poly1305 encryption for Discord voice UDP packets.
//!
//! Discord derives the 24-byte XSalsa20 nonce from the 12-byte RTP header
//! (zero-padded). This module wraps the [`xsalsa20poly1305`] crate so callers
//! don't have to deal with nonce construction manually.

use crate::Result;
use xsalsa20poly1305::aead::{Aead, KeyInit};
use xsalsa20poly1305::{Key, Nonce, XSalsa20Poly1305};

/// Length of the Poly1305 authentication tag appended to each ciphertext.
pub const TAG_LEN: usize = 16;

/// Convenience wrapper around [`XSalsa20Poly1305`] keyed with the
/// `secret_key` Discord sends in `op 4` Session Description.
#[derive(Clone)]
pub struct VoiceCrypto {
    cipher: XSalsa20Poly1305,
}

impl std::fmt::Debug for VoiceCrypto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceCrypto").finish_non_exhaustive()
    }
}

impl VoiceCrypto {
    /// Build a crypto wrapper from the 32-byte `secret_key`.
    #[must_use]
    pub fn new(secret_key: &[u8; 32]) -> Self {
        Self {
            cipher: XSalsa20Poly1305::new(Key::from_slice(secret_key)),
        }
    }

    /// Encrypt `plaintext` using `rtp_header` (12 bytes) as the nonce source.
    /// The returned `Vec` has length `plaintext.len() + TAG_LEN`.
    ///
    /// # Errors
    /// Returns an error if the underlying AEAD encryption fails (extremely
    /// unlikely for XSalsa20-Poly1305).
    pub fn encrypt(&self, rtp_header: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = build_nonce(rtp_header);
        Ok(self.cipher.encrypt(Nonce::from_slice(&nonce), plaintext)?)
    }

    /// Decrypt a ciphertext produced by [`encrypt`](Self::encrypt).
    ///
    /// # Errors
    /// Returns an error if authentication fails or the header is malformed.
    pub fn decrypt(&self, rtp_header: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        let nonce = build_nonce(rtp_header);
        Ok(self.cipher.decrypt(Nonce::from_slice(&nonce), ciphertext)?)
    }
}

/// Build a 24-byte nonce by zero-padding the 12-byte RTP header.
fn build_nonce(rtp_header: &[u8]) -> [u8; 24] {
    let mut nonce = [0u8; 24];
    let n = rtp_header.len().min(24);
    nonce[..n].copy_from_slice(&rtp_header[..n]);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let crypto = VoiceCrypto::new(&key);
        let header = [0x80, 0x78, 0x00, 0x01, 0, 0, 0, 0x60, 0, 0, 0, 0xAB];
        let plain = [1, 2, 3, 4, 5];
        let cipher = crypto.encrypt(&header, &plain).unwrap();
        assert_eq!(cipher.len(), plain.len() + TAG_LEN);
        let back = crypto.decrypt(&header, &cipher).unwrap();
        assert_eq!(back, plain);
    }

    #[test]
    fn decrypt_rejects_tampered_ciphertext() {
        let key = [7u8; 32];
        let crypto = VoiceCrypto::new(&key);
        let header = [0u8; 12];
        let mut cipher = crypto.encrypt(&header, &[9, 9, 9]).unwrap();
        cipher[0] ^= 0xFF;
        assert!(crypto.decrypt(&header, &cipher).is_err());
    }
}
