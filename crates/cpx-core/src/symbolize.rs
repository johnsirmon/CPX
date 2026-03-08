use crate::ingest::CaseDocument;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    CustomerName,
    TenantId,
    SubscriptionId,
    EmailAddress,
    Username,
    Hostname,
    IpAddress,
    ResourceId,
    CustomerUrl,
    InternalIdentifier,
}

impl EntityKind {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::CustomerName => "C",
            Self::TenantId => "T",
            Self::SubscriptionId => "S",
            Self::EmailAddress => "E",
            Self::Username => "U",
            Self::Hostname => "H",
            Self::IpAddress => "IP",
            Self::ResourceId => "R",
            Self::CustomerUrl => "URL",
            Self::InternalIdentifier => "ID",
        }
    }

    pub fn summary_key(self) -> &'static str {
        match self {
            Self::CustomerName => "customer_name",
            Self::TenantId => "tenant_id",
            Self::SubscriptionId => "subscription_id",
            Self::EmailAddress => "email_address",
            Self::Username => "username",
            Self::Hostname => "hostname",
            Self::IpAddress => "ip_address",
            Self::ResourceId => "resource_id",
            Self::CustomerUrl => "customer_url",
            Self::InternalIdentifier => "internal_identifier",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub kind: EntityKind,
    pub symbol: String,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolizedCase {
    pub source_name: String,
    pub sanitized_contents: String,
    pub entries: Vec<SymbolEntry>,
    pub line_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SymbolizeError {
    #[error("detectable sensitive content remained after symbolization")]
    UnsanitizedContent,
}

impl SymbolizedCase {
    pub fn symbol_count(&self) -> usize {
        self.entries.len()
    }

    pub fn count_by_kind(&self, kind: EntityKind) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.kind == kind)
            .count()
    }
}

pub fn symbolize(document: &CaseDocument) -> Result<SymbolizedCase, SymbolizeError> {
    let mut entries = Vec::new();
    let sanitized_lines = document
        .contents
        .lines()
        .map(|line| sanitize_line(line, &mut entries))
        .collect::<Vec<_>>();
    let sanitized_contents = sanitized_lines.join("\n");

    if contains_detectable_sensitive_content(&sanitized_contents) {
        return Err(SymbolizeError::UnsanitizedContent);
    }

    Ok(SymbolizedCase {
        source_name: document.source_name.clone(),
        sanitized_contents,
        entries,
        line_count: document.line_count,
    })
}

fn sanitize_line(line: &str, entries: &mut Vec<SymbolEntry>) -> String {
    let line = replace_customer_name(line, entries);
    let line = replace_url_tokens(&line, entries);
    let line = replace_resource_ids(&line, entries);
    let line = replace_email_tokens(&line, entries);
    let line = replace_labeled_usernames(&line, entries);
    let line = replace_host_tokens(&line, entries);
    let line = replace_ip_tokens(&line, entries);
    let line = replace_labeled_uuids(&line, "Tenant:", EntityKind::TenantId, entries);
    let line = replace_labeled_uuids(&line, "Subscription:", EntityKind::SubscriptionId, entries);
    replace_generic_uuid_tokens(&line, entries)
}

fn replace_customer_name(line: &str, entries: &mut Vec<SymbolEntry>) -> String {
    const PREFIX: &str = "Customer ";
    const SUFFIX: &str = " reported";

    if let Some(prefix_start) = line.find(PREFIX) {
        let name_start = prefix_start + PREFIX.len();
        if let Some(name_end_rel) = line[name_start..].find(SUFFIX) {
            let name_end = name_start + name_end_rel;
            let raw = &line[name_start..name_end];

            if !raw.trim().is_empty() {
                let symbol = symbol_for(entries, EntityKind::CustomerName, raw);
                let mut output = String::with_capacity(line.len());
                output.push_str(&line[..name_start]);
                output.push_str(&symbol);
                output.push_str(&line[name_end..]);
                return output;
            }
        }
    }

    line.to_owned()
}

fn replace_resource_ids(line: &str, entries: &mut Vec<SymbolEntry>) -> String {
    replace_token_matches(line, entries, EntityKind::ResourceId, |token| {
        token.starts_with("/subscriptions/")
    })
}

