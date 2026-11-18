//! # Cryptographic Primitives & Envelope Encryption
//!
//! Provides authenticated encryption at rest (AES-256-GCM), key management,
//! tenant-isolated associated data binding, secret masking, and zeroization under Rule 05.
//!
//! ## Rule 05 Compliance & Security Invariants
//!
//! 1. **Authenticated Encryption at Rest:** Sensitive credentials (GitHub installation tokens,
//!    OAuth access tokens, refresh tokens) are encrypted with AES-256-GCM.
//! 2. **Cryptographic Tenant Isolation:** Every encryption binds the ciphertext to the
//!    `owner_session_id` and `github_installation_id` as Authenticated Associated Data (AAD).
//!    Ciphertext cannot be decrypted under another tenant's or installation's context.
//! 3. **Secret Redaction:** `EncryptionKey` and `EncryptedPayload` strictly redact sensitive
//!    key and ciphertext material in [`fmt::Debug`] and [`fmt::Display`] implementations.
//! 4. **Memory Hygiene / Zeroization:** `EncryptionKey` overwrites internal key material with
//!    zeroes upon being dropped.
//! 5. **High-Entropy Ephemeral Nonces:** Every encryption generates a fresh, 96-bit (12-byte)
//!    cryptographically secure random nonce from the OS random source (`ring::rand::SystemRandom`).

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use std::fmt;
use uuid::Uuid;

use crate::error::CoreError;

/// Decodes a hex string into raw bytes.
fn decode_hex(hex_str: &str) -> Result<Vec<u8>, CoreError> {
    let s = hex_str.trim();
    if s.len() % 2 != 0 {
        return Err(CoreError::Crypto("Hex string has odd length".to_string()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| CoreError::Crypto(format!("Invalid hex character at {i}: {e}")))
        })
        .collect()
}

/// 256-bit (32-byte) symmetric encryption key for AES-256-GCM.
///
/// Under Rule 05, key material is zeroized upon being dropped and masked in
/// [`fmt::Debug`] and [`fmt::Display`].
pub struct EncryptionKey {
    key: [u8; 32],
}

impl Drop for EncryptionKey {
    fn drop(&mut self) {
        // Overwrite key bytes in memory upon drop
        self.key.fill(0);
    }
}

impl Clone for EncryptionKey {
    fn clone(&self) -> Self {
        Self { key: self.key }
    }
}

impl fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***[REDACTED_ENCRYPTION_KEY]***")
    }
}

impl fmt::Display for EncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***[REDACTED_ENCRYPTION_KEY]***")
    }
}

impl EncryptionKey {
    /// Creates an `EncryptionKey` from a raw 32-byte array.
    #[must_use]
    pub const fn from_bytes(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Creates an `EncryptionKey` from a 32-byte slice.
    ///
    /// # Errors
    /// Returns [`CoreError::Crypto`] if slice length is not exactly 32 bytes.
    pub fn from_slice(slice: &[u8]) -> Result<Self, CoreError> {
        if slice.len() != 32 {
            return Err(CoreError::Crypto(format!(
                "Invalid key length: expected 32 bytes, got {}",
                slice.len()
            )));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(slice);
        Ok(Self { key })
    }

    /// Parses an `EncryptionKey` from a 64-character hex-encoded string.
    ///
    /// # Errors
    /// Returns [`CoreError::Crypto`] if hex is invalid or length is not 32 bytes.
    pub fn from_hex(hex_str: &str) -> Result<Self, CoreError> {
        let bytes = decode_hex(hex_str)?;
        Self::from_slice(&bytes)
    }

    /// Derives an `EncryptionKey` from a passphrase and salt using HKDF-SHA256.
    ///
    /// # Errors
    /// Returns [`CoreError::Crypto`] if HKDF derivation fails.
    pub fn from_passphrase(passphrase: &str, salt: &[u8]) -> Result<Self, CoreError> {
        let salt = ring::hkdf::Salt::new(ring::hkdf::HKDF_SHA256, salt);
        let prk = salt.extract(passphrase.as_bytes());
        let info: &[&[u8]] = &[b"web-repo-to-ai:token-encryption:v1"];
        let okm = prk
            .expand(info, &ring::aead::AES_256_GCM)
            .map_err(|e| CoreError::Crypto(format!("HKDF expand failed: {e:?}")))?;
        let mut key = [0u8; 32];
        okm.fill(&mut key)
            .map_err(|e| CoreError::Crypto(format!("HKDF fill failed: {e:?}")))?;
        Ok(Self { key })
    }

    /// Generates a new cryptographically random 256-bit encryption key.
    ///
    /// # Errors
    /// Returns [`CoreError::Crypto`] if OS random number generation fails.
    pub fn generate() -> Result<Self, CoreError> {
        let rng = SystemRandom::new();
        let mut key = [0u8; 32];
        rng.fill(&mut key)
            .map_err(|e| CoreError::Crypto(format!("Failed to generate secure random key: {e}")))?;
        Ok(Self { key })
    }

    /// Returns a reference to the raw 32-byte key material.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

/// Ciphertext payload containing encrypted bytes and unique 96-bit nonce.
#[derive(Clone, PartialEq, Eq)]
pub struct EncryptedPayload {
    /// Ciphertext bytes including 128-bit (16-byte) authentication tag.
    pub ciphertext: Vec<u8>,
    /// 96-bit (12-byte) initialization vector / nonce.
    pub nonce: [u8; 12],
}

impl fmt::Debug for EncryptedPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptedPayload")
            .field("ciphertext", &"***[REDACTED_CIPHERTEXT]***")
            .field("ciphertext_len", &self.ciphertext.len())
            .field("nonce_len", &self.nonce.len())
            .finish()
    }
}

impl fmt::Display for EncryptedPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EncryptedPayload(len={}, nonce_len={})",
            self.ciphertext.len(),
            self.nonce.len()
        )
    }
}

