// Title: Miniforge App Foundation
// Subtitle: parity between the Rust types and the Clojure Malli mirror
// Author: Christopher Lester
// Copyright 2025-2026 Christopher Lester (christopher@miniforge.ai)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Every case in `bindings/parity_cases.json` must decode, or fail to
//! decode, exactly as its `valid` flag says. The Clojure suite runs the
//! same table against the Malli mirror, so the two bindings cannot
//! disagree on a case without one suite failing.

use serde_json::Value;
use workbench_contract::{StateVarRegistry, ViewPlugin, WorkbenchSnapshotV1};

const CASES: &str = include_str!("../../../bindings/parity_cases.json");

fn fixture(tenant: &str, kind: &str) -> Value {
    let text = match (tenant, kind) {
        ("career", "registry") => include_str!("../../../fixtures/career/registry.json"),
        ("career", "snapshot") => include_str!("../../../fixtures/career/snapshot.json"),
        ("miniforge", "registry") => include_str!("../../../fixtures/miniforge/registry.json"),
        ("miniforge", "snapshot") => include_str!("../../../fixtures/miniforge/snapshot.json"),
        ("portfolio", "registry") => include_str!("../../../fixtures/portfolio/registry.json"),
        ("portfolio", "snapshot") => include_str!("../../../fixtures/portfolio/snapshot.json"),
        _ => panic!("no fixture {tenant}/{kind}"),
    };
    serde_json::from_str(text).expect("fixture is JSON")
}

/// The case's document: its inline `document`, or its base fixture with
/// the value at `pointer` set (when the case has `set`) or removed.
fn document(case: &Value) -> Value {
    if let Some(doc) = case.get("document") {
        return doc.clone();
    }
    let mut doc = fixture(
        case["base"].as_str().expect("case names a base"),
        case["type"].as_str().expect("case names a type"),
    );
    let pointer = case["pointer"].as_str().expect("case names a pointer");
    let (parent, key) = pointer.rsplit_once('/').expect("pointer has a key");
    let target = doc.pointer_mut(parent).expect("pointer parent exists");
    match (target, case.get("set")) {
        (Value::Object(map), Some(value)) => {
            map.insert(key.to_owned(), value.clone());
        }
        (Value::Object(map), None) => {
            map.remove(key);
        }
        (Value::Array(items), Some(value)) => {
            items[key.parse::<usize>().expect("array index")] = value.clone();
        }
        (other, _) => panic!("cannot edit {pointer} in {other}"),
    }
    doc
}

fn decodes(kind: &str, doc: Value) -> bool {
    match kind {
        "registry" => serde_json::from_value::<StateVarRegistry>(doc).is_ok(),
        "snapshot" => serde_json::from_value::<WorkbenchSnapshotV1>(doc).is_ok(),
        "view_plugin" => serde_json::from_value::<ViewPlugin>(doc).is_ok(),
        _ => panic!("unknown case type {kind}"),
    }
}

#[test]
fn every_parity_case_decodes_as_declared() {
    let table: Value = serde_json::from_str(CASES).expect("parity table is JSON");
    let cases = table["cases"].as_array().expect("parity table has cases");
    assert!(!cases.is_empty(), "parity table has no cases");
    let mismatches: Vec<&str> = cases
        .iter()
        .filter(|case| {
            let kind = case["type"].as_str().expect("case names a type");
            let valid = case["valid"].as_bool().expect("case states valid");
            decodes(kind, document(case)) != valid
        })
        .map(|case| case["name"].as_str().unwrap_or("<unnamed>"))
        .collect();
    assert!(
        mismatches.is_empty(),
        "the Rust types disagree with the parity table: {mismatches:?}"
    );
}