fn replace_url_tokens(line: &str, entries: &mut Vec<SymbolEntry>) -> String {
    replace_token_matches(line, entries, EntityKind::CustomerUrl, looks_like_url)
}

fn replace_email_tokens(line: &str, entries: &mut Vec<SymbolEntry>) -> String {
    replace_token_matches(line, entries, EntityKind::EmailAddress, looks_like_email)
}

fn replace_labeled_usernames(line: &str, entries: &mut Vec<SymbolEntry>) -> String {
    let line = replace_labeled_tokens(
        line,
        "Username:",
        EntityKind::Username,
        entries,
        looks_like_username,
    );
    replace_labeled_tokens(
        &line,
        "User:",
        EntityKind::Username,
        entries,
        looks_like_username,
    )
}

fn replace_ip_tokens(line: &str, entries: &mut Vec<SymbolEntry>) -> String {
    replace_token_matches(line, entries, EntityKind::IpAddress, looks_like_ipv4)
}

fn replace_host_tokens(line: &str, entries: &mut Vec<SymbolEntry>) -> String {
    const KEYWORD: &str = "host ";

    let mut output = String::new();
    let mut cursor = 0;

    while let Some(keyword_rel) = line[cursor..].find(KEYWORD) {
        let keyword_start = cursor + keyword_rel;
        let token_start = keyword_start + KEYWORD.len();
        let token_end = scan_token_end(line, token_start);

        output.push_str(&line[cursor..token_start]);

        let token = &line[token_start..token_end];
        let (prefix, core, suffix) = split_token_affixes(token);

        if looks_like_hostname(core) {
            let symbol = symbol_for(entries, EntityKind::Hostname, core);
            output.push_str(prefix);
            output.push_str(&symbol);
            output.push_str(suffix);
        } else {
            output.push_str(token);
        }

        cursor = token_end;
    }

    output.push_str(&line[cursor..]);
    output
}

fn replace_labeled_tokens<F>(
    line: &str,
    marker: &str,
    kind: EntityKind,
    entries: &mut Vec<SymbolEntry>,
    predicate: F,
) -> String
where
    F: Fn(&str) -> bool,
{
    let mut output = String::new();
    let mut cursor = 0;

    while let Some(marker_rel) = line[cursor..].find(marker) {
        let marker_start = cursor + marker_rel;
        let token_start = marker_start + marker.len();
        let token_start = skip_spaces(line, token_start);
        let token_end = scan_token_end(line, token_start);

        output.push_str(&line[cursor..token_start]);

        let token = &line[token_start..token_end];
        let (prefix, core, suffix) = split_token_affixes(token);

        if predicate(core) {
            let symbol = symbol_for(entries, kind, core);
            output.push_str(prefix);
            output.push_str(&symbol);
            output.push_str(suffix);
        } else {
            output.push_str(token);
        }

        cursor = token_end;
    }

    output.push_str(&line[cursor..]);
    output
}

fn replace_labeled_uuids(
    line: &str,
    marker: &str,
    kind: EntityKind,
    entries: &mut Vec<SymbolEntry>,
) -> String {
    let mut output = String::new();
    let mut cursor = 0;

    while let Some(marker_rel) = line[cursor..].find(marker) {
        let marker_start = cursor + marker_rel;
        let token_start = marker_start + marker.len();
        let token_start = skip_spaces(line, token_start);
        let token_end = scan_token_end(line, token_start);

        output.push_str(&line[cursor..token_start]);

        let token = &line[token_start..token_end];
        let (prefix, core, suffix) = split_token_affixes(token);

        if looks_like_uuid(core) {
            let symbol = symbol_for(entries, kind, core);
            output.push_str(prefix);
            output.push_str(&symbol);
            output.push_str(suffix);
        } else {
            output.push_str(token);
        }

        cursor = token_end;
    }

    output.push_str(&line[cursor..]);
    output
}

fn replace_generic_uuid_tokens(line: &str, entries: &mut Vec<SymbolEntry>) -> String {
    replace_token_matches(
        line,
        entries,
        EntityKind::InternalIdentifier,
        looks_like_uuid,
    )
}

