# CPX Product Requirements Document

- Status: Draft
- Version: 0.4
- Last Updated: 2026-03-08
- Owner: John Sirmon
- Canonical product name: CPX
- Working codename: PressureHull
- Intended audience: Product, engineering, security, and design reviewers
- Document rule: This PRD defines what CPX must do and why it matters. It should remain readable by both humans and coding agents. Detailed architecture, language choice, crate layout, and implementation tradeoffs belong in separate design docs or ADRs.

---

## 1. Executive summary

CPX is a local-first tool for preparing support case material for AI-assisted workflows without sending raw customer identifiers to an external model. In v1, CPX will ingest text-based case inputs, replace sensitive values with typed symbols, emit a compact model-safe projection, and locally rehydrate approved output when needed. The product exists to reduce data egress risk and prompt noise at the same time. Because the project itself is expected to be implemented largely with AI models and agents, this PRD also acts as a stable spec-first contract for repo structure, validation, and change control.

## 2. Problem statement

Support engineers and AI-assisted workflows routinely work with logs, notes, metadata, and case attachments that contain customer-identifying information such as names, email addresses, tenant IDs, subscription IDs, resource IDs, hostnames, and URLs. Sending raw case material directly to an AI model creates two immediate problems:

1. Security and compliance risk: raw customer data leaves the trusted local boundary.
2. Efficiency and quality loss: long, repetitive, noisy inputs waste tokens and reduce model signal quality.

Existing approaches are usually incomplete in one of two ways: they rely on manual redaction that is slow and error-prone, or they require heavier data-processing stacks than this project needs for an initial secure workflow tool.

## 3. Product statement and v1 outcome

CPX MUST enable a user to transform one support case into a model-safe artifact without exposing raw sensitive values in the outbound model path. For v1, success means the product can:

- ingest one case worth of text-based support material from files or standard input,
- symbolize detected sensitive values into case-scoped placeholders,
- emit a compact projection that preserves diagnostic meaning for AI reasoning, and
- rehydrate approved results locally using a trusted local vault.

## 4. Scope, goals, and non-goals

### Goals

For v1, CPX MUST:

- make it materially safer to use AI on support case data by keeping raw sensitive values local,
- reduce prompt size on repetitive support inputs while preserving diagnostic usefulness,
- provide a predictable CLI workflow for both interactive and automated use,
- keep the product easy to audit, package, and operate as a local tool, and
- establish a stable enough artifact contract that downstream workflows can integrate without guessing.

### Non-goals

For v1, CPX will NOT:

- act as a general analytics engine, SQL engine, or observability platform,
- provide a hosted service, multi-user system, or networked control plane,
- invoke AI models directly or manage model routing,
- parse arbitrary binary attachments such as images or PDFs without prior text conversion,
- perform cross-case identity stitching or long-term entity resolution,
- claim perfect PII detection or formal compliance certification,
- provide a full GUI or interactive case explorer, or
- autonomously decide whether model output is safe to apply.

## 5. Users and jobs-to-be-done

| User | Primary job-to-be-done | Desired outcome |
|---|---|---|
| Support engineer | When preparing an escalated case for AI review, I want a safe local way to compress and sanitize the case so I can use AI without leaking customer data. | Faster diagnosis with lower copy-paste risk |
| AI workflow developer | When building an automated support workflow, I want a deterministic step that strips sensitive data and emits a stable artifact before each model call. | Reliable automation and easier testing |
| Security-conscious operator | When reviewing AI usage in support workflows, I want evidence that raw customer data stays local and does not appear in logs or outbound payloads. | Clearer trust boundary and auditability |

## 6. Assumptions and constraints

CPX v1 is based on the following assumptions and constraints:

- Users run CPX on a trusted local workstation or runner with access to local files.
- Initial supported environments are Windows and Linux.
- Inputs are text or have already been converted to text before entering CPX.
- The outbound model call is handled by another tool or workflow; CPX prepares input and optionally rehydrates output.
- v1 is optimized for one user processing one case at a time.
- A representative reference corpus will be maintained to measure projection quality, safety, and regressions.
- The implementation process will be heavily AI-assisted, so project requirements, examples, and validation commands must stay explicit, plain-text, and version-controlled.
- Detailed implementation choices are expected to evolve; this document should stay stable if the internals change.

## 7. Core domain terms

| Term | Definition |
|---|---|
| Case | The full set of text inputs that represent one support incident or investigation processed together by CPX |
| Session | One local processing run for a case |
| Symbol | A typed placeholder such as `E1` or `T2` that stands in for a raw sensitive value |
| Vault | Local encrypted storage that holds the symbol-to-raw-value mapping |
| Projection | The model-safe artifact emitted by CPX for downstream AI use |
| Rehydration | Local-only replacement of symbols back to raw values after model output is approved |

## 8. Primary user stories and acceptance criteria

### US1. Create a safe projection from case material

As a support engineer, I want to run CPX against case notes, logs, and metadata so I can produce a model-safe artifact without manual redaction.

Acceptance criteria:

- CPX MUST accept file input and standard input for a single case workflow.
- CPX MUST prevent detected raw sensitive values from appearing in emitted projection output.
- CPX MUST fail with a visible error when it cannot safely complete processing.

### US2. Embed CPX in an automated workflow

As an AI workflow developer, I want a predictable command-line contract so I can place CPX before and after model calls in a scripted pipeline.

Acceptance criteria:

- CPX MUST support machine-friendly standard input, standard output, and exit codes.
- CPX MUST write diagnostics to standard error rather than contaminating projection output.
- CPX MUST expose a stable format version in emitted artifacts.

### US3. Trust the local-only safety boundary

As a security-conscious operator, I want confidence that CPX does not require outbound networking for core operation and does not leak raw values through logs.

Acceptance criteria:

- Core CPX execution MUST not require network access.
- Raw sensitive values MUST be omitted or redacted from logs by default.
- CPX MUST make local artifacts and warnings visible enough for review and troubleshooting.

### US4. Rehydrate approved output locally

As a support engineer, I want to turn approved symbolic output back into actionable real values on my machine so I can use results in the trusted local workflow.

Acceptance criteria:

- Rehydration MUST only occur locally with access to the local vault.
- Rehydration MUST never occur on the outbound model path.
- CPX MUST fail clearly if the required vault data is missing or inaccessible.

## 9. Functional requirements

### 9.1 Input and ingest

CPX MUST:

- accept one or more text inputs that together represent a single case,
- support raw case text, logs, notes, structured metadata, and text-converted attachments,
- normalize encoding and line endings before downstream processing or fail with a clear error, and
- preserve timestamps, ordering, or relative sequence where present and useful.

CPX SHOULD:

- support both batch file processing and pipeline-friendly standard input workflows, and
- preserve enough structure to reconstruct a meaningful case timeline.

### 9.2 Sensitive data detection and symbolization

Before any projection is emitted, CPX MUST replace detected sensitive values with typed symbols.

Default sensitive entity categories for v1:

| Entity type | Symbol prefix | Examples |
|---|---|---|
| Customer name | `C` | `C1`, `C2` |
| Tenant ID | `T` | `T1` |
| Subscription ID | `S` | `S1` |
| Email address | `E` | `E1` |
| Username | `U` | `U1` |
| Hostname | `H` | `H1` |
| IP address | `IP` | `IP1` |
| Resource ID | `R` | `R1` |
| Customer-specific URL | `URL` | `URL1` |
| Internal identifier | `ID` | `ID1` |

Symbol behavior requirements:

- Symbols MUST be stable within one case.
- Symbols SHOULD differ across unrelated cases by default.
- Raw values MUST NOT appear in emitted projection output.
- Raw values MUST only be stored in the local vault.
- Detection uncertainty or unsupported input conditions MUST surface as warnings or errors instead of failing silently.

Detection approach constraints for v1:

- Detection MUST be deterministic: the same input MUST produce the same symbols under the same rules.
- Detection MAY use rule-based matching, pattern matching, or regular expressions.
- Probabilistic or model-based detection is out of scope for v1.
- False negatives are higher severity than false positives, so ambiguous values SHOULD be symbolized rather than passed through.
- The detection rule set MUST be auditable by a human reviewer.

### 9.3 Case compilation and structure preservation

CPX MUST transform ingested inputs into a structured internal representation that preserves diagnostic value for downstream reasoning. The internal representation MUST support:

- symbol tables,
- reusable message templates or similar compression of repeated structure where helpful,
- event or timeline slices for temporal reasoning, and
- non-sensitive aggregates that help summarize repeated failures or patterns.

The implementation MAY use any internal data model. This PRD intentionally does not lock the project to a specific in-memory layout or library choice.

### 9.4 Projection artifact

CPX MUST emit a model-safe projection artifact that contains only the information needed for downstream AI reasoning. Projection output MUST:

- include a format version,
- preserve enough event structure and references for useful reasoning,
- include non-sensitive dictionaries or summaries when they improve comprehension, and
- exclude the symbol-to-raw-value map.

Projection output SHOULD support smaller targeted slices instead of forcing one large blob for every use case.

Illustrative projection example:

```text
CASE case-42
FORMAT cpx-v1
VERSIONS v1=1.31.2
ERRORS e7=0x80070005
EVENTS
 t+00 H1 tm23 v1
 t+03 H1 tm18 e7 R1
 t+05 H1 tm12 S1>S2
```

The example above is illustrative rather than normative. The formal artifact specification belongs in a separate design document once implementation starts.

### 9.5 Local vault and rehydration

CPX MUST:

- store raw sensitive values only in local encrypted storage,
- scope symbol mappings to one case unless a future version explicitly introduces a merge model,
- permit rehydration only in a trusted local path, and
- avoid exposing raw vault content in logs or normal command output.

CPX SHOULD:

- survive normal process restart without losing required case-local mappings when the same trust context is available, and
- make missing-key or unreadable-vault failures explicit to the user.

### 9.6 CLI and workflow contract

The initial workflow contract for v1 is command-line first.

CPX MUST:

- support file-based and standard-input workflows,
- write machine output to standard output and diagnostics to standard error,
- use non-zero exit codes for unsafe, incomplete, or failed processing, and
- provide a stable enough command surface for scripting and automation.

Illustrative command shapes:

```text
cpx ingest <input-path-or-stdin>
cpx project --format cpx-v1 --output projection.txt
cpx rehydrate model-output.txt --output trusted-output.txt
```

Exact flags may evolve, but the product MUST preserve a predictable CLI-first integration path.

Exit code contract for v1:

| Code | Name | Meaning |
|---|---|---|
| `0` | Success | Processing completed safely and produced the requested output |
| `1` | General error | Unexpected internal failure |
| `2` | Safety failure | A raw sensitive value could not be safely handled; outbound-safe output MUST NOT be emitted |
| `3` | Vault error | Required vault data is missing, unreadable, or inaccessible |
| `4` | Input error | Input is missing, unreadable, or unsupported |
| `5` | Format mismatch | Artifact format version is incompatible with the requested operation |

Automation and CI MUST treat exit code `2` as a hard stop.

### 9.7 Configuration, versioning, and observability

CPX MUST:

- version its projection artifact from the first externally shared format,
- document compatibility expectations between artifacts and binary versions,
- provide local-only diagnostics for warnings, skips, and safety-related failures, and
- avoid requiring outbound network calls for core operation.

CPX SHOULD:

- allow local configuration of vault location, logging level, and output behavior, and
- provide summary counts for detected symbols, skipped inputs, and warnings.

### 9.8 AI-assisted delivery contract

Because CPX is expected to be built primarily with AI coding tools and agents, the repository and specification set MUST support spec-driven delivery rather than relying on tribal knowledge.

The project repository MUST:

- keep the PRD, ADRs, examples, and validation commands in version-controlled plain text,
- make build, test, and validation entry points explicit enough for a new coding agent to discover without guesswork,
- keep externally visible contracts versioned and easy to locate, including CLI behavior, artifact format, and sample workflows, and
- separate product requirements from implementation detail so agents can change internals without silently changing product scope.

The project workflow MUST:

- require tests or validation updates for non-trivial generated code changes,
- treat CI or equivalent automated validation as a merge gate for agent-authored changes,
- keep dependencies pinned or otherwise intentionally controlled, and
- preserve a human-review path for security-sensitive, data-handling, or format-breaking changes.

The repository SHOULD:

- prefer small modules, explicit interfaces, and deterministic commands over clever implicit behavior,
- keep examples and fixtures close to the behaviors they specify, and
- make it easy to trace a user-visible behavior back to a requirement, test, or design decision.
- provide one canonical end-to-end example that shows input, projection output, and local rehydration at a small synthetic scale.

## 10. Non-functional requirements

### 10.1 Security and privacy

- Raw customer data MUST remain inside the local trusted boundary.
- CPX MUST NOT automatically rehydrate content during any outbound model path.
- Logs MUST avoid raw sensitive values by default.
- Any persisted vault material MUST be encrypted at rest.

### 10.2 Performance and usability

- CPX SHOULD feel interactive for normal CLI use on a standard developer laptop.
- Projection generation SHOULD favor targeted, compact outputs over oversized prompt blobs.
- The product SHOULD materially reduce token count on repetitive case material without destroying diagnostic signal.

### 10.3 Portability and distribution

- v1 release artifacts MUST be distributed as a single native binary per supported operating system.
- CPX MUST support Windows and Linux in v1.
- Core operation MUST NOT depend on a separately managed service.

### 10.4 Reliability, testing, and maintainability

- CPX MUST be validated against a reference corpus of representative case inputs before release.
- CPX MUST include round-trip tests that verify symbolization plus rehydration behavior.
- The repository SHOULD keep product requirements separate from detailed implementation docs so the PRD remains easy to maintain as the code evolves.
- AI-generated or agent-authored changes SHOULD remain small, reviewable, and backed by automated validation.
- The project SHOULD maintain clear top-level repo guidance such as README, setup instructions, and contribution rules so a fresh agent or human can navigate the codebase safely.

### 10.5 Reference corpus requirements

The reference corpus is a version-controlled set of synthetic test cases used for safety, regression, and release validation. For v1, the corpus MUST:

- include at least 8 synthetic cases that cover the default entity categories in Section 9.2,
- include at least 2 adversarial cases where sensitive values appear in unusual positions such as URLs, broken formatting, or split lines,
- live at a documented repository path,
- include expected assertions such as symbol counts or known-safe output checks for each case, and
- be runnable through a single documented validation command.

The reference corpus MUST NOT contain real customer data.

## 11. Success metrics and measurement

| Metric | Target for v1 | Measurement method |
|---|---|---|
| Projection size reduction | Median token count reduced by at least 50% versus raw case text on the reference corpus | Compare projected artifact to raw baseline across representative cases |
| Projection safety | Zero known raw sensitive values present in emitted projections on the reference corpus | Automated corpus scan and release validation checks |
| Rehydration fidelity | At least 99% exact round-trip restoration for supported symbolic fields | Automated round-trip tests on approved sample outputs |
| Workflow usability | At least one documented file workflow and one documented stdin/stdout workflow work end-to-end | Release validation and example command checks |
| Local-only runtime boundary | Zero required outbound network calls during core end-to-end execution | Offline or network-blocked validation run |
| Packaging simplicity | Single native binary produced for each supported operating system | Release artifact inspection |
| Failure visibility | Unsafe or incomplete processing returns a non-zero exit code and a user-visible diagnostic | Integration tests covering failure paths |
| AI delivery readiness | A fresh coding agent can identify the main spec, validation commands, and example workflows without manual explanation | Repo audit against documented entry points and sample implementation task |
| Onboarding clarity | A new contributor can run one documented synthetic end-to-end example without reading internal chat history | Quickstart validation against the canonical example |