impl EncryptedPayload {
    /// Creates a new `EncryptedPayload`.
    #[must_use]
    pub const fn new(ciphertext: Vec<u8>, nonce: [u8; 12]) -> Self {
        Self { ciphertext, nonce }
    }

    /// Constructs an `EncryptedPayload` from a ciphertext vector and nonce slice.
    ///
    /// # Errors
    /// Returns [`CoreError::Crypto`] if nonce is not exactly 12 bytes.
    pub fn from_parts(ciphertext: Vec<u8>, nonce_slice: &[u8]) -> Result<Self, CoreError> {
        if nonce_slice.len() != 12 {
            return Err(CoreError::Crypto(format!(
                "Invalid nonce length: expected 12 bytes, got {}",
                nonce_slice.len()
            )));
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(nonce_slice);
        Ok(Self { ciphertext, nonce })
    }

    /// Returns the ciphertext bytes (including auth tag).
    #[must_use]
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    /// Returns the 12-byte nonce.
    #[must_use]
    pub const fn nonce(&self) -> &[u8; 12] {
        &self.nonce
    }
}

/// AES-256-GCM authenticated encryption engine.
pub struct Aes256GcmEngine;

impl Aes256GcmEngine {
    /// Nonce size in bytes (96 bits).
    pub const NONCE_LEN: usize = 12;
    /// Tag size in bytes (128 bits).
    pub const TAG_LEN: usize = 16;

    /// Encrypts `plaintext` with `key` and binds `associated_data` via AES-256-GCM.
    ///
    /// A fresh random 96-bit nonce is generated for every encryption.
    ///
    /// # Errors
    /// Returns [`CoreError::Crypto`] if RNG or encryption fails.
    pub fn encrypt(
        key: &EncryptionKey,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Result<EncryptedPayload, CoreError> {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; Self::NONCE_LEN];
        rng.fill(&mut nonce_bytes)
            .map_err(|e| CoreError::Crypto(format!("Failed to generate random nonce: {e}")))?;

        let unbound_key = UnboundKey::new(&AES_256_GCM, key.as_bytes())
            .map_err(|e| CoreError::Crypto(format!("Failed to initialize AES-256-GCM key: {e}")))?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|_| CoreError::Crypto("Failed to create AES-GCM nonce".to_string()))?;

        let aad = Aad::from(associated_data);

        let mut in_out = plaintext.to_vec();
        less_safe_key
            .seal_in_place_append_tag(nonce, aad, &mut in_out)
            .map_err(|e| CoreError::Crypto(format!("AES-256-GCM encryption failed: {e}")))?;

        Ok(EncryptedPayload {
            ciphertext: in_out,
            nonce: nonce_bytes,
        })
    }

