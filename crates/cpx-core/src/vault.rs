use std::fs;
use std::path::Path;

use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::symbolize::SymbolEntry;

const VAULT_VERSION: u32 = 1;
const VAULT_KEY_LEN: usize = 32;
const VAULT_NONCE_LEN: usize = 12;
const VAULT_SALT_LEN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultHandle {
    pub case_id: String,
    pub entries: Vec<SymbolEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRequest<'a> {
    pub case_id: &'a str,
    pub entries: &'a [SymbolEntry],
    pub passphrase: &'a str,
}

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("vault passphrase was empty")]
    EmptyPassphrase,
    #[error("failed to read or write the vault")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize or parse the vault")]
    Serde(#[from] serde_json::Error),
    #[error("failed to gather local randomness")]
    Randomness,
    #[error("vault contents were invalid: {0}")]
    InvalidVault(&'static str),
    #[error("vault decryption failed")]
    DecryptionFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct VaultEnvelope {
    version: u32,
    case_id: String,
    salt_hex: String,
    nonce_hex: String,
    ciphertext_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct VaultPayload {
    case_id: String,
    entries: Vec<SymbolEntry>,
}

pub fn store(
    path: impl AsRef<Path>,
    request: &StoreRequest<'_>,
) -> Result<VaultHandle, VaultError> {
    if request.passphrase.is_empty() {
        return Err(VaultError::EmptyPassphrase);
    }

    let mut salt = [0_u8; VAULT_SALT_LEN];
    let mut nonce = [0_u8; VAULT_NONCE_LEN];
    getrandom(&mut salt).map_err(|_| VaultError::Randomness)?;
    getrandom(&mut nonce).map_err(|_| VaultError::Randomness)?;

    let payload = VaultPayload {
        case_id: request.case_id.to_owned(),
        entries: request.entries.to_vec(),
    };
    let plaintext = serde_json::to_vec(&payload)?;
    let key = derive_key(request.passphrase, &salt)?;
    let cipher = Aes256GcmSiv::new_from_slice(&key).map_err(|_| VaultError::DecryptionFailed)?;
    let aad = aad_for(request.case_id);
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &plaintext,
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| VaultError::DecryptionFailed)?;

    let envelope = VaultEnvelope {
        version: VAULT_VERSION,
        case_id: request.case_id.to_owned(),
        salt_hex: encode_hex(&salt),
        nonce_hex: encode_hex(&nonce),
        ciphertext_hex: encode_hex(&ciphertext),
    };

    fs::write(path, serde_json::to_vec_pretty(&envelope)?)?;

    Ok(VaultHandle {
        case_id: request.case_id.to_owned(),
        entries: request.entries.to_vec(),
    })
}

pub fn open(path: impl AsRef<Path>, passphrase: &str) -> Result<VaultHandle, VaultError> {
    if passphrase.is_empty() {
        return Err(VaultError::EmptyPassphrase);
    }

    let envelope: VaultEnvelope = serde_json::from_slice(&fs::read(path)?)?;

    if envelope.version != VAULT_VERSION {
        return Err(VaultError::InvalidVault("unsupported vault version"));
    }

    let salt = decode_hex_exact(&envelope.salt_hex, VAULT_SALT_LEN)?;
    let nonce = decode_hex_exact(&envelope.nonce_hex, VAULT_NONCE_LEN)?;
    let ciphertext = decode_hex(&envelope.ciphertext_hex)?;
    let key = derive_key(passphrase, &salt)?;
    let cipher = Aes256GcmSiv::new_from_slice(&key).map_err(|_| VaultError::DecryptionFailed)?;
    let aad = aad_for(&envelope.case_id);
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &ciphertext,
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| VaultError::DecryptionFailed)?;
    let payload: VaultPayload = serde_json::from_slice(&plaintext)?;

    if payload.case_id != envelope.case_id {
        return Err(VaultError::InvalidVault("case id mismatch"));
    }

    Ok(VaultHandle {
        case_id: payload.case_id,
        entries: payload.entries,
    })
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; VAULT_KEY_LEN], VaultError> {
    let params = Params::new(64 * 1024, 3, 1, Some(VAULT_KEY_LEN))
        .map_err(|_| VaultError::InvalidVault("invalid argon2 parameters"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0_u8; VAULT_KEY_LEN];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| VaultError::DecryptionFailed)?;
    Ok(key)
}

fn aad_for(case_id: &str) -> String {
    format!("cpx-vault-v1:{case_id}")
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(hex_char(byte >> 4));
        output.push(hex_char(byte & 0x0f));
    }

    output
}

fn decode_hex_exact(value: &str, expected_len: usize) -> Result<Vec<u8>, VaultError> {
    let decoded = decode_hex(value)?;

    if decoded.len() != expected_len {
        return Err(VaultError::InvalidVault(
            "decoded hex length was unexpected",
        ));
    }

    Ok(decoded)
}

fn decode_hex(value: &str) -> Result<Vec<u8>, VaultError> {
    if value.len() % 2 != 0 {
        return Err(VaultError::InvalidVault("hex data had an odd length"));
    }

    let mut output = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let high = from_hex_digit(bytes[index])?;
        let low = from_hex_digit(bytes[index + 1])?;
        output.push((high << 4) | low);
        index += 2;
    }

    Ok(output)
}

fn hex_char(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'a' + (nibble - 10)) as char,
        _ => unreachable!("nibble must stay within 0..=15"),
    }
}

fn from_hex_digit(value: u8) -> Result<u8, VaultError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(VaultError::InvalidVault(
            "hex data contained an invalid digit",
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::symbolize::{EntityKind, SymbolEntry};

    use super::{open, store, StoreRequest, VaultError};

    #[test]
    fn stores_and_opens_a_case_local_vault() {
        let path = temp_vault_path("roundtrip");
        let entries = vec![
            SymbolEntry {
                kind: EntityKind::Hostname,
                symbol: "H1".to_owned(),
                raw: "ama-prod-17".to_owned(),
            },
            SymbolEntry {
                kind: EntityKind::EmailAddress,
                symbol: "E1".to_owned(),
                raw: "alice@example.com".to_owned(),
            },
        ];

        let handle = store(
            &path,
            &StoreRequest {
                case_id: "canonical-case",
                entries: &entries,
                passphrase: "correct horse battery staple",
            },
        )
        .expect("expected vault storage to succeed");
        let reopened =
            open(&path, "correct horse battery staple").expect("expected vault open to succeed");

        assert_eq!(handle.case_id, "canonical-case");
        assert_eq!(reopened.entries, entries);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn rejects_the_wrong_passphrase() {
        let path = temp_vault_path("wrong-passphrase");
        let entries = vec![SymbolEntry {
            kind: EntityKind::CustomerUrl,
            symbol: "URL1".to_owned(),
            raw: "https://portal.contoso.example.com/case/42".to_owned(),
        }];

        store(
            &path,
            &StoreRequest {
                case_id: "case-42",
                entries: &entries,
                passphrase: "alpha",
            },
        )
        .expect("expected vault storage to succeed");

        let error = open(&path, "beta").expect_err("expected vault open to fail");

        assert!(matches!(error, VaultError::DecryptionFailed));

        let _ = fs::remove_file(path);
    }

    fn temp_vault_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for tests")
            .as_nanos();
        std::env::temp_dir().join(format!("cpx-{name}-{unique}.vault"))
    }
}
