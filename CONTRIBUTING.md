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

```powershell
cargo fmt --all
cargo test --workspace
```

## AI-assisted contributions

AI-generated code is welcome, but it should be:

- reviewed by a human before merge,
- tested or otherwise validated,
- aligned to the PRD and ADRs, and
- accompanied by any required doc updates.

