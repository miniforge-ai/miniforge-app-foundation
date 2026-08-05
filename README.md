<!--
  Title: Miniforge App Foundation
  Author: Christopher Lester (christopher@miniforge.ai)
  Copyright 2025-2026 Christopher Lester. Licensed under Apache 2.0.
-->

# Miniforge App Foundation

The public seam every Miniforge app and every workbench tenant depends on. Three
small crates, no domain logic, no product coupling — this repository exists so
that an adapter can be written against a stable set of types without pulling in
an application.

```
   your adapter ─┐
 career adapter ─┤
   risk adapter ─┼──▶ workbench-contract ◀── Minibench (the app shell)
miniforge adapter┘        (pure types)         data plane + kernel + UI
                                 │
                                 ▼
                   app-foundation-contracts
                    (transport envelope)
```

## Crates

| Crate | Purpose |
|---|---|
| `miniforge-app-foundation-contracts` | The envelope every app exposes regardless of domain: `SnapshotSignatureV1`, `LicenseValidationResponseV1`, `AppConfigV1` (+ `DistributionV1`), and the schema-version constants |
| `miniforge-app-foundation-data-plane` | The `DataPlaneProvider` trait and `build_router`, which wires the five routes every app serves: `/v1/snapshots/{latest,latest/metadata,:id}`, `/v1/app-config`, `/v1/license/validate` |
| `workbench-contract` | The cross-product state-validation seam: `WorkbenchSnapshotV1`, `StateVariable` / `StateVarRegistry`, `StateEvaluation`, `EvidenceRef`, `ViewPlugin` |

The split matters. `app-foundation-contracts` owns the shapes that are identical
across apps because they describe how an app talks to its data plane.
`workbench-contract` owns the workbench *domain* — what a state variable is, how
an evaluation is shaped, what rides in a snapshot. A `WorkbenchSnapshotV1` is one
product body carried over the foundation's snapshot routes, reusing
`SnapshotSignatureV1` for its envelope rather than re-spelling it.

## Writing an adapter

An adapter lives in **your** repository, reads your product's own data, and emits
a `WorkbenchSnapshotV1`. It depends on `workbench-contract` and nothing else here.
The dependency arrow only points inward: adapters depend on the contract, the
shell depends on the contract, and neither depends on the other.

```toml
[dependencies]
workbench-contract = { git = "https://github.com/miniforge-ai/miniforge-app-foundation.git" }
```

Two things define the shape you must produce:

1. **The types** in `crates/workbench-contract/src/lib.rs`. They are the canonical
   definition; Swift and Clojure consumers mirror them, never the reverse.
2. **The golden fixtures** under `fixtures/<tenant>/`. Each is a registry plus a
   snapshot that round-trips through the Rust types, and they are checked by this
   repository's test suite. Three tenants are included — `career`, `portfolio`,
   `miniforge` — deliberately unalike, because the contract's whole claim is that
   a wildly different product projects into the same snapshot with no contract
   change.

Read `fixtures/miniforge/` first if you want the plainest example.

A registry declares, per state variable, what an evaluation must be able to point
at to be trusted (`evidence_requirements`). That declaration is enforced —
[Minibench](https://github.com/miniforge-ai/minibench)'s `validate` refuses a
`pass` that cites no evidence. Write your registry honestly; a requirement you
cannot meet should be declared as a waiver, not quietly omitted.

## Serving snapshots

If your product also wants to serve its snapshots over the standard routes,
implement `DataPlaneProvider` and hand it to `build_router`. Snapshots cross that
seam as `serde_json::Value`, which is what keeps the router domain-neutral; the
typed contract is re-applied at the consuming edges.

```rust
use miniforge_app_foundation_contracts::{AppConfigV1, LicenseValidationResponseV1};
use miniforge_app_foundation_data_plane::{DataPlaneProvider, build_router};
use serde_json::Value;

struct MyProvider { /* ... */ }

#[async_trait::async_trait]
impl DataPlaneProvider for MyProvider {
    async fn latest_snapshot(&self) -> Result<Value, String> { /* ... */ }
    async fn snapshot_by_id(&self, id: &str) -> Result<Option<Value>, String> { /* ... */ }
    async fn app_config(&self) -> AppConfigV1 { /* ... */ }
    async fn validate_license(&self) -> LicenseValidationResponseV1 { /* ... */ }
}

let app = build_router(MyProvider { /* ... */ });
```

`latest_snapshot_metadata` has a default implementation that derives the common
envelope fields from the full snapshot. Override it if your product has richer
metadata worth serving without the body.

## Other language bindings

Rust is canonical. `bindings/swift/` documents the codegen route for the Swift
shell. A Clojure Malli mirror of the wire shape exists and has not yet moved into
this repository; until it does, an adapter in another language should validate
against the golden fixtures.

## Build

```bash
cargo test --workspace --all-targets
```

No credentials, no submodule, no private dependency. That is the point of this
repository — everything here is public so that everything depending on it can be
built by anyone.

## Consumers

- [Minibench](https://github.com/miniforge-ai/minibench) — the workbench app
  shell: hosts, validates, compares, and renders the snapshots adapters emit.

## License

Apache-2.0. Copyright 2025–2026 Christopher Lester (christopher@miniforge.ai).
See [LICENSE](LICENSE).
