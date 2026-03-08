# CPX user guide

CPX helps you prepare support case material for AI-assisted workflows without exposing
raw customer identifiers in the outbound artifact.

This guide explains the workflow, the trust boundary, the artifact you create, and the
main operating rules for using CPX safely.

## Core concepts

| Term | Meaning |
| --- | --- |
| Raw case material | The original text you keep on the trusted local side |
| Projection artifact | The model-safe text output that contains symbolic values instead of raw identifiers |
| Symbol | A typed placeholder such as `C1`, `T1`, `E1`, `H1`, or `R1` |
| Vault | The encrypted local file that stores the raw symbol mappings for one case |
| Rehydration | The local step that restores symbolic output back to the original values |
| Case ID | The stable identifier used to name the case and match projection output to its vault |

## The CPX trust boundary

CPX separates the workflow into two sides:

- **trusted local side**: your raw case input, your passphrase, and your `.cpxvault` file
- **downstream reasoning side**: the projection artifact you can hand to an external AI workflow

The vault is intentionally local-only. CPX does not embed the raw symbol map in the
projection artifact, and the core runtime does not make outbound network calls.

## Typical workflow

1. Start with a text file or stdin containing one case worth of material.
2. Run `cpx project` to create a symbolic projection.
3. Keep the generated `.cpxvault` file local.
4. Send only the projection artifact to the downstream AI workflow.
5. Save the symbolic answer from that workflow.
6. Run `cpx rehydrate` locally with the matching vault.

The repository's canonical synthetic example under `tests\corpus\canonical-case\`
shows this flow end to end.

## What CPX detects and symbolizes

CPX v1 is designed around deterministic typed symbols. The default entity kinds are:

| Kind | Symbol prefix | Example meaning |
| --- | --- | --- |
| Customer name | `C` | customer or organization name |
| Tenant ID | `T` | labeled tenant UUID |
| Subscription ID | `S` | labeled subscription UUID |
| Email address | `E` | mailbox or contact address |
| Username | `U` | username or account name |
| Hostname | `H` | machine or service hostname |
| IPv4 address | `IP` | IPv4 endpoint |
| Resource ID | `R` | Azure-style resource identifier |
| URL | `URL` | web endpoint |
| Internal or generic ID | `ID` | generic identifier that should not stay raw |

Important behavior:

- the same raw value of the same kind reuses the same symbol within one case
- symbolization is deterministic within a case
- if CPX detects unsafe leftover content after symbolization, it fails instead of emitting
  a success-shaped outbound artifact

## Reading a projection artifact

CPX v1 emits a line-oriented text artifact. A typical artifact looks like this:

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

What the main sections mean:

- `FORMAT` - the artifact format version; v1 is `cpx-v1`
- `CASE` - the case id used to bind related files together
- `SUMMARY` - small non-sensitive counters or helpers
- `EVENTS` - one symbolic event line per input line in the current implementation

Timestamps and error codes in the example stay literal because they are not part of the
default symbolized entity set.

For the normative artifact contract, see
[`docs\adr\0001-projection-artifact-format.md`](adr/0001-projection-artifact-format.md).

## Vault behavior

The `.cpxvault` file is part of the trusted local boundary.

During `cpx project`:

- if the passphrase environment variable is not set, CPX can still emit the projection,
  but it does not write a vault unless you explicitly request one and provide a passphrase
- if the passphrase environment variable is set, CPX writes a vault by default next to the
  projection output unless you override the path with `--vault-output`

During `cpx rehydrate`:

- CPX needs the vault file
- CPX needs the same passphrase environment variable used to encrypt the vault
- CPX rejects mismatched cases and unsupported format markers

If you lose the passphrase, you lose the ability to open that vault.

## Working with files and stdin

CPX supports both file-based and stdin workflows.

### File-based workflow

This is the easiest path for repeatable operator use:

```powershell
$env:CPX_PASSPHRASE = "replace-with-a-local-passphrase"
cpx project .\case.txt --output .\out\projection.txt
cpx rehydrate .\model-output.txt --vault .\out\case-id.cpxvault --output .\out\trusted-output.txt
```

### Stdin workflow

Use stdin when CPX is part of a larger shell or automation pipeline:

```powershell
Get-Content -Raw .\case.txt | cpx project - --output .\out\projection.txt
Get-Content -Raw .\model-output.txt | cpx rehydrate - --vault .\out\case-id.cpxvault --output .\out\trusted-output.txt
```

Notes:

- omitting the input path also means stdin
- when rehydrating from stdin, `--vault` is required because CPX cannot infer the vault path
- if you rehydrate from a file and omit `--vault`, CPX tries to infer the case id from the
  input contents and looks for a sibling `<case-id>.cpxvault`

## Safe operating practices

- Use synthetic data in demos and examples.
- Share the projection, not the raw input and not the vault.
- Set passphrases through a local secret-management workflow when possible.
- Keep vault files local to the trusted environment.
- Treat exit code `2` as a hard stop for automation.
- Keep projection artifacts and trusted rehydrated outputs separate in your storage layout.

## What CPX does not do in v1

CPX v1 does not:

- make the outbound model call for you
- rehydrate content automatically during an outbound workflow
- parse arbitrary binary attachments such as images or PDFs without prior text conversion
- replace the need to review the rehydrated result on the trusted side

## Where to go next

- [`..\quickstart.md`](../quickstart.md) - first run
- [`cli-reference.md`](cli-reference.md) - full command and flag reference
- [`troubleshooting.md`](troubleshooting.md) - common operator issues
