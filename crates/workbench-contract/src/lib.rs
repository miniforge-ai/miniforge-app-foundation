// Title: Workbench Contract
// Subtitle: state-variable / evaluation / snapshot boundary types
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

//! The Minibench state-validation seam.
//!
//! Every product that wants to appear in the workbench (portfolio
//! risk, career intelligence, miniforge governance, future products)
//! emits a [`WorkbenchSnapshotV1`] from its own domain data. The
//! generic kernel and the Swift shell in `Minibench` consume only the
//! types in this crate — they never link a product's domain crate.
//!
//! The split mirrors `miniforge-app-foundation`: that crate owns the
//! domain-NEUTRAL transport envelope (signature, license, app-config,
//! the five-route data plane); this crate owns the workbench DOMAIN —
//! what a state variable is, how an evaluation is shaped, what rides
//! in a snapshot. A [`WorkbenchSnapshotV1`] is just one product body
//! carried over the foundation's `/v1/snapshots/*` routes, reusing
//! [`SnapshotSignatureV1`] for its envelope signature.
//!
//! Schema versions are pinned by the `*_V1` string constants; the wire
//! is JSON. The golden fixtures under `fixtures/<tenant>/` are the
//! cross-language contract test: the Rust types here, the Swift types
//! (typeshare), and the Clojure Malli mirror must all round-trip them.

use std::collections::BTreeMap;

use miniforge_app_foundation_contracts::SnapshotSignatureV1;
use serde::{Deserialize, Serialize};
use serde_json::Value;

//------------------------------------------------------------------------------
// Schema version pins

/// `schema_version` literal for a [`WorkbenchSnapshotV1`] body.
pub const WORKBENCH_SNAPSHOT_V1: &str = "workbench_snapshot/v1";
/// `schema_version` literal for a [`StateVarRegistry`].
pub const STATE_VAR_REGISTRY_V1: &str = "state_var_registry/v1";

//------------------------------------------------------------------------------
// Enumerations (closed — universal across every tenant)

/// Evaluation outcome. Closed because the kernel's gate-effect
/// resolution and the shell's rendering switch over exactly these.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StateStatus {
    Pass,
    Warn,
    Fail,
    Blocked,
    NotApplicable,
    Unknown,
}

/// What a state variable measures. Drives evaluator selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StateVarKind {
    Presence,
    Integrity,
    Sufficiency,
    Alignment,
    Compliance,
    Quality,
    Regression,
    Budget,
    Privacy,
}

/// Underlying value domain of a state variable's measured value.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    Boolean,
    Number,
    String,
    Enum,
    Object,
    Array,
}

/// Registry lifecycle of a state variable.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Draft,
    Active,
    Deprecated,
    Experimental,
}

/// View-plugin tier. `generic` views render any registry; `primitive`
/// views compose the shared Swift kit; `bespoke` views are
/// product-authored and read named entity-bag namespaces.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewTier {
    Generic,
    Primitive,
    Bespoke,
}

//------------------------------------------------------------------------------
// Product identity
//
// `product` is an open string, not an enum: a new tenant must NOT
// require a contract bump. Known values are pinned as constants so
// producers and the kernel compare against a constant rather than
// re-spelling the literal.

/// Portfolio risk / backtesting tenant.
pub const PRODUCT_PORTFOLIO: &str = "portfolio";
/// Career intelligence tenant.
pub const PRODUCT_CAREER: &str = "career";
/// Miniforge governed-SDLC / ETL tenant.
pub const PRODUCT_MINIFORGE: &str = "miniforge";

//------------------------------------------------------------------------------
// Evidence

/// A character/offset span into an artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceSpan {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

/// A pointer from an evaluation to the evidence that backs it. Lean
/// superset of the tenant evidence shapes (career `EvidenceRef`,
/// miniforge evidence-bundle refs, portfolio source-provenance).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRef {
    pub id: String,
    pub source_role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<EvidenceSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<String>,
    /// RFC 3339 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

//------------------------------------------------------------------------------
// State variable registry

/// What an evaluation must be able to point at to be trusted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRequirements {
    pub required_refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_include_hash: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_include_source_role: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness_sla_hours: Option<u64>,
}

