# CPX synthetic corpus

This directory contains synthetic support-case fixtures used for safety, regression, and onboarding.

## Rules

- Never place real customer data here.
- Keep fixtures small enough to review quickly.
- Store expected outputs next to the fixture that produced them.
- Update the manifest when adding or changing a corpus case.

## Current status

The PRD release gate now expects 10 synthetic cases total, including adversarial cases and round-trip assertions. The repository currently includes 10 cases with two adversarial fixtures and round-trip validation through the local vault.

## Drift guard

This corpus is the main repo-drift guard for projection behavior.

- The PRD defines what CPX is supposed to do.
- The ADR locks the `cpx-v1` artifact shape.
- These fixtures catch unexpected projection or safety regressions.
- CI and `.\scripts\validate-local.ps1` treat this corpus test as a merge gate.

If behavior changes intentionally, update the relevant spec and the expected corpus outputs together so the checked-in baseline stays trustworthy.

## Expected layout

```text
tests\corpus\
  manifest.json
  adversarial-punctuation-case\
  adversarial-resource-punctuation-case\
  canonical-case\
  customer-url-case\
  false-positive-control-case\
  internal-id-case\
  mixed-repeat-entity-case\
  repeat-email-case\
  subscription-email-case\
  username-ip-case\
    input.txt
    expected-sanitized.txt
    expected-projection.txt
```

