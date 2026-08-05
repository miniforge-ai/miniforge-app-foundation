<!--
  Title: Miniforge App Foundation
  Author: Christopher Lester (christopher@miniforge.ai)
  Copyright 2025-2026 Christopher Lester. Licensed under Apache 2.0.
-->

# AGENTS.md

`miniforge-app-foundation` — the public seam. Three crates of types and one
router, depended on by every Miniforge app and every workbench tenant adapter.

## What this repo is (and is not)

- **Is:** the contract. The domain-neutral app envelope, the five-route data
  plane, and the workbench state-validation types.
- **Is not:** an application, and not any product's adapter. Nothing here reads
  a product's data or knows what a product measures.

## The rule that decides most changes

**Nothing in this repository may depend on anything outside it.** It is the
bottom of the dependency graph: apps and adapters depend on it, it depends on
no one. A change that needs to know about a specific product is a change that
belongs in that product's adapter.

This is also why it is Apache-2.0 and public while several of its consumers are
proprietary and private. That direction is fine and deliberate — closed products
may depend on an open seam. The reverse must never happen: no private repo's
name, source path, namespace, or policy constant belongs in this repository,
including in fixture `notes` and `owner` fields.

## Compatibility

The types here are a published wire contract with consumers outside this
organisation. Adding an optional field is cheap; changing or removing one is
not. A breaking change needs a new `*_V2` alongside the old, not an edit in
place, and the `*_V1` schema-version constants are pinned by tests so a typo
surfaces as a failure rather than silent wire drift.

## Fixtures are the contract test

`fixtures/<tenant>/` holds a registry and a snapshot per tenant, round-tripped
through the Rust types by the test suite. They are also the reference an adapter
author reads. Three deliberately unalike tenants are included because the
contract's claim is neutrality — if a change makes one tenant's fixture
awkward, the change is probably wrong.

## Standards

Follows the Miniforge engineering standards at `standards/miniforge/` (public
submodule); load `standards/miniforge/index.mdc` first. Relevant: 001
stratified-design, 006 named-constants, 008 no-dead-code, 230 Rust style
(user-facing text in `strings.rs`), 716 tests-with-code, 810 the Apache-2.0
header.
