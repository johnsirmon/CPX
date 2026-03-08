# CPX

CPX is a local-first CLI for preparing support case material for AI-assisted workflows without sending raw customer identifiers to an external model.

This repository is initialized from `prd.md` and is intentionally scaffolded for AI-assisted implementation. The current focus is M0 foundation work: lock the artifact contract, establish the repo structure, and make the implementation path explicit for both humans and coding agents.

## Current status

- Product requirements: `prd.md`
- Artifact format ADR: `docs\adr\0001-projection-artifact-format.md`
- Agent guidance: `AGENTS.md`
- Contribution rules: `CONTRIBUTING.md`
- Starter corpus path: `tests\corpus\`
- Canonical synthetic example: `tests\corpus\canonical-case\`

The codebase now includes an initial M1 slice: normalized ingest, deterministic rule-based symbolization for the default v1 entity categories, projection output for the `cpx-v1` text artifact, and a corpus-driven validation baseline with eight synthetic cases. Vault-backed rehydration is still ahead.

## Planned workflow

Once the Rust toolchain is installed:

```powershell
cargo fmt --all
cargo test --workspace
cargo run -p cpx-cli -- --help
```

Expected CLI shape for v1:

```text
cpx ingest <input-path-or-stdin>
cpx project --format cpx-v1 --output projection.txt
cpx rehydrate model-output.txt --output trusted-output.txt
```

## Repository layout

```text
crates\
  cpx-cli\     Binary entry point for the CPX CLI
  cpx-core\    Core library surfaces for ingest, symbolize, project, rehydrate, and vault
docs\
  adr\         Normative architecture and contract decisions
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

1. Implement vault-backed local rehydration.
2. Add automated round-trip tests for approved symbolic output.
3. Harden symbolization rules against additional adversarial formatting.
4. Install the Rust toolchain in this environment and run the full workspace validation commands.

