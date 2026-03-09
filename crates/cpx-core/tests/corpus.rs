use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use cpx_core::ingest::{ingest, IngestRequest};
use cpx_core::project::project;
use cpx_core::rehydrate::{rehydrate, RehydrateRequest};
use cpx_core::symbolize::{symbolize, SymbolEntry};
use cpx_core::vault::{open, store, StoreRequest};
use cpx_core::FORMAT_VERSION;
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
    assert!(manifest.minimum_cases_for_v1 >= 10);
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
    let vault_path = temp_vault_path(&case.id);
    let passphrase = "corpus-test-passphrase";
    store(
        &vault_path,
        &StoreRequest {
            case_id: &projection.case_id,
            entries: &symbolized.entries,
            passphrase,
        },
    )
    .expect("expected vault storage to succeed");
    let vault = open(&vault_path, passphrase).expect("expected vault reopen to succeed");
    let rehydrated = rehydrate(&RehydrateRequest {
        projection_response: symbolized.sanitized_contents.clone(),
        vault,
    })
    .expect("expected round-trip rehydration to succeed");

    assert_eq!(
        projection.format_version, FORMAT_VERSION,
        "case {} projection format version differed; {}",
        case.id, case.notes
    );
    assert_projection_starts_with_format_marker(case, &projection.body);
    assert_no_raw_values_leaked(
        case,
        "sanitized output",
        &symbolized.sanitized_contents,
        &symbolized.entries,
    );
    assert_no_raw_values_leaked(
        case,
        "projection output",
        &projection.body,
        &symbolized.entries,
    );
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
    assert_eq!(
        normalize_fixture(&rehydrated),
        normalize_fixture(&document.contents),
        "case {} rehydrated output differed; {}",
        case.id,
        case.notes
    );

    let _ = fs::remove_file(vault_path);
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

fn temp_vault_path(case_id: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for tests")
        .as_nanos();
    std::env::temp_dir().join(format!("cpx-corpus-{case_id}-{unique}.vault"))
}

fn normalize_fixture(contents: &str) -> String {
    contents
        .replace("\r\n", "\n")
        .trim_end_matches('\n')
        .to_owned()
}

fn assert_projection_starts_with_format_marker(case: &CorpusCase, projection_body: &str) {
    let first_line = projection_body
        .lines()
        .next()
        .expect("expected projection output to contain a format line");

    assert_eq!(
        first_line,
        format!("FORMAT {FORMAT_VERSION}"),
        "case {} projection header drifted from the ADR contract; {}",
        case.id,
        case.notes
    );
}

fn assert_no_raw_values_leaked(
    case: &CorpusCase,
    surface_name: &str,
    output: &str,
    entries: &[SymbolEntry],
) {
    for entry in entries {
        assert!(
            !output.contains(&entry.raw),
            "case {} {} leaked the raw value for symbol {}; {}",
            case.id,
            surface_name,
            entry.symbol,
            case.notes
        );
    }
}