## 12. Milestones and definition of done

### M0. Project foundation

Exit criteria:

- The case concept, reference corpus, and CLI-first workflow are agreed.
- The repository has a minimal scaffolding path for ingest, project, and rehydrate commands.
- The artifact versioning approach is chosen.
- The normative artifact format doc or ADR exists before implementation moves beyond M0.
- Repo entry points for specs, build, test, and validation are documented for both humans and coding agents.

### M1. Safe ingest and symbolization MVP

Exit criteria:

- CPX can ingest representative case inputs.
- Default sensitive entity categories are detected and symbolized.
- Unsafe or unsupported processing paths fail visibly.

### M2. Projection MVP

Exit criteria:

- CPX emits a model-safe projection artifact with versioning.
- The projection preserves enough structure for timeline and failure reasoning.
- Median token reduction is measurable on the reference corpus.

### M3. Local vault and rehydration

Exit criteria:

- Vault-backed local rehydration works for supported fields.
- Round-trip tests pass on the reference corpus.
- Logs and normal output avoid raw sensitive values by default.

### M4. Pilot readiness

Exit criteria:

- Windows and Linux release artifacts are produced.
- Example automation workflows are documented and validated.
- Success metrics and known limitations are reviewed before pilot use.
- Repo guidance is strong enough that AI-assisted contributors can make scoped changes without inventing undocumented patterns.

## 13. Risks and dependencies

| Risk or dependency | Impact | Mitigation |
|---|---|---|
| Missed sensitive-data patterns lead to false negatives | High | Maintain a representative corpus, add regression tests, and make uncertainty visible instead of silent |
| Compression removes too much diagnostic context | High | Measure reasoning quality on the corpus and preserve targeted structural cues |
| Key management or vault access becomes too complex for v1 | Medium | Keep the trust model narrow in v1 and resolve the key strategy early |
| Input variability across support cases causes brittle ingest behavior | Medium | Define supported input contracts early and fail clearly on unsupported content |
| Artifact format churn breaks integrations | Medium | Version the format from the start and document compatibility expectations |
| AI-generated implementation drifts from the spec or introduces inconsistent patterns | High | Keep a spec-first repo structure, require validation gates, and preserve human review for sensitive changes |

## 14. Open questions

- [ ] [Owner: John] What is the v1 key management strategy for the local vault: OS keychain, passphrase, file key, or another local-only approach?
- [ ] [Owner: John] Which input forms are mandatory for pilot readiness: single file, directory, stdin, structured JSON, or all of the above?
- [ ] [Owner: John] Will v1 ship with fixed built-in detection rules only, or is local rule configuration required before pilot use?
- [ ] [Owner: John] What compatibility promise should CPX make for `cpx-v1` artifacts across patch and minor releases?
- [ ] [Owner: John] What minimum reference corpus is required to trust the release gates for safety and projection quality?
- [ ] [Owner: John] What level of human review is mandatory before merging agent-authored changes, especially for vault, symbolization, and artifact format code?

## 15. Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-03-07 | Use CPX as the canonical product name; keep PressureHull as an internal codename for the runtime/reference implementation. | Reduces naming confusion in docs, code, and release artifacts |
| 2026-03-07 | Keep this PRD product-focused and move detailed implementation choices into separate design docs or ADRs. | Makes the PRD more durable as the implementation evolves |
| 2026-03-07 | Treat v1 as local-first, text-first, and single-user. | Keeps the initial scope narrow, testable, and aligned to the core trust boundary |
| 2026-03-07 | Keep external model invocation out of CPX scope. | Preserves a clear product boundary and avoids mixing safety prep with model routing |
| 2026-03-07 | Treat the repo as spec-driven and agent-friendly from day one. | The project is expected to be built largely with AI coding tools, so requirements, validation, and repo conventions must be explicit and machine-readable |
