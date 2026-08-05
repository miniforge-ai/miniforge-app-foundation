<!--
  Title: Miniforge App Foundation
  Author: Christopher Lester (christopher@miniforge.ai)
  Copyright 2025-2026 Christopher Lester. Licensed under Apache 2.0.
-->

# CLAUDE.md

This repository vendors the Miniforge engineering rules as a git submodule at
`standards/miniforge/`. Load them before any task. A fresh clone needs
`git submodule update --init --recursive`; the submodule is public, so this
works without credentials.

## Entry points

1. [`standards/miniforge/index.mdc`](standards/miniforge/index.mdc) — Miniforge engineering rules (load first)
2. [README.md](README.md) — what the crates are, how to write an adapter
3. [AGENTS.md](AGENTS.md) — the boundary rules that decide most changes

## What this repo is

The public seam: `miniforge-app-foundation-contracts` (app envelope),
`miniforge-app-foundation-data-plane` (the five-route axum router), and
`workbench-contract` (the workbench state-validation types). No domain logic.

## Conventions (the ones that bite)

- **Apache-2.0, open source** — every file carries the Apache-2.0 header, and
  `license = "Apache-2.0"`. Rule 810 applies.
- **Nothing outside may leak in.** Private product repos depend on this one, not
  the other way round. No private repo name, source path, namespace, or policy
  constant belongs here — fixture `notes` and `owner` fields included. That is
  a review-blocking defect, not a nit.
- **The types are a published wire contract.** Consumers exist outside this
  organisation. Add optional fields; do not change or remove existing ones. A
  breaking change is a new `*_V2`, not an edit in place.
- **Rust** per `languages/rust` (230): edition 2024, `unsafe_code = "forbid"`,
  clippy `all = "deny"`, user-facing text in `strings.rs`.
- **Fixtures are tests.** `fixtures/<tenant>/` is round-tripped by the suite and
  read by adapter authors. Keep the three tenants unalike.
- **Babashka over shell** (740): tasks live in `bb.edn`, not `scripts/*.sh`.
- **Pre-commit gate** is `bb pre-commit`. Enable with
  `git config core.hooksPath .githooks`. Never bypass — fix the cause.
- **PR docs** (721): every feature branch gets `docs/pull-requests/YYYY-MM-DD-branch.md`.

## Before pushing any PR

Run a **standards gap analysis**: audit the diff against the vendored
`standards/miniforge/` rules and fix the gaps *before* push — not after a
reviewer finds them.
