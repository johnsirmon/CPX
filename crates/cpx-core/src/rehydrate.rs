use thiserror::Error;

use crate::vault::VaultHandle;
use crate::FORMAT_VERSION;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RehydrateRequest {
    pub projection_response: String,
    pub vault: VaultHandle,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RehydrateError {
    #[error("rehydration input was empty")]
    EmptyInput,
    #[error("rehydration input used unsupported format '{actual}'")]
    FormatMismatch { actual: String },
    #[error("rehydration input case '{actual}' did not match vault case '{expected}'")]
    CaseMismatch { expected: String, actual: String },
}

pub fn rehydrate(request: &RehydrateRequest) -> Result<String, RehydrateError> {
    let contents = request.projection_response.as_str();

    if contents.trim().is_empty() {
        return Err(RehydrateError::EmptyInput);
    }

    if let Some(actual) = detect_format(contents) {
        if actual != FORMAT_VERSION {
            return Err(RehydrateError::FormatMismatch {
                actual: actual.to_owned(),
            });
        }
    }

    if let Some(actual) = detect_case_id(contents) {
        if actual != request.vault.case_id {
            return Err(RehydrateError::CaseMismatch {
                expected: request.vault.case_id.clone(),
                actual: actual.to_owned(),
            });
        }
    }

    Ok(rehydrate_symbols(contents, &request.vault))
}

pub fn detect_case_id(contents: &str) -> Option<&str> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("CASE ")
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

fn detect_format(contents: &str) -> Option<&str> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("FORMAT ")
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

fn rehydrate_symbols(contents: &str, vault: &VaultHandle) -> String {
    let mut mappings = vault
        .entries
        .iter()
        .map(|entry| (entry.symbol.as_str(), entry.raw.as_str()))
        .collect::<Vec<_>>();
    mappings.sort_by(|left, right| {
        right
            .0
            .len()
            .cmp(&left.0.len())
            .then_with(|| left.0.cmp(right.0))
    });

    let mut output = String::with_capacity(contents.len());
    let mut index = 0;

    while index < contents.len() {
        if let Some((symbol, raw)) = mappings.iter().find(|(symbol, _)| {
            contents[index..].starts_with(symbol)
                && has_symbol_boundaries(contents, index, symbol.len())
        }) {
            output.push_str(raw);
            index += symbol.len();
            continue;
        }

        let ch = contents[index..]
            .chars()
            .next()
            .expect("index should stay inside the string");
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

fn has_symbol_boundaries(contents: &str, start: usize, len: usize) -> bool {
    let before_ok = start == 0
        || !contents[..start]
            .chars()
            .next_back()
            .expect("start > 0 implies a previous character")
            .is_ascii_alphanumeric();
    let end = start + len;
    let after_ok = end == contents.len()
        || !contents[end..]
            .chars()
            .next()
            .expect("end < len implies a next character")
            .is_ascii_alphanumeric();

    before_ok && after_ok
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::ingest::{ingest, IngestRequest};
    use crate::symbolize::symbolize;
    use crate::vault::{store, StoreRequest};

    use super::{rehydrate, RehydrateError, RehydrateRequest};

    #[test]
    fn rehydrates_the_canonical_case() {
        let document = ingest(IngestRequest {
            source_name: "canonical-case.txt".to_owned(),
            contents: include_str!("../../../tests/corpus/canonical-case/input.txt").to_owned(),
        })
        .expect("expected ingest to succeed");
        let symbolized = symbolize(&document).expect("expected symbolization to succeed");
        let vault_path = temp_vault_path("canonical-roundtrip");
        let vault = store(
            &vault_path,
            &StoreRequest {
                case_id: "canonical-case",
                entries: &symbolized.entries,
                passphrase: "test-passphrase",
            },
        )
        .expect("expected vault store to succeed");

        let rehydrated = rehydrate(&RehydrateRequest {
            projection_response: normalize_fixture(&symbolized.sanitized_contents),
            vault,
        })
        .expect("expected rehydration to succeed");

        assert_eq!(
            normalize_fixture(&rehydrated),
            normalize_fixture(&document.contents)
        );

        let _ = std::fs::remove_file(vault_path);
    }

    #[test]
    fn rejects_a_mismatched_case_id() {
        let error = rehydrate(&RehydrateRequest {
            projection_response: "FORMAT cpx-v1\nCASE case-two\nEVENTS\n t+00 H1".to_owned(),
            vault: crate::vault::VaultHandle {
                case_id: "case-one".to_owned(),
                entries: Vec::new(),
            },
        })
        .expect_err("expected a mismatched case id to fail");

        assert_eq!(
            error,
            RehydrateError::CaseMismatch {
                expected: "case-one".to_owned(),
                actual: "case-two".to_owned(),
            }
        );
    }

    #[test]
    fn rehydrates_symbols_wrapped_in_punctuation() {
        let rehydrated = rehydrate(&RehydrateRequest {
            projection_response: "Summary: [H1] retried URL1 after S1>S2.".to_owned(),
            vault: crate::vault::VaultHandle {
                case_id: "case-one".to_owned(),
                entries: vec![
                    crate::symbolize::SymbolEntry {
                        kind: crate::symbolize::EntityKind::Hostname,
                        symbol: "H1".to_owned(),
                        raw: "ama-prod-17".to_owned(),
                    },
                    crate::symbolize::SymbolEntry {
                        kind: crate::symbolize::EntityKind::CustomerUrl,
                        symbol: "URL1".to_owned(),
                        raw: "https://portal.contoso.example.com/case/42".to_owned(),
                    },
                    crate::symbolize::SymbolEntry {
                        kind: crate::symbolize::EntityKind::SubscriptionId,
                        symbol: "S1".to_owned(),
                        raw: "11111111-2222-3333-4444-555555555555".to_owned(),
                    },
                    crate::symbolize::SymbolEntry {
                        kind: crate::symbolize::EntityKind::SubscriptionId,
                        symbol: "S2".to_owned(),
                        raw: "66666666-7777-8888-9999-000000000000".to_owned(),
                    },
                ],
            },
        })
        .expect("expected punctuation-wrapped symbols to rehydrate");

        assert_eq!(
            rehydrated,
            "Summary: [ama-prod-17] retried https://portal.contoso.example.com/case/42 after 11111111-2222-3333-4444-555555555555>66666666-7777-8888-9999-000000000000."
        );
    }

    fn normalize_fixture(contents: &str) -> String {
        contents
            .replace("\r\n", "\n")
            .trim_end_matches('\n')
            .to_owned()
    }

    fn temp_vault_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for tests")
            .as_nanos();
        std::env::temp_dir().join(format!("cpx-{name}-{unique}.vault"))
    }
}
