# CPX Copilot Instructions

Read these first when behavior, scope, or format questions come up:

1. `prd.md`
2. `docs\adr\0001-projection-artifact-format.md`
3. `README.md`
4. `AGENTS.md`
5. `tests\corpus\README.md`

## Build, test, and validation commands

- On Windows with the MSVC toolchain, install Visual Studio Build Tools with the C++ workload so `link.exe` is available.
- For a fresh Windows shell, run `.\scripts\bootstrap-windows.ps1` first.
- For repeatable local validation, run `.\scripts\validate-local.ps1`.
- If formatting needs to be rewritten before validation, run `.\scripts\validate-local.ps1 -WriteFormatting`.
- Format the workspace: `cargo fmt --all`
- Build the CLI: `cargo build -p cpx-cli`
- Run all tests: `cargo test --workspace`
- Run the corpus integration test: `cargo test -p cpx-core --test corpus corpus_cases_match_expected_outputs`
- Run a single focused unit test: `cargo test -p cpx-core reuses_symbols_for_repeated_values`
- Check the CLI shape: `cargo run -p cpx-cli -- --help`
- Exercise the local end-to-end flow: set `CPX_PASSPHRASE`, then run `cargo run -p cpx-cli -- project <input> --output projection.txt` followed by `cargo run -p cpx-cli -- rehydrate <symbolic-text> --vault canonical-case.cpxvault --output trusted-output.txt`

## High-level architecture

- `crates\cpx-cli` is a thin Clap-based entry point. The `project` command validates `--format`, then runs `ingest -> symbolize -> project` from `cpx-core` and can write a case-local encrypted vault when `CPX_PASSPHRASE` (or another `--passphrase-env`) is set. The `ingest` command only emits a small summary (`SOURCE`, `LINES`, `CHARS`). The `rehydrate` command reads symbolic text plus a vault file and restores raw values locally.
- `crates\cpx-core` owns the actual pipeline:
  - `ingest.rs` normalizes CRLF to LF, trims the case text, and rejects empty input.
  - `symbolize.rs` performs deterministic, rule-based replacement for customer names, URLs, Azure resource IDs, email addresses, usernames, hostnames, IPv4 addresses, labeled tenant/subscription UUIDs, and generic UUIDs.
  - `project.rs` emits the line-oriented `cpx-v1` artifact defined by ADR 0001. Current artifacts include `FORMAT`, `CASE`, `SUMMARY`, and `EVENTS`, with one `t+NN` event line per sanitized input line.
  - `vault.rs` stores case-local symbol mappings in an encrypted vault file derived from a user-supplied passphrase and never emits the raw map into projection output.
  - `rehydrate.rs` restores symbolic model output back to raw local values using the vault and rejects mismatched case IDs or unsupported format markers.
- Corpus validation lives in `crates\cpx-core\tests\corpus.rs`. It loads `tests\corpus\manifest.json`, runs the core pipeline over each synthetic fixture, compares sanitized output and projection output against the expected files, and asserts round-trip rehydration through the local vault.

## Key conventions

- Follow the repository source-of-truth order from `README.md`: `prd.md` -> `docs\adr\` -> `.github\` instructions -> `AGENTS.md`.
- Do not add outbound network calls to the core runtime. CPX is local-first, and artifacts must stay safe for downstream model use without exposing raw identifiers.
- Do not silently change the `cpx-v1` artifact format. If the projection contract changes, update `docs\adr\0001-projection-artifact-format.md` first.
- Projection artifacts must never contain the raw symbol-to-value map. The format version marker must remain explicit, and `FORMAT cpx-v1` must stay the first non-empty line.
- The encrypted vault is part of the local-only trust boundary. Passphrases come from an environment variable (default `CPX_PASSPHRASE`), and raw values should only appear again on the trusted rehydration path.
- Determinism is part of the product contract here: keep the symbolization pass order stable, preserve the existing `SUMMARY`/`EVENTS` ordering, keep `t+NN` zero-padded event numbering, and reuse the same symbol when the same raw value of the same entity kind repeats within one case.
- Entity symbols are kind-specific and stable within a case. Current prefixes in `symbolize.rs` are `C`, `T`, `S`, `E`, `U`, `H`, `IP`, `R`, `URL`, and `ID`; keep new behavior aligned with those conventions unless the PRD or ADR changes.
- `symbolize.rs` ends with a safety check that fails if detectable sensitive-looking content remains. Treat that as a hard failure, not a warning or best-effort fallback.
- CLI behavior uses explicit exit codes from `crates\cpx-core\src\lib.rs`. Preserve those mappings when adding new command paths or failure modes.
- The synthetic corpus is the canonical regression baseline. Never add real customer data, keep expected outputs next to each fixture, and update `tests\corpus\manifest.json` when adding or changing corpus cases. The current release gate expects 10 total cases including adversarial coverage and round-trip assertions.
- The workspace forbids `unsafe_code`; keep additions compatible with the workspace lint settings in the root `Cargo.toml`.
