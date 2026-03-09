# CPX

CPX is a local-first CLI for preparing support case material for AI-assisted workflows
without sending raw customer identifiers to an external model.

CPX ingests text, replaces sensitive values with stable typed symbols, emits a
model-safe projection artifact, stores the raw mappings in a local encrypted vault,
and can later rehydrate approved symbolic output back to the original local values.

## Start here

If your goal is to use CPX rather than develop it, read these in order:

1. [`quickstart.md`](quickstart.md) - shortest end-to-end path
2. [`docs/user-guide.md`](docs/user-guide.md) - concepts, workflow, and safe operating guidance
3. [`docs/cli-reference.md`](docs/cli-reference.md) - commands, flags, examples, and exit codes
4. [`docs/troubleshooting.md`](docs/troubleshooting.md) - common failures and recovery steps

Useful reference material:

- [`tests\corpus\canonical-case\`](tests/corpus/canonical-case/) - synthetic end-to-end example
- [`docs\adr\0001-projection-artifact-format.md`](docs/adr/0001-projection-artifact-format.md) - artifact contract
- [`prd.md`](prd.md) - product source of truth

## What CPX does

CPX is built around a simple workflow:

1. Read local case material from a file or stdin.
2. Detect supported sensitive values and replace them with symbols such as `C1`, `T1`,
   `E1`, `H1`, `R1`, or `URL1`.
3. Emit a projection artifact that is safe to hand to a downstream AI workflow.
4. Store the raw symbol map in a case-local encrypted vault that stays on the trusted side.
5. Rehydrate approved symbolic output locally when you need the original values again.

The core runtime is intentionally local-first and network-free. CPX prepares data for
another model workflow; it does not make the outbound model call for you.

## CLI at a glance

CPX currently exposes three commands:

- `cpx ingest` - validate and summarize an input
- `cpx project` - create a model-safe projection and optional vault
- `cpx rehydrate` - restore symbolic output with a local vault

Typical Windows usage:

```powershell
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"

cpx project .\case.txt --output .\out\projection.txt
cpx rehydrate .\model-output.txt --vault .\out\case-id.cpxvault --output .\out\trusted-output.txt
```

If the `CPX_PASSPHRASE` environment variable is set when you run `cpx project`, CPX
will write a case-local `.cpxvault` file automatically unless you override the path
with `--vault-output`.

## What you should expect on disk

After a normal projection and rehydration workflow, you will usually have:

- a raw local input file such as `case.txt`
- a model-safe projection file such as `projection.txt`
- an encrypted local vault such as `canonical-case.cpxvault`
- a trusted rehydrated output file such as `trusted-output.txt`

The projection file is the one intended for downstream reasoning. The vault file is
part of the trusted local boundary and should be kept local.

## Current implementation status

The repository currently includes:

- normalized ingest for files and stdin
- deterministic rule-based symbolization for the default v1 entity categories
- projection output for the `cpx-v1` artifact format
- encrypted case-local vault storage
- local rehydration of symbolic output
- a corpus-driven validation baseline with ten synthetic cases

The canonical synthetic example lives under `tests\corpus\canonical-case\`.

## How repo drift is prevented

CPX treats repo drift as implementation diverging from the intended design. The guardrails are meant to stay simple:

- `prd.md` defines the intended behavior and scope.
- `docs\adr\0001-projection-artifact-format.md` locks the `cpx-v1` projection contract.
- `tests\corpus\` is the regression baseline; unexpected projection or safety changes should fail there first.
- CI and `.\scripts\validate-local.ps1` run those checks before changes are merged.

If you intentionally change behavior, update the relevant PRD or ADR first, then update the corpus fixtures and tests in the same change.

## Documentation map

### Use CPX

- [`quickstart.md`](quickstart.md)
- [`docs/user-guide.md`](docs/user-guide.md)
- [`docs/cli-reference.md`](docs/cli-reference.md)
- [`docs/troubleshooting.md`](docs/troubleshooting.md)

### Understand the product contract

- [`prd.md`](prd.md)
- [`docs/adr/0001-projection-artifact-format.md`](docs/adr/0001-projection-artifact-format.md)

### Develop or contribute

- [`CONTRIBUTING.md`](CONTRIBUTING.md)
- [`AGENTS.md`](AGENTS.md)
- [`tests/corpus/README.md`](tests/corpus/README.md)

## Source-of-truth order

When instructions conflict, follow this order:

1. `prd.md`
2. ADRs in `docs\adr\`
3. repository instructions in `.github\`
4. `AGENTS.md`
5. inline code comments and local conventions

Do not change user-visible behavior, artifact contracts, or product scope without
updating the relevant source-of-truth document.
