# ADR 0001: CPX projection artifact format v1

- Status: Accepted
- Date: 2026-03-08
- Related PRD: `prd.md` sections 9.4, 9.6, 9.8, 12

## Context

The PRD requires CPX to emit a versioned, model-safe projection artifact and to lock the normative format before implementation moves beyond M0. This ADR establishes the initial artifact contract so implementation can proceed without reverse-engineering the format from code.

## Decision

For v1, CPX projection artifacts will use a UTF-8 plain-text, line-oriented format.

### Required records

The artifact MUST contain these records:

1. `FORMAT <format-version>`
   - Type: string
   - Constraint: first non-empty line
   - Initial value: `cpx-v1`
2. `CASE <case-id>`
   - Type: string token
   - Constraint: one per artifact
3. `EVENTS`
   - Type: section header
   - Constraint: appears once
4. Event lines below `EVENTS`
   - Type: free-form symbolic event records
   - Constraint: each line represents one symbolic event or slice

### Optional records

The artifact MAY contain these sections when they improve reasoning quality:

- `VERSIONS`
- `ERRORS`
- `SUMMARY`
- other named sections that remain non-sensitive and are documented before release

### Field rules

- The version marker MUST be explicit in the artifact.
- The artifact MUST NOT contain the raw symbol-to-value map.
- The artifact MUST remain safe for downstream model use.
- Non-sensitive helper dictionaries MAY appear if they improve comprehension.
- Format-breaking changes MUST create a new version identifier.

## Minimal example

```text
FORMAT cpx-v1
CASE case-minimal
EVENTS
 t+00 H1 tm1
```

## Expanded example

```text
FORMAT cpx-v1
CASE case-canonical
VERSIONS
 v1=1.31.2
ERRORS
 e7=0x80070005
SUMMARY
 repeated_error_count=2
EVENTS
 t+00 H1 tm23 v1
 t+03 H1 tm18 e7 R1
 t+05 H1 tm12 S1>S2
```

## Consequences

- The CLI and tests can target a stable artifact contract immediately.
- The first projection implementation should produce a small subset of this format rather than inventing an alternative.
- Any future binary or JSON representation must either wrap or supersede this contract with a new ADR and format version.

