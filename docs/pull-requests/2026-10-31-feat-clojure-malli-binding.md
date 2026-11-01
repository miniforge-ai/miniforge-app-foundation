<!--
  Title: Miniforge App Foundation
  Author: Christopher Lester (christopher@miniforge.ai)
  Copyright 2025-2026 Christopher Lester. Licensed under Apache 2.0.
-->

# feat: publish the Clojure Malli mirror of the workbench types

## Overview

Adds `bindings/clojure/`, a Malli mirror of the workbench wire types
(`ai.miniforge.workbench-contract.schema`), as a git-coordinate Clojure lib.
The README already said this mirror existed but had not moved here yet.

## Motivation

Miniforge's two workbench adapters validate their output against this mirror.
Until now it lived in a private repository. A public clone of miniforge could
not resolve its dependencies, and miniforge CI and release needed a GitHub App
token to fetch it. With the mirror here, miniforge can depend on it over HTTPS
with no credentials.

## Changes in Detail

1. `bindings/clojure/src/ai/miniforge/workbench_contract/schema.clj`: the mirror,
   with field rules corrected to match serde. An `Option<T>` field may be omitted
   or null. A `#[serde(default)]` field (`thresholds`, `consumes`, `entities`,
   `axes`) may be omitted but not null. `u32` and `u64` fields carry their Rust
   bounds, and arrays accept any sequential collection.
2. `bindings/clojure/deps.edn`: `{:paths ["src"]}` plus Malli 0.17.0, the version
   the mirror was already published with.
3. `bindings/clojure/test/.../schema_test.clj`: every registry and snapshot under
   `fixtures/<tenant>/` validates against the mirror; an off-enum status and a
   missing signature do not.
4. `bindings/parity_cases.json`: 31 edge cases, each a fixture edit plus whether
   the Rust types accept it. `crates/workbench-contract/tests/parity.rs` and the
   Clojure suite both run the table, so the bindings cannot disagree on a case
   without one suite failing.
5. `bb test:clojure` runs the Clojure tests. `bb pre-commit` depends on it, and CI runs
   it in a new `clojure binding` job.
6. README: how to depend on the binding, and miniforge listed as a consumer.
7. AGENTS.md: a field added to the Rust types is added to the mirror in the same
   change. Parity edge cases go in the shared table.
8. `Cargo.lock`: async-trait 0.1.91 to 0.1.92, in its own commit. Clippy 1.99 adds
   `double_must_use`, which fires on the attribute 0.1.91 generates for each
   `DataPlaneProvider` method. The gate failed on current stable Rust with no
   source change.

## Testing Plan

1. `bb test:clojure`: 3 tests, 72 assertions, 0 failures. `cargo test -p
   workbench-contract --test parity`: passes.
2. Removing `"pass"` from the mirror's `StateStatus` enum makes 3 assertions fail,
   so the fixture test catches mirror drift.
3. Flipping one case's `valid` flag fails both the Rust and the Clojure suite.
4. `bb pre-commit` runs the existing cargo gate and the new test.

## Deployment Plan

None. Consumers opt in by pinning a SHA of this repository with
`:deps/root "bindings/clojure"`.

## Related Issues/PRs

Miniforge follows with a PR that points its workbench adapters at this binding
and drops the CI and release token steps.

## Checklist

- [x] Apache-2.0 header on every new file
- [x] Tests with the code
- [x] PR doc included
