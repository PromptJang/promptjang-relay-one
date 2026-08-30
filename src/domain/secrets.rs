use crate::domain::DomainError;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

pub fn new_secret(prefix: &str) -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    format!("{prefix}{}", hex::encode(bytes))
}

pub fn new_webhook_secret() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    format!("whsec_{}", STANDARD.encode(bytes))
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

pub fn hash_password(password: &str) -> Result<String, DomainError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| DomainError::internal(format!("password hashing failed: {error}")))
}

pub fn verify_password(password: &str, encoded: &str) -> bool {
    PasswordHash::new(encoded).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    })
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
    fn webhook_secret_is_standard_base64_with_256_bits() {
        // Arrange and act
        let secret = new_webhook_secret();
        let encoded = secret.strip_prefix("whsec_").expect("prefix");

        // Assert
        assert_eq!(STANDARD.decode(encoded).map(|bytes| bytes.len()), Ok(32));
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
    fn password_round_trip_verifies() {
        // Arrange
        let password = "at-least-twelve-characters";
        let encoded = hash_password(password).expect("hashing succeeds");

        // Act
        let verified = verify_password(password, &encoded);

        // Assert
        assert!(verified);
    }

    #[test]
    fn wrong_password_or_corrupt_hash_fails_closed() {
        // Arrange
        let encoded = hash_password("correct-horse-battery").expect("hashing succeeds");

        // Act
        let wrong_password = verify_password("incorrect-stapler", &encoded);
        let corrupt_hash = verify_password("correct-horse-battery", "not-a-hash");

        // Assert
        assert!(!wrong_password);
        assert!(!corrupt_hash);
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
