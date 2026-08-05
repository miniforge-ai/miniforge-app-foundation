# Swift bindings

The Swift types for the Minibench shell are **generated** from the canonical
Rust types in `crates/workbench-contract/src/lib.rs`, not hand-written. This
keeps the Swift consumer from drifting off the contract.

## Codegen

The tool is not chosen yet — `typeshare` and `serde-reflection` are both
candidates, and `typeshare` is the expected answer. Until that is settled this
directory holds only the integration note.

When the tool lands, the steps are:

1. Annotate the contract structs and enums with `#[typeshare]`.
2. Run typeshare against `crates/workbench-contract` to emit `Generated.swift`.
3. Commit the generated file here; never edit it by hand.
4. The golden fixtures under `fixtures/<tenant>/` are the round-trip test —
   decode each in Swift, re-encode, assert structural equality, exactly as
   `cargo test` does on the Rust side.

## Why generated, not shared

The contract crosses Rust (canonical) → Swift (app shell) → Clojure (adapters,
via a Malli mirror). A shared binary type would force every side into one
language. Generated or mirrored types from a single canonical source, verified
against shared JSON fixtures, is the pattern used across Miniforge — and it is
why the fixtures in this repository are tests rather than samples.
