use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestRequest {
    pub source_name: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseDocument {
    pub source_name: String,
    pub contents: String,
    pub line_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IngestError {
    #[error("input was empty after normalization")]
    EmptyInput,
}

pub fn ingest(request: IngestRequest) -> Result<CaseDocument, IngestError> {
    let normalized = request.contents.replace("\r\n", "\n");
    let trimmed = normalized.trim();

    if trimmed.is_empty() {
        return Err(IngestError::EmptyInput);
    }

    Ok(CaseDocument {
        source_name: request.source_name,
        contents: trimmed.to_owned(),
        line_count: trimmed.lines().count(),
    })
}

#[cfg(test)]
mod tests {
    use super::{ingest, IngestError, IngestRequest};

    #[test]
    fn normalizes_line_endings_and_counts_lines() {
        let document = ingest(IngestRequest {
            source_name: "case.txt".to_owned(),
            contents: "line one\r\nline two\r\n".to_owned(),
        })
        .expect("expected successful ingest");

        assert_eq!(document.source_name, "case.txt");
        assert_eq!(document.contents, "line one\nline two");
        assert_eq!(document.line_count, 2);
    }

    #[test]
    fn rejects_empty_input() {
        let error = ingest(IngestRequest {
            source_name: "case.txt".to_owned(),
            contents: "   \r\n\t".to_owned(),
        })
        .expect_err("expected empty input to fail");

        assert_eq!(error, IngestError::EmptyInput);
    }
}