    /// Decrypts and authenticates `payload` with `key` and `associated_data`.
    ///
    /// # Errors
    /// Returns [`CoreError::Crypto`] if ciphertext is tampered, key is incorrect,
    /// or `associated_data` does not match the encryption context.
    pub fn decrypt(
        key: &EncryptionKey,
        payload: &EncryptedPayload,
        associated_data: &[u8],
    ) -> Result<Vec<u8>, CoreError> {
        if payload.ciphertext.len() < Self::TAG_LEN {
            return Err(CoreError::Crypto(
                "Ciphertext too short: missing authentication tag".to_string(),
            ));
        }

        let unbound_key = UnboundKey::new(&AES_256_GCM, key.as_bytes())
            .map_err(|e| CoreError::Crypto(format!("Failed to initialize AES-256-GCM key: {e}")))?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        let nonce = Nonce::try_assume_unique_for_key(&payload.nonce)
            .map_err(|_| CoreError::Crypto("Failed to create AES-GCM nonce".to_string()))?;

        let aad = Aad::from(associated_data);

        let mut in_out = payload.ciphertext.clone();
        let decrypted_slice = less_safe_key
            .open_in_place(nonce, aad, &mut in_out)
            .map_err(|_| {
                CoreError::Crypto(
                    "Decryption failed: authentication tag or associated data mismatch (integrity check failed)"
                        .to_string(),
                )
            })?;

        Ok(decrypted_slice.to_vec())
    }
}

/// Helper building canonical Authenticated Associated Data (AAD) for a tenant/installation grant.
#[must_use]
pub fn build_token_aad(owner_session_id: &Uuid, github_installation_id: i64) -> Vec<u8> {
    format!("tenant:{owner_session_id}:installation:{github_installation_id}").into_bytes()
}

/// Encrypts an installation access token binding it to `owner_session_id` and `github_installation_id`.
///
/// Returns `(ciphertext_with_tag, nonce)`.
///
/// # Errors
/// Returns [`CoreError::Crypto`] on encryption failure.
pub fn encrypt_token(
    key: &EncryptionKey,
    token: &str,
    owner_session_id: &Uuid,
    github_installation_id: i64,
) -> Result<(Vec<u8>, Vec<u8>), CoreError> {
    let aad = build_token_aad(owner_session_id, github_installation_id);
    let payload = Aes256GcmEngine::encrypt(key, token.as_bytes(), &aad)?;
    Ok((payload.ciphertext, payload.nonce.to_vec()))
}

