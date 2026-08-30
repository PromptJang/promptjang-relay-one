use crate::domain::DomainError;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

pub fn new_secret(prefix: &str) -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    format!("{prefix}{}", hex::encode(bytes))
}

pub fn hash_bytes(value: &[u8]) -> String {
    hex::encode(Sha256::digest(value))
}

pub fn hash_secret(value: &str) -> String {
    hash_bytes(value.as_bytes())
}

pub fn encrypt_secret(key: &[u8; 32], value: &str) -> Result<Vec<u8>, DomainError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| DomainError::internal("encryption key rejected"))?;
    let mut nonce_bytes = [0_u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), value.as_bytes())
        .map_err(|_| DomainError::internal("secret encryption failed"))?;
    let mut encoded = nonce_bytes.to_vec();
    encoded.extend(ciphertext);
    Ok(encoded)
}

pub fn decrypt_secret(key: &[u8; 32], encoded: &[u8]) -> Result<String, DomainError> {
    if encoded.len() < 13 {
        return Err(DomainError::internal("encrypted secret is invalid"));
    }
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| DomainError::internal("encryption key rejected"))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&encoded[..12]), &encoded[12..])
        .map_err(|_| DomainError::internal("secret decryption failed"))?;
    String::from_utf8(plaintext)
        .map_err(|_| DomainError::internal("decrypted secret is invalid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_secret_uses_prefix_and_64_hex_characters() {
        // Arrange
        let prefix = "pj_oss_";

        // Act
        let secret = new_secret(prefix);

        // Assert
        assert!(secret.starts_with(prefix));
        assert_eq!(secret.len(), prefix.len() + 64);
        assert!(
            secret[prefix.len()..]
                .chars()
                .all(|c| c.is_ascii_hexdigit())
        );
    }

    #[test]
    fn new_secret_is_unique_per_call() {
        // Arrange
        let prefix = "whsec_";

        // Act
        let first = new_secret(prefix);
        let second = new_secret(prefix);

        // Assert
        assert_ne!(first, second);
    }

    #[test]
    fn hash_bytes_matches_known_sha256_vector() {
        // Arrange
        let input = b"abc";

        // Act
        let digest = hash_bytes(input);

        // Assert
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hash_secret_is_deterministic_for_equal_values() {
        // Arrange
        let value = "pj_session_example";

        // Act
        let first = hash_secret(value);
        let second = hash_secret(value);

        // Assert
        assert_eq!(first, second);
    }

    #[test]
    fn encrypted_secret_round_trip_and_random_nonce() {
        let key = [7_u8; 32];
        let first = encrypt_secret(&key, "whsec_example").expect("encrypt");
        let second = encrypt_secret(&key, "whsec_example").expect("encrypt");
        assert_ne!(first, second);
        assert_eq!(decrypt_secret(&key, &first).as_deref(), Ok("whsec_example"));
    }
}