/// A single product-owned thing the workbench validates. The registry
/// lives in the PRODUCT repo; this is its wire shape so the kernel can
/// load it and the shell can render its detail page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateVariable {
    /// Dotted, product-qualified id, e.g. `portfolio.lens.validation_readiness`.
    pub id: String,
    pub version: String,
    pub product: String,
    pub area: String,
    pub kind: StateVarKind,
    pub description: String,
    pub value_type: ValueType,
    /// Status-band cutoffs, e.g. `{"pass": 0.85, "warn": 0.65}`.
    #[serde(default)]
    pub thresholds: BTreeMap<String, f64>,
    pub evidence_requirements: EvidenceRequirements,
    pub score_components: Vec<String>,
    /// Status → gate/review effect, e.g. `{"fail": "blocks_transition"}`.
    pub gate_effects: BTreeMap<String, String>,
    pub lifecycle: Lifecycle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// A product's registry of state variables. Fetched once per tenant;
/// a [`WorkbenchSnapshotV1`] references it by [`RegistryRef`] rather
/// than carrying it inline every run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateVarRegistry {
    pub schema_version: String,
    pub registry_id: String,
    pub version: String,
    pub product: String,
    pub state_vars: Vec<StateVariable>,
}

/// Reference from a snapshot to the registry that produced its
/// evaluations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryRef {
    pub registry_id: String,
    pub version: String,
}

//------------------------------------------------------------------------------
// Evaluation

/// One observation produced while evaluating a state variable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// The scored, evidence-backed result of evaluating one state
/// variable against one snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateEvaluation {
    pub state_var_id: String,
    pub product: String,
    pub status: StateStatus,
    /// The measured value, shape per the state var's `value_type`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    pub score: f64,
    pub confidence: f64,
    /// Per-component sub-scores keyed by the var's `score_components`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_components: Option<BTreeMap<String, f64>>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub findings: Vec<Finding>,
    /// Resolved gate/review effect for this status. The registry owns
    /// the status→effect mapping; the evaluator copies the resolved
    /// effect here so the shell needn't re-resolve.
    pub gate_effect: String,
    /// Optional delta vs a regression baseline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regression: Option<Value>,
    /// RFC 3339 timestamp.
    pub evaluated_at: String,
}

//------------------------------------------------------------------------------
// View plugin

/// Declaration a `Minibench` view plugin ships so the shell knows what
/// to mount and which entity-bag namespaces to hand it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewPlugin {
    pub id: String,
    pub product: String,
    pub tier: ViewTier,
    /// Entity-bag namespaces this view reads (Tier-3 bespoke only).
    #[serde(default)]
    pub consumes: Vec<String>,
    pub title: String,
}

//------------------------------------------------------------------------------
// Run variant — the configuration that produced a snapshot
//
// The workbench's primary job is comparing PERMUTATIONS: the same
// logical task run under different workflows, prompts, agent models, or
// extraction methods (mechanical vs semantic). A snapshot tags itself
// with the variant that produced it; snapshots sharing an
// `experiment_id` form one comparison set the kernel can lay out as a
// run matrix.

/// A versioned reference to a configured artifact (a workflow, a prompt).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VariantRef {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// The configuration a run was produced under. `label` names the column
/// in the comparison matrix; the typed axes are the common ones, `axes`
/// holds any others.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunVariant {
    /// Logical task being permuted. Runs sharing this id are comparable.
    pub experiment_id: String,
    /// Human label for this variant, e.g. "opus+semantic".
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<VariantRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<VariantRef>,
    /// Agent model id, e.g. "claude-opus-4-8".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Extraction strategy, e.g. "mechanical" or "semantic".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Arbitrary additional comparison axes, label → value.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub axes: BTreeMap<String, String>,
}

//------------------------------------------------------------------------------
// Snapshot
//
// The product body carried over the foundation's `/v1/snapshots/*`
// routes. Envelope head (`schema_version`, `snapshot_id`,
// `generated_at`, `signature`) matches the foundation convention so
// `latest_snapshot_metadata`'s default derivation works unchanged.