/// Decrypts an installation access token verifying its `owner_session_id` and `github_installation_id` binding.
///
/// # Errors
/// Returns [`CoreError::Crypto`] if ciphertext, nonce, key, or tenant binding are invalid.
pub fn decrypt_token(
    key: &EncryptionKey,
    ciphertext: &[u8],
    nonce: &[u8],
    owner_session_id: &Uuid,
    github_installation_id: i64,
) -> Result<String, CoreError> {
    let aad = build_token_aad(owner_session_id, github_installation_id);
    let payload = EncryptedPayload::from_parts(ciphertext.to_vec(), nonce)?;
    let decrypted_bytes = Aes256GcmEngine::decrypt(key, &payload, &aad)?;
    String::from_utf8(decrypted_bytes)
        .map_err(|e| CoreError::Crypto(format!("Decrypted token bytes are not valid UTF-8: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_key_generation_and_masking() {
        let key = EncryptionKey::generate().expect("key generation succeeds");
        assert_eq!(key.as_bytes().len(), 32);

        // Verify Debug and Display redaction
        let debug_str = format!("{key:?}");
        assert_eq!(debug_str, "***[REDACTED_ENCRYPTION_KEY]***");
        let display_str = format!("{key}");
        assert_eq!(display_str, "***[REDACTED_ENCRYPTION_KEY]***");

        // Test from_slice
        let slice = [42u8; 32];
        let key2 = EncryptionKey::from_slice(&slice).expect("valid slice");
        assert_eq!(key2.as_bytes(), &slice);

        // Invalid length
        assert!(EncryptionKey::from_slice(&[42u8; 31]).is_err());
        assert!(EncryptionKey::from_slice(&[42u8; 33]).is_err());
    }

    #[test]
    fn test_encryption_key_from_hex_and_passphrase() {
        let hex_valid = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key = EncryptionKey::from_hex(hex_valid).expect("hex parsing succeeds");
        assert_eq!(key.as_bytes()[0], 0x01);
        assert_eq!(key.as_bytes()[1], 0x23);

        // Odd length
        assert!(EncryptionKey::from_hex("abc").is_err());
        // Invalid hex char
        assert!(EncryptionKey::from_hex(
            "zz23456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        )
        .is_err());
        // Too short
        assert!(EncryptionKey::from_hex("0123456789abcdef").is_err());

        // Test HKDF passphrase derivation
        let salt = b"salt-for-unit-test";
        let derived_1 = EncryptionKey::from_passphrase("super-secret-passphrase", salt).unwrap();
        let derived_2 = EncryptionKey::from_passphrase("super-secret-passphrase", salt).unwrap();
        let derived_3 = EncryptionKey::from_passphrase("different-passphrase", salt).unwrap();

        assert_eq!(derived_1.as_bytes(), derived_2.as_bytes());
        assert_ne!(derived_1.as_bytes(), derived_3.as_bytes());
    }

    #[test]
    fn test_encrypted_payload_masking() {
        let payload = EncryptedPayload::new(vec![1, 2, 3, 4], [9u8; 12]);
        let debug_str = format!("{payload:?}");
        assert!(debug_str.contains("***[REDACTED_CIPHERTEXT]***"));
        assert!(!debug_str.contains("1, 2, 3, 4"));

        let display_str = format!("{payload}");
        assert!(display_str.contains("EncryptedPayload(len=4, nonce_len=12)"));
    }

    #[test]
    fn test_aes256gcm_encryption_decryption_roundtrip() {
        let key = EncryptionKey::generate().unwrap();
        let plaintext = b"ghs_16CharacterGitHubTokenSimulatedString1234567890";
        let aad = b"tenant:42:installation:999";

        let payload = Aes256GcmEngine::encrypt(&key, plaintext, aad).unwrap();
        assert_eq!(payload.nonce().len(), 12);
        // Ciphertext includes 16-byte tag
        assert_eq!(payload.ciphertext().len(), plaintext.len() + 16);

        let decrypted = Aes256GcmEngine::decrypt(&key, &payload, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes256gcm_tamper_detection() {
        let key = EncryptionKey::generate().unwrap();
        let plaintext = b"sensitive payload data";
        let aad = b"context-string";

        let mut payload = Aes256GcmEngine::encrypt(&key, plaintext, aad).unwrap();

        // 1. Tamper with ciphertext byte
        payload.ciphertext[0] ^= 0x01;
        assert!(Aes256GcmEngine::decrypt(&key, &payload, aad).is_err());
        payload.ciphertext[0] ^= 0x01; // Restore

        // 2. Tamper with nonce byte
        payload.nonce[0] ^= 0xFF;
        assert!(Aes256GcmEngine::decrypt(&key, &payload, aad).is_err());
        payload.nonce[0] ^= 0xFF; // Restore

        // 3. Tamper with associated data
        let wrong_aad = b"different-context";
        assert!(Aes256GcmEngine::decrypt(&key, &payload, wrong_aad).is_err());

        // 4. Wrong key
        let wrong_key = EncryptionKey::generate().unwrap();
        assert!(Aes256GcmEngine::decrypt(&wrong_key, &payload, aad).is_err());
    }

    #[test]
    fn test_token_encryption_tenant_isolation() {
        let key = EncryptionKey::generate().unwrap();
        let token = "ghs_MockTokenForTestingTenantIsolationUnderRule05";
        let session_a = Uuid::new_v4();
        let session_b = Uuid::new_v4();
        let install_1 = 12345678i64;
        let install_2 = 87654321i64;

        let (ciphertext, nonce) = encrypt_token(&key, token, &session_a, install_1).unwrap();

        // Valid decryption under same tenant and installation
        let decrypted = decrypt_token(&key, &ciphertext, &nonce, &session_a, install_1).unwrap();
        assert_eq!(decrypted, token);

        // Cross-tenant access attempt (different session ID) fails
        let cross_tenant_err = decrypt_token(&key, &ciphertext, &nonce, &session_b, install_1);
        assert!(cross_tenant_err.is_err());

        // Cross-installation access attempt (different installation ID) fails
        let cross_install_err = decrypt_token(&key, &ciphertext, &nonce, &session_a, install_2);
        assert!(cross_install_err.is_err());

        // Tampered nonce fails
        let mut bad_nonce = nonce.clone();
        bad_nonce[0] ^= 0xAA;
        assert!(decrypt_token(&key, &ciphertext, &bad_nonce, &session_a, install_1).is_err());
    }
}
