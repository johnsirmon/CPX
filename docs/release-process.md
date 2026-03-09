# CPX release process

This document explains how CPX release artifacts are produced and what gets published.

## What a release produces

CPX v1 release artifacts are packaged as one native binary per supported operating system.

Current release archives:

- `cpx-windows-x86_64-v0.1.0.zip`
- `cpx-linux-x86_64-v0.1.0.tar.gz`

Each archive contains:

- the CPX binary for that platform
- `README.md`
- `install-and-use.md`
- `VERSION.txt`

The archive filename and the top-level extracted folder both include the release version.

On both supported platforms, extracting the archive yields a single top-level directory named after the release archive, for example `cpx-windows-x86_64-v0.1.0`.

This keeps the operator install path simple without requiring Rust or a source checkout.

## How releases are triggered

The repository release workflow is defined in [release.yml](../.github/workflows/release.yml).

It runs in two modes:

1. `workflow_dispatch`
2. a pushed git tag that matches `v*`

Behavior by trigger:

- `workflow_dispatch` builds and uploads versioned workflow artifacts using the workspace version from `Cargo.toml`
- a pushed `v*` tag builds, packages, uploads workflow artifacts, and publishes versioned GitHub release assets using that tag

Tagged release guardrails:

- the pushed tag must match the workspace version exactly, for example workspace `0.1.0` requires tag `v0.1.0`
- the workflow uses `contents: write` permission so it can attach packaged archives to the GitHub release
- GitHub auto-generates the release notes from the tagged changeset

## What the release workflow validates

Before packaging, the workflow runs:

- `cargo test --workspace`
- `cargo build --release -p cpx-cli`

The main cross-platform validation gate remains [ci.yml](../.github/workflows/ci.yml), which also checks formatting, corpus regression, and CLI help.

## Recommended release steps

1. Run local validation first.
2. Confirm docs are current, especially install and usage guidance.
3. Update the workspace version if needed.
4. Create and push a tag such as `v0.1.0`.
5. Verify that the release workflow publishes both archives.
6. Smoke-test each archive by extracting it and running `cpx --help`.

Versioning rule:

- the Cargo workspace version is the source of truth for the binary version
- release tags should use the matching `v<version>` form, for example `v0.1.0`
- published archive names include that version, for example `cpx-windows-x86_64-v0.1.0.zip`

## Local validation before tagging

On Windows:

```powershell
.\scripts\validate-local.ps1
```

Equivalent manual checks:

```powershell
cargo fmt --all
cargo test --workspace
cargo test -p cpx-core --test corpus corpus_cases_match_expected_outputs
cargo run -p cpx-cli -- --help
```

## Creating a release tag

Example:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

After the tag is pushed, GitHub Actions publishes the release assets automatically.

## Operator-facing install source

For non-technical users, the intended install path is:

1. download the correct release archive
2. extract it
3. follow `install-and-use.md`

That path is simpler and safer than asking end users to install Rust or build from source.
