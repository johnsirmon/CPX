# CPX

CPX is a local-first CLI for preparing support case material for AI-assisted workflows without sending raw customer identifiers to an external model.

This repository is initialized from `prd.md` and is intentionally scaffolded for AI-assisted implementation. The current focus is completing the local-only end-to-end workflow while keeping the repo explicit enough for both humans and coding agents to extend safely.

## Current status

- Product requirements: `prd.md`
- Artifact format ADR: `docs\adr\0001-projection-artifact-format.md`
- Agent guidance: `AGENTS.md`
- Contribution rules: `CONTRIBUTING.md`
- Starter corpus path: `tests\corpus\`
- Canonical synthetic example: `tests\corpus\canonical-case\`

The codebase now includes normalized ingest, deterministic rule-based symbolization for the default v1 entity categories, projection output for the `cpx-v1` text artifact, encrypted case-local vault storage, local rehydration, and a corpus-driven validation baseline with ten synthetic cases.

If any file above does not exist, skip it and continue.

## Source-of-truth order

When instructions conflict, follow this order:

1. `prd.md`
2. ADRs in `docs/adr/`
3. repository instructions in `.github/`
4. `AGENTS.md`
5. inline code comments and local conventions

Do not change user-visible behavior, artifact contracts, or product scope without updating the relevant source-of-truth document.


## Windows quick start

For a Windows-first shell, start here:

```powershell
.\scripts\bootstrap-windows.ps1
```

What the bootstrap script does:

- adds `%USERPROFILE%\.cargo\bin` to `PATH` for the current PowerShell session when Rust is installed but not yet visible in the shell,
- imports the Visual Studio C++ build environment into the current session when Build Tools are installed but `link.exe` is not yet on `PATH`, and
- runs the local validation script once prerequisites are available.

If Rust or Visual Studio Build Tools are missing, the script exits with an explicit error and prints the next install step to unblock the session.

This repository also pins the stable Rust channel plus `rustfmt` in `rust-toolchain.toml`, so a standard `rustup` install will resolve the expected formatter automatically for this workspace.

## Planned workflow

Once the Rust toolchain is installed:

- On Windows with the MSVC toolchain, install Visual Studio Build Tools with the C++ workload so `link.exe` is available.
- Use `.\scripts\validate-local.ps1` for CI-aligned local validation without rewriting formatting.
- Use `.\scripts\validate-local.ps1 -WriteFormatting` if you want the formatter to rewrite files before validation.

Canonical commands:

```powershell
cargo fmt --all
cargo test --workspace
cargo test -p cpx-core --test corpus corpus_cases_match_expected_outputs
cargo run -p cpx-cli -- --help
```

Current CLI shape:

```text
cpx ingest <input-path-or-stdin>
cpx project <input-path-or-stdin> --format cpx-v1 --output projection.txt [--vault-output case.cpxvault]
cpx rehydrate <model-output-or-symbolic-text> --vault case.cpxvault --output trusted-output.txt
```

If the `CPX_PASSPHRASE` environment variable is set, `cpx project` will also write a case-local `.cpxvault` file alongside the projection output directory by default.

## End-to-end local workflow

```powershell
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"

cargo run -p cpx-cli -- project .\tests\corpus\canonical-case\input.txt --output .\out\projection.txt
cargo run -p cpx-cli -- rehydrate .\tests\corpus\canonical-case\expected-sanitized.txt --vault .\out\canonical-case.cpxvault --output .\out\trusted-output.txt
```

Expected result:

- `projection.txt` contains only symbolic values and the `FORMAT cpx-v1` artifact.
- `canonical-case.cpxvault` stores the encrypted local symbol mappings for that case.
- `trusted-output.txt` restores the symbolic text back to the original local values.

## Repository layout

```text
crates\
  cpx-cli\     Binary entry point for the CPX CLI
  cpx-core\    Core library surfaces for ingest, symbolize, project, rehydrate, and vault
docs\
  adr\         Normative architecture and contract decisions
scripts\
  *.ps1        Windows-first bootstrap and validation helpers
tests\
  corpus\      Synthetic validation corpus and canonical examples
```

## Implementation principles

- Keep the core runtime local-first and network-free.
- Keep dependencies minimal and auditable.
- Prefer explicit contracts over inferred behavior.
- Never use real customer data in examples or tests.
- Treat `prd.md` as the product source of truth and ADRs as the implementation contract source of truth.

## Next implementation steps

1. Add CI and native release automation for Windows and Linux.
2. Harden symbolization rules against additional adversarial formatting and false-negative edge cases.
3. Expand operator docs and scripted workflow examples for pilot use.
4. Install the Rust toolchain in this environment and run the full workspace validation commands.