fn replace_token_matches<F>(
    line: &str,
    entries: &mut Vec<SymbolEntry>,
    kind: EntityKind,
    predicate: F,
) -> String
where
    F: Fn(&str) -> bool,
{
    let mut output = String::with_capacity(line.len());
    let mut cursor = 0;

    while cursor < line.len() {
        let next_char = line[cursor..]
            .chars()
            .next()
            .expect("cursor should stay in bounds");

        if next_char.is_whitespace() {
            output.push(next_char);
            cursor += next_char.len_utf8();
            continue;
        }

        let token_end = scan_token_end(line, cursor);
        let token = &line[cursor..token_end];
        let (prefix, core, suffix) = split_token_affixes(token);

        if predicate(core) {
            let symbol = symbol_for(entries, kind, core);
            output.push_str(prefix);
            output.push_str(&symbol);
            output.push_str(suffix);
        } else {
            output.push_str(token);
        }

        cursor = token_end;
    }

    output
}

fn symbol_for(entries: &mut Vec<SymbolEntry>, kind: EntityKind, raw: &str) -> String {
    if let Some(existing) = entries
        .iter()
        .find(|entry| entry.kind == kind && entry.raw == raw)
    {
        return existing.symbol.clone();
    }

    let index = entries.iter().filter(|entry| entry.kind == kind).count() + 1;
    let symbol = format!("{}{}", kind.prefix(), index);

    entries.push(SymbolEntry {
        kind,
        symbol: symbol.clone(),
        raw: raw.to_owned(),
    });

    symbol
}

fn contains_detectable_sensitive_content(contents: &str) -> bool {
    contents.lines().any(|line| {
        line.split_whitespace().any(|token| {
            let (_, core, _) = split_token_affixes(token);
            core.starts_with("/subscriptions/")
                || looks_like_url(core)
                || looks_like_email(core)
                || looks_like_ipv4(core)
                || looks_like_uuid(core)
                || looks_like_hostname(core)
        })
    })
}

fn looks_like_url(candidate: &str) -> bool {
    if !(candidate.starts_with("https://") || candidate.starts_with("http://")) {
        return false;
    }

    let remainder = candidate.split_once("://").map_or("", |(_, value)| value);
    !remainder.is_empty() && remainder.contains('.')
}

fn looks_like_email(candidate: &str) -> bool {
    let Some((local, domain)) = candidate.split_once('@') else {
        return false;
    };

    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn looks_like_hostname(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '.' | '_'))
        && candidate.chars().any(|ch| matches!(ch, '-' | '.'))
}

fn looks_like_username(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        && candidate.chars().any(|ch| ch.is_ascii_alphabetic())
}

fn looks_like_ipv4(candidate: &str) -> bool {
    let parts = candidate.split('.').collect::<Vec<_>>();

    if parts.len() != 4 {
        return false;
    }

    parts.iter().all(|part| {
        !part.is_empty()
            && part.len() <= 3
            && part.chars().all(|ch| ch.is_ascii_digit())
            && part.parse::<u8>().is_ok()
    })
}

fn looks_like_uuid(candidate: &str) -> bool {
    let parts = candidate.split('-').collect::<Vec<_>>();

    if parts.len() != 5 {
        return false;
    }

    let expected_lengths = [8, 4, 4, 4, 12];
    parts
        .iter()
        .zip(expected_lengths)
        .all(|(part, len)| part.len() == len && part.chars().all(|ch| ch.is_ascii_hexdigit()))
}

fn scan_token_end(line: &str, start: usize) -> usize {
    let mut end = start;

    while end < line.len() {
        let ch = line[end..]
            .chars()
            .next()
            .expect("end should stay inside the string");

        if ch.is_whitespace() {
            break;
        }

        end += ch.len_utf8();
    }

    end
}

fn skip_spaces(line: &str, start: usize) -> usize {
    let mut index = start;

    while index < line.len() {
        let ch = line[index..]
            .chars()
            .next()
            .expect("index should stay inside the string");

        if !ch.is_whitespace() {
            break;
        }

        index += ch.len_utf8();
    }

    index
}

