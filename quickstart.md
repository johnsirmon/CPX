# CPX quick start

If you need installation steps first, start with [`docs/install-and-use.md`](docs/install-and-use.md).

This guide shows the fastest safe path through the CPX workflow:

1. project raw case text into a model-safe artifact
2. keep the encrypted vault local
3. rehydrate approved symbolic output back to the original values

The commands below are written for Windows PowerShell and use the repository's
synthetic canonical example so you can test the flow without real customer data.

## Before you start

Preferred path:

- use the released `cpx` binary for your platform
- if `cpx` is not already on `PATH`, replace `cpx` below with the full path to the binary

You will also need:

- a local passphrase for vault encryption
- a text input file
- a working directory where CPX can write output files

This repository already includes a safe synthetic example input at:

`.\tests\corpus\canonical-case\input.txt`

## Step 1: create an output directory and set a passphrase

```powershell
New-Item -ItemType Directory -Force .\out | Out-Null
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"
```

Keep this passphrase local. If you lose it, you will not be able to rehydrate the
vault that CPX creates.

## Step 2: create a projection artifact

```powershell
cpx project .\tests\corpus\canonical-case\input.txt --output .\out\projection.txt
```

Because `CPX_PASSPHRASE` is set, CPX will also create a vault next to the output file.
With the canonical example, you should end up with:

- `.\out\projection.txt`
- `.\out\canonical-case.cpxvault`

CPX writes the projection body to the file you requested and writes the vault path to
standard error.

## Step 3: inspect the projection you can share downstream

```powershell
Get-Content .\out\projection.txt
```

You should see symbolic values instead of raw customer values, for example:

```text
FORMAT cpx-v1
CASE canonical-case
SUMMARY
 line_count=5
 symbolized_entities=5
 customer_name=1
 tenant_id=1
 email_address=1
 hostname=1
 resource_id=1
EVENTS
 t+00 Customer C1 reported that the AMA agent on host H1 failed to read workspace R1.
 t+01 Contact: E1
 t+02 Tenant: T1
 t+03 Timestamp: 2026-03-08T00:00:00Z
 t+04 Error: 0x80070005
```

This is the artifact you would hand to the downstream AI workflow, not the raw input
and not the vault.

## Step 4: simulate a model response

For a quick repo-local test, use the synthetic symbolic response that already exists:

```powershell
Copy-Item .\tests\corpus\canonical-case\expected-sanitized.txt .\out\model-output.txt
```

In a real workflow, `.\out\model-output.txt` would be the symbolic answer returned by
your external AI tool or copied from your AI chat session.

## Step 5: rehydrate the symbolic output locally

```powershell
cpx rehydrate .\out\model-output.txt --vault .\out\canonical-case.cpxvault --output .\out\trusted-output.txt
```

This restores the symbolic values back to the original local values using the vault.

## Step 6: inspect the trusted output

```powershell
Get-Content .\out\trusted-output.txt
```

You should now see the original synthetic customer values again.

## Use the same pattern with your own case

Replace the example file with your own text case:

```powershell
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"
New-Item -ItemType Directory -Force .\out | Out-Null

cpx project .\my-case.txt --output .\out\my-case-projection.txt
cpx rehydrate .\model-answer.txt --vault .\out\my-case.cpxvault --output .\out\my-case-trusted-output.txt
```

If you do not pass `--vault-output`, CPX names the vault from the case id and places it
next to the projection output.

## When to use `cpx ingest`

Use `cpx ingest` when you want to quickly validate an input and see a small summary
before projecting it:

```powershell
cpx ingest .\my-case.txt
```

The command emits:

- `SOURCE`
- `LINES`
- `CHARS`

## Important operating rules

- share the projection, not the vault
- keep the passphrase local
- keep vault files on the trusted side of your workflow
- use only synthetic data in examples, tests, and demos
- treat exit code `2` as a hard stop because it means CPX could not safely produce
  outbound-safe output

## Fallback if you are running from a source checkout

If you do not yet have a released binary, run the same commands by prefixing them with:

```powershell
cargo run -p cpx-cli --
```

Example:

```powershell
cargo run -p cpx-cli -- project .\tests\corpus\canonical-case\input.txt --output .\out\projection.txt
```

For more detail after this guide, continue with [`docs/user-guide.md`](docs/user-guide.md)
and [`docs/cli-reference.md`](docs/cli-reference.md).
