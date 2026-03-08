use std::fs;
use std::path::{Path, PathBuf};

use cpx_core::ingest::{ingest, IngestRequest};
use cpx_core::project::project;
use cpx_core::symbolize::symbolize;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    version: u32,
    minimum_cases_for_v1: usize,
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
struct CorpusCase {
    id: String,
    input: String,
    expected_sanitized: String,
    expected_projection: String,
    expected_symbol_count: usize,
    notes: String,
}

#[test]
fn corpus_cases_match_expected_outputs() {
    let corpus_dir = corpus_dir();
    let manifest_text = fs::read_to_string(corpus_dir.join("manifest.json"))
        .expect("expected corpus manifest to be readable");
    let manifest: CorpusManifest =
        serde_json::from_str(&manifest_text).expect("expected manifest json to parse");

    assert_eq!(manifest.version, 1);
    assert!(manifest.minimum_cases_for_v1 >= 8);
    assert!(manifest.cases.len() >= manifest.minimum_cases_for_v1);
    assert!(
        manifest
            .cases
            .iter()
            .filter(|case| case.id.starts_with("adversarial-"))
            .count()
            >= 2,
        "expected at least two adversarial corpus cases"
    );

    for case in &manifest.cases {
        run_case(&corpus_dir, case);
    }
}

fn run_case(corpus_dir: &Path, case: &CorpusCase) {
    let input = read_fixture(corpus_dir, &case.input);
    let expected_sanitized = read_fixture(corpus_dir, &case.expected_sanitized);
    let expected_projection = read_fixture(corpus_dir, &case.expected_projection);

    let document = ingest(IngestRequest {
        source_name: format!("{}.txt", case.id),
        contents: input,
    })
    .expect("expected ingest to succeed");
    let symbolized = symbolize(&document).expect("expected symbolization to succeed");
    let projection = project(&symbolized).expect("expected projection to succeed");

    assert_eq!(
        normalize_fixture(&symbolized.sanitized_contents),
        normalize_fixture(&expected_sanitized),
        "case {} sanitized output differed; {}",
        case.id,
        case.notes
    );
    assert_eq!(
        symbolized.symbol_count(),
        case.expected_symbol_count,
        "case {} symbol count differed; {}",
        case.id,
        case.notes
    );
    assert_eq!(
        normalize_fixture(&projection.body),
        normalize_fixture(&expected_projection),
        "case {} projection output differed; {}",
        case.id,
        case.notes
    );
}

fn read_fixture(corpus_dir: &Path, relative_path: &str) -> String {
    let path = corpus_dir.join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "expected fixture '{}' to be readable: {error}",
            path.display()
        )
    })
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/corpus")
        .canonicalize()
        .expect("expected corpus directory to exist")
}

fn normalize_fixture(contents: &str) -> String {
    contents
        .replace("\r\n", "\n")
        .trim_end_matches('\n')
        .to_owned()
}
