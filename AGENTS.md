# CPX agent instructions

Read these files first:

1. `prd.md`
2. `docs\adr\0001-projection-artifact-format.md`
3. `README.md`
4. `tests\corpus\README.md`

## Commands

Run these once the Rust toolchain is installed:

- On Windows with the MSVC toolchain, make sure Visual Studio Build Tools with the C++ workload are installed so `link.exe` is present.

```powershell
cargo fmt --all
cargo test --workspace
cargo run -p cpx-cli -- --help
```

## Repo structure

- `crates\cpx-cli\` contains the CLI entry point.
- `crates\cpx-core\` contains the core library surfaces.
- `docs\adr\` contains normative implementation decisions.
- `tests\corpus\` contains synthetic fixtures and expected outputs.

## Hard boundaries

- Do not add outbound network calls to the core runtime.
- Do not add real customer data to the repository.
- Do not silently change the artifact format; update the ADR first.
- Do not change product scope in code without updating `prd.md`.
- Do not add large dependencies without a written rationale.

## Preferred working style

- Make small, reviewable changes.
- Add or update tests whenever behavior changes.
- Keep commands deterministic and machine-readable.
- Prefer explicit exit codes and errors over hidden fallback behavior.
- Keep examples aligned with the canonical corpus.

