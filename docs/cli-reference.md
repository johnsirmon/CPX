# CPX CLI reference

This document describes the current CPX command surface for v1.

## Command overview

| Command | Purpose |
| --- | --- |
| `cpx ingest` | Read an input and emit a small summary |
| `cpx project` | Ingest text, symbolize supported values, emit a projection artifact, and optionally write a vault |
| `cpx rehydrate` | Rehydrate symbolic output with a matching local vault |

If you run `cpx` with no subcommand, CPX prints help and exits successfully.

## Input and output rules

- If the input path is omitted, CPX reads from stdin.
- Passing `-` as the input path also means stdin.
- If `--output` is omitted, CPX writes machine output to stdout.
- Diagnostics such as vault location messages go to stderr.

## `cpx ingest`

### Synopsis

```text
cpx ingest [input] [--output <path>]
```

### Purpose

Use `cpx ingest` to validate that CPX can read the input and to get a quick summary
before projecting it.

### Output

The command emits three lines:

- `SOURCE <name>`
- `LINES <count>`
- `CHARS <count>`

### Examples

From a file:

```powershell
cpx ingest .\case.txt
```

From stdin:

```powershell
Get-Content -Raw .\case.txt | cpx ingest -
```

Write the summary to a file:

```powershell
cpx ingest .\case.txt --output .\out\ingest-summary.txt
```

## `cpx project`

### Synopsis

```text
cpx project [input] [--output <path>] [--format cpx-v1] [--vault-output <path>] [--passphrase-env <name>]
```

### Purpose

`cpx project` runs the main outbound-safe preparation flow:

1. read input
2. ingest and normalize it
3. symbolize supported sensitive values
4. emit a `cpx-v1` projection artifact
5. optionally store a local encrypted vault

### Flags

| Flag | Meaning |
| --- | --- |
| `--output <path>` | Write the projection to a file instead of stdout |
| `--format <value>` | Artifact format version; current accepted value is `cpx-v1` |
| `--vault-output <path>` | Explicit path for the encrypted vault file |
| `--passphrase-env <name>` | Environment variable to read for the vault passphrase; default is `CPX_PASSPHRASE` |

### Vault behavior

`cpx project` handles vault output this way:

- no passphrase env var and no `--vault-output` -> emit projection only
- passphrase env var set and no `--vault-output` -> emit projection and write a default vault next to the projection output
- `--vault-output` set but passphrase env var missing -> fail with a vault error
- passphrase env var present but empty -> fail with a vault error

When CPX chooses the default path, it writes:

`<output-directory>\<case-id>.cpxvault`

If `--output` is omitted, CPX uses the current working directory for the default vault path.

### Examples

Projection to stdout:

```powershell
cpx project .\case.txt
```

Projection to a file with the default passphrase env var:

```powershell
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"
cpx project .\case.txt --output .\out\projection.txt
```

Projection with an explicit vault path:

```powershell
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"
cpx project .\case.txt --output .\out\projection.txt --vault-output .\secure\case-a.cpxvault
```

Projection from stdin:

```powershell
Get-Content -Raw .\case.txt | cpx project - --output .\out\projection.txt
```

## `cpx rehydrate`

### Synopsis

```text
cpx rehydrate [input] [--output <path>] [--vault <path>] [--passphrase-env <name>]
```

### Purpose

Use `cpx rehydrate` after you receive an approved symbolic response from the downstream
AI workflow and want to restore the original values locally.

### Flags

| Flag | Meaning |
| --- | --- |
| `--output <path>` | Write the rehydrated result to a file instead of stdout |
| `--vault <path>` | Explicit path to the matching `.cpxvault` file |
| `--passphrase-env <name>` | Environment variable containing the vault passphrase; default is `CPX_PASSPHRASE` |

### Vault lookup behavior

- if `--vault` is passed, CPX uses that path
- if input comes from stdin and `--vault` is omitted, CPX fails because it cannot infer the path
- if input comes from a file and `--vault` is omitted, CPX tries to detect the case id from
  the input and look for a sibling `<case-id>.cpxvault`

### Examples

Rehydrate from a file with an explicit vault:

```powershell
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"
cpx rehydrate .\out\model-output.txt --vault .\out\canonical-case.cpxvault --output .\out\trusted-output.txt
```

Rehydrate from stdin:

```powershell
Get-Content -Raw .\out\model-output.txt | cpx rehydrate - --vault .\out\canonical-case.cpxvault --output .\out\trusted-output.txt
```

Rehydrate with an inferred vault path when the symbolic input still includes a case id:

```powershell
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"
cpx rehydrate .\out\model-output-with-case.txt --output .\out\trusted-output.txt
```

The inferred-path flow works only when the symbolic input contains a detectable case id
and the vault sits next to that input file with the expected `<case-id>.cpxvault` name.
For the most predictable operator workflow, pass `--vault` explicitly.

## Exit codes

CPX uses the v1 exit-code contract below:

| Code | Name | Meaning |
| --- | --- | --- |
| `0` | Success | Processing completed safely and produced the requested output |
| `1` | General error | Unexpected internal failure |
| `2` | Safety failure | A raw sensitive value could not be safely handled; outbound-safe output must not be emitted |
| `3` | Vault error | Required vault data is missing, unreadable, mismatched, or inaccessible |
| `4` | Input error | Input is missing, unreadable, empty, or unsupported |
| `5` | Format mismatch | Artifact format version is incompatible with the requested operation |

Automation should treat exit code `2` as a hard stop.

## Related docs

- [`..\quickstart.md`](../quickstart.md)
- [`user-guide.md`](user-guide.md)
- [`troubleshooting.md`](troubleshooting.md)
