use thiserror::Error;

use crate::symbolize::{EntityKind, SymbolizedCase};
use crate::FORMAT_VERSION;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projection {
    pub format_version: &'static str,
    pub case_id: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProjectError {
    #[error("cannot project an empty case")]
    EmptyCase,
}

pub fn project(case: &SymbolizedCase) -> Result<Projection, ProjectError> {
    if case.sanitized_contents.trim().is_empty() {
        return Err(ProjectError::EmptyCase);
    }

    let case_id = case_id_from_source(&case.source_name);
    let mut body = String::new();

    body.push_str("FORMAT ");
    body.push_str(FORMAT_VERSION);
    body.push('\n');
    body.push_str("CASE ");
    body.push_str(&case_id);
    body.push('\n');
    body.push_str("SUMMARY\n");
    body.push_str(" line_count=");
    body.push_str(&case.line_count.to_string());
    body.push('\n');
    body.push_str(" symbolized_entities=");
    body.push_str(&case.symbol_count().to_string());
    body.push('\n');

    for kind in [
        EntityKind::CustomerName,
        EntityKind::TenantId,
        EntityKind::SubscriptionId,
        EntityKind::EmailAddress,
        EntityKind::Username,
        EntityKind::Hostname,
        EntityKind::IpAddress,
        EntityKind::ResourceId,
        EntityKind::CustomerUrl,
        EntityKind::InternalIdentifier,
    ] {
        let count = case.count_by_kind(kind);

        if count > 0 {
            body.push(' ');
            body.push_str(kind.summary_key());
            body.push('=');
            body.push_str(&count.to_string());
            body.push('\n');
        }
    }

    body.push_str("EVENTS\n");

    for (index, line) in case.sanitized_contents.lines().enumerate() {
        body.push_str(" t+");
        body.push_str(&format!("{index:02}"));
        body.push(' ');
        body.push_str(line);
        body.push('\n');
    }

    Ok(Projection {
        format_version: FORMAT_VERSION,
        case_id,
        body,
    })
}

fn case_id_from_source(source_name: &str) -> String {
    let segments = source_name
        .split(|ch| ch == '\\' || ch == '/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let file_name = segments.last().copied().unwrap_or(source_name);
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(name, _)| name);
    let normalized_stem = normalize_case_component(stem);

    if normalized_stem == "input" {
        if let Some(parent) = segments.iter().rev().nth(1) {
            let normalized_parent = normalize_case_component(parent);

            if !normalized_parent.is_empty() {
                return normalized_parent;
            }
        }
    }

    if normalized_stem.is_empty() {
        return "case".to_owned();
    }

    normalized_stem
}

fn normalize_case_component(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_was_dash = false;
        } else if !previous_was_dash {
            normalized.push('-');
            previous_was_dash = true;
        }
    }

    normalized.trim_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use crate::ingest::{ingest, IngestRequest};
    use crate::symbolize::symbolize;

    use super::project;

    #[test]
    fn projects_the_canonical_case() {
        let document = ingest(IngestRequest {
            source_name: "canonical-case.txt".to_owned(),
            contents: include_str!("../../../tests/corpus/canonical-case/input.txt").to_owned(),
        })
        .expect("expected ingest to succeed");
        let symbolized = symbolize(&document).expect("expected symbolization to succeed");
        let projection = project(&symbolized).expect("expected projection to succeed");
        let expected = normalize_fixture(include_str!(
            "../../../tests/corpus/canonical-case/expected-projection.txt"
        ));

        assert_eq!(projection.case_id, "canonical-case");
        assert_eq!(normalize_fixture(&projection.body), expected);
    }

    #[test]
    fn uses_parent_directory_when_the_file_name_is_input() {
        let document = ingest(IngestRequest {
            source_name: r"C:\fixtures\canonical-case\input.txt".to_owned(),
            contents: include_str!("../../../tests/corpus/canonical-case/input.txt").to_owned(),
        })
        .expect("expected ingest to succeed");
        let symbolized = symbolize(&document).expect("expected symbolization to succeed");
        let projection = project(&symbolized).expect("expected projection to succeed");

        assert_eq!(projection.case_id, "canonical-case");
    }

    fn normalize_fixture(contents: &str) -> String {
        contents
            .replace("\r\n", "\n")
            .trim_end_matches('\n')
            .to_owned()
    }
}
