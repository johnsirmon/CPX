# CPX synthetic corpus

This directory contains synthetic support-case fixtures used for safety, regression, and onboarding.

## Rules

- Never place real customer data here.
- Keep fixtures small enough to review quickly.
- Store expected outputs next to the fixture that produced them.
- Update the manifest when adding or changing a corpus case.

## Current status

The PRD requires at least 8 synthetic cases for v1, including adversarial cases. This scaffold includes one canonical starter case so implementation can begin with a shared example and directory shape.

## Expected layout

```text
tests\corpus\
  manifest.json
  canonical-case\
    input.txt
    expected-sanitized.txt
    expected-projection.txt
    expected-rehydrated.txt
```

