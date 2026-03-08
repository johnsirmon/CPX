# Contributing to CPX

Thanks for contributing.

## Before making changes

Read:

- `prd.md`
- `AGENTS.md`
- `docs\adr\0001-projection-artifact-format.md`

## Contribution rules

- Keep changes tightly scoped.
- Preserve the local-first trust boundary.
- Do not add real customer data to code, fixtures, or docs.
- Keep AI-generated changes reviewable and validated.
- Update docs or ADRs when changing visible behavior or contracts.

## Development workflow

When the Rust toolchain is installed, use:

- On Windows with the MSVC toolchain, install Visual Studio Build Tools with the C++ workload so `link.exe` is available to Cargo.
- For a fresh Windows shell, run `.\scripts\bootstrap-windows.ps1` first.
- For repeatable local checks, run `.\scripts\validate-local.ps1`.

```powershell
cargo fmt --all
cargo test --workspace
cargo test -p cpx-core --test corpus corpus_cases_match_expected_outputs
cargo run -p cpx-cli -- --help
```

## AI-assisted contributions

AI-generated code is welcome, but it should be:

- reviewed by a human before merge,
- tested or otherwise validated,
- aligned to the PRD and ADRs, and
- accompanied by any required doc updates.

