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
   unchanged except for the Apache-2.0 header and one comment that now names
   `miniforge-app-foundation-contracts` as the source of `SnapshotSignatureV1`.
2. `bindings/clojure/deps.edn`: `{:paths ["src"]}` plus Malli 0.17.0, the version
   the mirror was already published with.
3. `bindings/clojure/test/.../schema_test.clj`: every registry and snapshot under
   `fixtures/<tenant>/` validates against the mirror; an off-enum status and a
   missing signature do not.
4. `bb test:clojure` runs those tests. `bb pre-commit` depends on it, and CI runs
   it in a new `clojure binding` job.
5. README: how to depend on the binding, and miniforge listed as a consumer.
6. AGENTS.md: a field added to the Rust types is added to the mirror in the same
   change.
7. `Cargo.lock`: async-trait 0.1.91 to 0.1.92, in its own commit. Clippy 1.99 adds
   `double_must_use`, which fires on the attribute 0.1.91 generates for each
   `DataPlaneProvider` method. The gate failed on current stable Rust with no
   source change.

## Testing Plan

1. `bb test:clojure`: 2 tests, 9 assertions, 0 failures.
2. Removing `"pass"` from the mirror's `StateStatus` enum makes 3 assertions fail,
   so the fixture test catches mirror drift.
3. `bb pre-commit` runs the existing cargo gate and the new test.

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
