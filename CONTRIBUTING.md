<!--
  Title: Miniforge App Foundation
  Author: Christopher Lester (christopher@miniforge.ai)
  Copyright 2025-2026 Christopher Lester. Licensed under Apache 2.0.
-->

# Contributing to Miniforge App Foundation

This repository is the seam other things are built on, so the bar for changing it
is higher than for an application, and the useful contribution is usually not a
change here at all — it is an adapter in your own repository. See the README for
how to write one.

## Quick start

```bash
git clone --recurse-submodules https://github.com/miniforge-ai/miniforge-app-foundation.git
cd miniforge-app-foundation
cargo test --workspace --all-targets
```

No credentials and no private dependency. If you cloned without
`--recurse-submodules`, run `git submodule update --init --recursive` — the
`standards/miniforge` submodule holds the engineering rules and is public.

Babashka runs the task and gate definitions in `bb.edn`
(`brew install borkdude/brew/babashka`). The Rust toolchain is pinned in
`rust-toolchain.toml`; `rustup` honours it automatically.

## What changes are easy, and what are not

**Easy:** a new optional field, a clearer doc comment, a fixture that exercises a
shape nobody covered, a bug in the router's error handling.

**Hard, and needs a strong case:** anything that changes an existing type. These
are a published wire contract with consumers outside this organisation, including
closed-source ones that cannot be fixed in the same commit. Adding an optional
field is backward compatible. Changing a field's type, renaming it, or removing
it is not, and the answer there is a new `*_V2` type alongside the existing one,
never an edit in place. The `*_V1` schema-version constants are pinned by tests
so accidental wire drift fails the build.

**Rejected on sight:** anything that makes this repository depend on something
outside it, and any reference to a private repository's name, source path,
namespace, or policy constant — fixture `notes` and `owner` fields included.
This repository sits at the bottom of the dependency graph and several of its
consumers are closed source. Closed products depending on an open seam is fine;
the seam knowing about them is not.

## Fixtures

`fixtures/<tenant>/` is both the round-trip test suite and the reference an
adapter author reads. Three tenants are included and are deliberately unalike,
because the contract's claim is that it is neutral across products. If a change
makes one tenant's fixture awkward to express, treat that as evidence against the
change rather than as a fixture to massage.

## Before you open a pull request

Enable the pre-commit hook once per clone:

```bash
git config core.hooksPath .githooks
```

It runs `bb pre-commit` — `cargo fmt --check`, `cargo clippy -D warnings`, and
`cargo test`. Do not bypass it; fix the cause.

Then:

1. **Audit your diff against `standards/miniforge/`**, starting at
   `standards/miniforge/index.mdc`. The ones that bite here are 001
   stratified-design, 006 named-constants, 008 no-dead-code, 230 Rust style, and
   810 the Apache-2.0 file header.
2. **Write the PR doc** — `docs/pull-requests/YYYY-MM-DD-branch-name.md` (rule
   721). For a contract change, record what you considered and rejected, not just
   what you did.
3. **Add tests with the code** (rule 716), and put the Apache-2.0 header on every
   new file.

## Licensing

Apache-2.0. By contributing you agree that your contributions are licensed under
the same terms.

## Reporting security issues

Do not open a public issue. See [SECURITY.md](SECURITY.md).