/// One run's worth of workbench state for a single tenant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkbenchSnapshotV1 {
    /// Always [`WORKBENCH_SNAPSHOT_V1`].
    pub schema_version: String,
    pub snapshot_id: String,
    /// RFC 3339 timestamp.
    pub generated_at: String,
    /// Tenant id — `PRODUCT_*` constant or a future product's string.
    pub product: String,
    pub run_id: String,
    /// The configuration this run was produced under. Snapshots sharing
    /// `variant.experiment_id` are comparable in the run matrix. Absent
    /// for one-off runs that aren't part of an experiment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<RunVariant>,
    /// Which registry the evaluations were scored against.
    pub registry_ref: RegistryRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hashes: Option<Vec<String>>,
    pub evaluations: Vec<StateEvaluation>,
    /// Product-namespaced entity bag for Tier-3 bespoke views. The
    /// generic kernel ignores it; a bespoke Swift view reads its
    /// declared namespaces (`career.lens_report`, `portfolio.market_state`,
    /// `miniforge.workflow_run`, ...).
    #[serde(default)]
    pub entities: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    /// Reused verbatim from `miniforge-app-foundation-contracts`.
    pub signature: SnapshotSignatureV1,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(json: &str) -> T
    where
        T: serde::de::DeserializeOwned + serde::Serialize + PartialEq + std::fmt::Debug,
    {
        let decoded: T = serde_json::from_str(json).expect("decode fixture");
        let reencoded = serde_json::to_string(&decoded).expect("re-encode");
        let redecoded: T = serde_json::from_str(&reencoded).expect("decode re-encoded");
        assert_eq!(decoded, redecoded, "value must survive a JSON round trip");
        decoded
    }

    #[test]
    fn schema_version_constants_are_stable() {
        assert_eq!(WORKBENCH_SNAPSHOT_V1, "workbench_snapshot/v1");
        assert_eq!(STATE_VAR_REGISTRY_V1, "state_var_registry/v1");
    }

    #[test]
    fn portfolio_fixtures_round_trip() {
        let reg: StateVarRegistry =
            round_trip(include_str!("../../../fixtures/portfolio/registry.json"));
        assert_eq!(reg.product, PRODUCT_PORTFOLIO);
        assert_eq!(reg.schema_version, STATE_VAR_REGISTRY_V1);

        let snap: WorkbenchSnapshotV1 =
            round_trip(include_str!("../../../fixtures/portfolio/snapshot.json"));
        assert_eq!(snap.schema_version, WORKBENCH_SNAPSHOT_V1);
        assert_eq!(snap.product, PRODUCT_PORTFOLIO);
        assert_eq!(snap.registry_ref.registry_id, reg.registry_id);
        // Every evaluation must name a state var that exists in the registry.
        for ev in &snap.evaluations {
            assert!(
                reg.state_vars.iter().any(|sv| sv.id == ev.state_var_id),
                "evaluation references unknown state var: {}",
                ev.state_var_id
            );
        }
    }

    #[test]
    fn career_fixtures_round_trip() {
        let reg: StateVarRegistry =
            round_trip(include_str!("../../../fixtures/career/registry.json"));
        assert_eq!(reg.product, PRODUCT_CAREER);
        let snap: WorkbenchSnapshotV1 =
            round_trip(include_str!("../../../fixtures/career/snapshot.json"));
        assert_eq!(snap.product, PRODUCT_CAREER);
        assert_eq!(snap.registry_ref.registry_id, reg.registry_id);
    }

    #[test]
    fn miniforge_fixtures_round_trip() {
        let reg: StateVarRegistry =
            round_trip(include_str!("../../../fixtures/miniforge/registry.json"));
        assert_eq!(reg.product, PRODUCT_MINIFORGE);
        let snap: WorkbenchSnapshotV1 =
            round_trip(include_str!("../../../fixtures/miniforge/snapshot.json"));
        assert_eq!(snap.product, PRODUCT_MINIFORGE);
        assert_eq!(snap.registry_ref.registry_id, reg.registry_id);
    }

    #[test]
    fn snapshot_reuses_foundation_signature() {
        let snap: WorkbenchSnapshotV1 =
            serde_json::from_str(include_str!("../../../fixtures/portfolio/snapshot.json"))
                .expect("decode");
        // Proves the envelope signature is the foundation type, not a
        // re-spelled local copy.
        let _sig: SnapshotSignatureV1 = snap.signature;
    }
}
