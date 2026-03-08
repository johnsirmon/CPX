# CPX synthetic corpus

This directory contains synthetic support-case fixtures used for safety, regression, and onboarding.

## Rules

- Never place real customer data here.
- Keep fixtures small enough to review quickly.
- Store expected outputs next to the fixture that produced them.
- Update the manifest when adding or changing a corpus case.

## Current status

The PRD requires at least 8 synthetic cases for v1, including adversarial cases. The repository now includes an 8-case baseline that covers the current deterministic rules plus two adversarial punctuation-heavy cases.

## Expected layout

```text
tests\corpus\
  manifest.json
  adversarial-punctuation-case\
  adversarial-resource-punctuation-case\
  canonical-case\
  customer-url-case\
  internal-id-case\
  repeat-email-case\
  subscription-email-case\
  username-ip-case\
    input.txt
    expected-sanitized.txt
    expected-projection.txt
```

