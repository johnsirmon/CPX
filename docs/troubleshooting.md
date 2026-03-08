# CPX troubleshooting

This guide covers common operator-facing problems when using CPX.

## Quick triage

Start with these questions:

1. Did you run `cpx project` or `cpx rehydrate` with the expected input file?
2. Is the passphrase environment variable set and non-empty?
3. Are you using the correct `.cpxvault` file for the same case?
4. Are you rehydrating from a file or from stdin?
5. Did CPX return a non-zero exit code?

## Common problems

### `environment variable 'CPX_PASSPHRASE' was not set`

**What it means**

CPX needed the vault passphrase, but the environment variable was missing.

**Where you usually see it**

- `cpx rehydrate`
- `cpx project` when `--vault-output` is explicitly requested

**What to do**

```powershell
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"
```

If you use a different environment variable name, pass it with `--passphrase-env`.

### `environment variable 'CPX_PASSPHRASE' was empty`

**What it means**

The variable exists, but CPX received an empty string.

**What to do**

Set a non-empty passphrase and rerun the command.

### `vault output requested but passphrase environment variable 'CPX_PASSPHRASE' was not set`

**What it means**

You asked `cpx project` to write a vault, but CPX had no passphrase to encrypt it.

**What to do**

- set the passphrase environment variable before running the command, or
- remove `--vault-output` if you only need the projection

### `--vault is required when rehydrating from stdin`

**What it means**

CPX cannot infer a vault path when the symbolic input comes from stdin.

**What to do**

Pass the vault explicitly:

```powershell
Get-Content -Raw .\model-output.txt | cpx rehydrate - --vault .\out\case-id.cpxvault --output .\out\trusted-output.txt
```

### `could not infer a vault path from the rehydration input; pass --vault explicitly`

**What it means**

You rehydrated from a file without `--vault`, but CPX could not detect the case id from
the symbolic input or could not resolve the default sibling path.

**What to do**

Pass the vault path explicitly with `--vault`.

### Case mismatch or wrong vault file

**What it means**

The symbolic input and the vault do not belong to the same case.

**Typical causes**

- using the vault from a different case
- renaming or mixing output files from multiple runs
- trying to reuse an older vault after regenerating the case under a different case id

**What to do**

- find the vault created by the same `cpx project` run
- keep projection, symbolic answer, and vault files together per case
- rerun `cpx project` if you are unsure which vault matches the projection

### `unsupported format '...'; only 'cpx-v1' is currently accepted`

**What it means**

You passed an unsupported value to `--format`.

**What to do**

Use:

```powershell
cpx project .\case.txt --format cpx-v1
```

or just omit `--format`, because `cpx-v1` is already the default.

### Input file could not be read

**What it means**

CPX could not open the file path you provided.

**What to check**

- the path is correct
- the file exists
- the current working directory is the one you expect
- the file is readable by the current user

### Output file could not be written

**What it means**

CPX could not create or overwrite the requested output path.

**What to check**

- the parent directory exists
- you have write access
- the file is not locked by another process

Create the output directory first if needed:

```powershell
New-Item -ItemType Directory -Force .\out | Out-Null
```

### Safety failure with exit code `2`

**What it means**

CPX detected content it could not safely handle as outbound-safe output.

**What to do**

- treat the run as blocked
- do not send partially processed output downstream
- inspect the input for unsupported or unusual sensitive values
- reduce the case to a smaller synthetic reproduction if you need to debug the behavior

For automation, exit code `2` should always stop the outbound flow.

### Lost passphrase

**What it means**

You still have the vault file, but you no longer have the passphrase used to encrypt it.

**What to do**

There is no supported recovery path for the missing passphrase. You must regenerate the
projection and vault from the original raw case material.

### `cargo` is not recognized

**What it means**

You are using the source-checkout fallback path instead of a released binary, but the Rust
toolchain is not available in the current shell.

**What to do**

Prefer the released `cpx` binary for normal operator use. If you intentionally run from
source, follow the contributor setup in [`..\README.md`](../README.md) and
[`..\CONTRIBUTING.md`](../CONTRIBUTING.md).

## Exit codes at a glance

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | General error |
| `2` | Safety failure |
| `3` | Vault error |
| `4` | Input error |
| `5` | Format mismatch |

## If you still need help

Check these references next:

- [`..\quickstart.md`](../quickstart.md)
- [`user-guide.md`](user-guide.md)
- [`cli-reference.md`](cli-reference.md)
- [`docs\adr\0001-projection-artifact-format.md`](adr/0001-projection-artifact-format.md)