fn split_token_affixes(token: &str) -> (&str, &str, &str) {
    let prefix_len = token
        .chars()
        .take_while(|ch| matches!(ch, '(' | '[' | '{' | '"' | '\''))
        .map(char::len_utf8)
        .sum::<usize>();
    let suffix_len = token[prefix_len..]
        .chars()
        .rev()
        .take_while(|ch| matches!(ch, '.' | ',' | ';' | ')' | ':' | ']' | '}' | '"' | '\''))
        .map(char::len_utf8)
        .sum::<usize>();
    let core_end = token.len().saturating_sub(suffix_len);
    let prefix = &token[..prefix_len];
    let core = &token[prefix_len..core_end];
    let suffix = &token[core_end..];

    (prefix, core, suffix)
}

#[cfg(test)]
mod tests {
    use crate::ingest::{ingest, IngestRequest};

    use super::{symbolize, EntityKind};

    #[test]
    fn symbolizes_the_canonical_case() {
        let document = ingest(IngestRequest {
            source_name: "canonical-case.txt".to_owned(),
            contents: include_str!("../../../tests/corpus/canonical-case/input.txt").to_owned(),
        })
        .expect("expected ingest to succeed");

        let symbolized = symbolize(&document).expect("expected symbolization to succeed");
        let expected = normalize_fixture(include_str!(
            "../../../tests/corpus/canonical-case/expected-sanitized.txt"
        ));

        assert_eq!(normalize_fixture(&symbolized.sanitized_contents), expected);
        assert_eq!(symbolized.symbol_count(), 5);
        assert_eq!(symbolized.count_by_kind(EntityKind::CustomerName), 1);
        assert_eq!(symbolized.count_by_kind(EntityKind::Hostname), 1);
        assert_eq!(symbolized.count_by_kind(EntityKind::ResourceId), 1);
        assert_eq!(symbolized.count_by_kind(EntityKind::EmailAddress), 1);
        assert_eq!(symbolized.count_by_kind(EntityKind::TenantId), 1);
    }

    #[test]
    fn symbolizes_usernames_ip_addresses_and_urls() {
        let document = ingest(IngestRequest {
            source_name: "extended.txt".to_owned(),
            contents: "Username: svc-collector\nClient IP: 10.42.0.7\nPortal URL: https://contoso.example.com/support/case/42".to_owned(),
        })
        .expect("expected ingest to succeed");

        let symbolized = symbolize(&document).expect("expected symbolization to succeed");

        assert_eq!(
            symbolized.sanitized_contents,
            "Username: U1\nClient IP: IP1\nPortal URL: URL1"
        );
        assert_eq!(symbolized.count_by_kind(EntityKind::Username), 1);
        assert_eq!(symbolized.count_by_kind(EntityKind::IpAddress), 1);
        assert_eq!(symbolized.count_by_kind(EntityKind::CustomerUrl), 1);
    }

    #[test]
    fn preserves_wrapping_punctuation_around_symbols() {
        let document = ingest(IngestRequest {
            source_name: "punctuation.txt".to_owned(),
            contents: "Username: [svc-collector]\nPortal URL: (https://portal.contoso.example.com/cases/777),".to_owned(),
        })
        .expect("expected ingest to succeed");

        let symbolized = symbolize(&document).expect("expected symbolization to succeed");

        assert_eq!(
            symbolized.sanitized_contents,
            "Username: [U1]\nPortal URL: (URL1),"
        );
    }

    #[test]
    fn reuses_symbols_for_repeated_values() {
        let document = ingest(IngestRequest {
            source_name: "repeat.txt".to_owned(),
            contents: "Contact: alice@example.com\nContact: alice@example.com".to_owned(),
        })
        .expect("expected ingest to succeed");

        let symbolized = symbolize(&document).expect("expected symbolization to succeed");

        assert_eq!(symbolized.sanitized_contents, "Contact: E1\nContact: E1");
        assert_eq!(symbolized.symbol_count(), 1);
    }

    #[test]
    fn fails_when_a_detectable_hostname_remains_unsanitized() {
        let document = ingest(IngestRequest {
            source_name: "host.txt".to_owned(),
            contents: "Observed ama-prod-17 without an explicit host marker".to_owned(),
        })
        .expect("expected ingest to succeed");

        let error = symbolize(&document).expect_err("expected unsanitized hostname to fail");

        assert_eq!(error, super::SymbolizeError::UnsanitizedContent);
    }

    fn normalize_fixture(contents: &str) -> String {
        contents
            .replace("\r\n", "\n")
            .trim_end_matches('\n')
            .to_owned()
    }
}
