;; Title: Miniforge App Foundation
;; Subtitle: Malli mirror of the workbench wire types (Clojure consumer)
;; Author: Christopher Lester
;; Copyright 2025-2026 Christopher Lester (christopher@miniforge.ai)
;;
;; Licensed under the Apache License, Version 2.0 (the "License");
;; you may not use this file except in compliance with the License.
;; You may obtain a copy of the License at
;;
;;     http://www.apache.org/licenses/LICENSE-2.0
;;
;; Unless required by applicable law or agreed to in writing, software
;; distributed under the License is distributed on an "AS IS" BASIS,
;; WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
;; See the License for the specific language governing permissions and
;; limitations under the License.

(ns ai.miniforge.workbench-contract.schema
  "Malli mirror of the canonical Rust workbench-contract types
   (crates/workbench-contract), for Clojure tenant adapters to validate
   at their emit boundary (standard 004 — validate at interfaces).

   Rust is canonical. This mirror validates the KEYWORDIZED JSON shape
   a Clojure adapter produces before Transit/JSON serialization: keys
   are unqualified snake_case keywords (:snapshot_id, :state_var_id),
   matching the wire field names. The golden fixtures under
   fixtures/<tenant>/ are the source of truth both this mirror and the
   Rust types must round-trip — keep them in lock-step.

   Field rules follow serde. An Option<T> field may be omitted or null.
   A #[serde(default)] field may be omitted but not null. Unsigned
   integers carry their Rust bounds. bindings/parity_cases.json holds
   the edge cases; the Rust and Clojure suites both run it.")

;; ---------------------------------------------------------------- enums
;;
;; Enum VALUES are strings, not keywords: JSON parsing keywordizes
;; KEYS only, so a wire value like "pass" stays a string. An adapter
;; builds the wire-shape map (keyword keys, string values), validates
;; it here, then serializes — keys stringify, values already match.

(def StateStatus
  [:enum "pass" "warn" "fail" "blocked" "not_applicable" "unknown"])

(def StateVarKind
  [:enum "presence" "integrity" "sufficiency" "alignment" "compliance"
   "quality" "regression" "budget" "privacy"])

(def ValueType
  [:enum "boolean" "number" "string" "enum" "object" "array"])

(def Lifecycle
  [:enum "draft" "active" "deprecated" "experimental"])

(def ViewTier
  [:enum "generic" "primitive" "bespoke"])

;; ---------------------------------------------------------------- integers
;;
;; JSON integers past Long/MAX_VALUE parse as BigInt, so the u64 bound
;; is checked with integer? rather than int?. A float such as 2.0 is
;; not an integer here, and serde rejects it for an integer field too.

(def u32-max
  "Largest value of a Rust u32."
  4294967295)

(def u64-max
  "Largest value of a Rust u64."
  18446744073709551615N)

(def U32
  [:and integer? [:>= 0] [:<= u32-max]])

(def U64
  [:and integer? [:>= 0] [:<= u64-max]])

;; ---------------------------------------------------------------- evidence

(def EvidenceSpan
  [:map {:closed false}
   [:start {:optional true} [:maybe U64]]
   [:end {:optional true} [:maybe U64]]])

(def EvidenceRef
  [:map {:closed false}
   [:id :string]
   [:source_role :string]
   [:artifact_id {:optional true} [:maybe :string]]
   [:span {:optional true} [:maybe EvidenceSpan]]
   [:quote {:optional true} [:maybe :string]]
   [:hash {:optional true} [:maybe :string]]
   [:trust_level {:optional true} [:maybe :string]]
   [:created_at {:optional true} [:maybe :string]]])

;; ---------------------------------------------------------------- registry

(def EvidenceRequirements
  [:map {:closed false}
   [:required_refs [:sequential :string]]
   [:min_count {:optional true} [:maybe U32]]
   [:must_include_hash {:optional true} [:maybe :boolean]]
   [:must_include_source_role {:optional true} [:maybe :boolean]]
   [:freshness_sla_hours {:optional true} [:maybe U64]]])

(def StateVariable
  [:map {:closed false}
   [:id :string]
   [:version :string]
   [:product :string]
   [:area :string]
   [:kind StateVarKind]
   [:description :string]
   [:value_type ValueType]
   [:thresholds {:optional true} [:map-of :keyword number?]]
   [:evidence_requirements EvidenceRequirements]
   [:score_components [:sequential :string]]
   [:gate_effects [:map-of :keyword :string]]
   [:lifecycle Lifecycle]
   [:owner {:optional true} [:maybe :string]]
   [:notes {:optional true} [:maybe :string]]])

(def StateVarRegistry
  [:map {:closed false}
   [:schema_version :string]
   [:registry_id :string]
   [:version :string]
   [:product :string]
   [:state_vars [:sequential StateVariable]]])

(def RegistryRef
  [:map {:closed false}
   [:registry_id :string]
   [:version :string]])

;; ---------------------------------------------------------------- evaluation

(def Finding
  [:map {:closed false}
   [:severity {:optional true} [:maybe :string]]
   [:message :string]
   [:data {:optional true} any?]])

(def StateEvaluation
  [:map {:closed false}
   [:state_var_id :string]
   [:product :string]
   [:status StateStatus]
   [:value {:optional true} any?]
   [:score number?]
   [:confidence number?]
   ;; Dynamic-map keys keywordize on JSON parse, same as fixed keys, so
   ;; they validate as :keyword; only enum/string VALUES stay strings.
   [:score_components {:optional true} [:maybe [:map-of :keyword number?]]]
   [:evidence_refs [:sequential EvidenceRef]]
   [:findings [:sequential Finding]]
   [:gate_effect :string]
   [:regression {:optional true} any?]
   [:evaluated_at :string]])

;; ---------------------------------------------------------------- view plugin

(def ViewPlugin
  [:map {:closed false}
   [:id :string]
   [:product :string]
   [:tier ViewTier]
   [:consumes {:optional true} [:sequential :string]]
   [:title :string]])

;; ---------------------------------------------------------------- run variant

(def VariantRef
  [:map {:closed false}
   [:id :string]
   [:version {:optional true} [:maybe :string]]])

(def RunVariant
  [:map {:closed false}
   [:experiment_id :string]
   [:label :string]
   [:workflow {:optional true} [:maybe VariantRef]]
   [:prompt {:optional true} [:maybe VariantRef]]
   [:model {:optional true} [:maybe :string]]
   [:method {:optional true} [:maybe :string]]
   [:axes {:optional true} [:map-of :keyword :string]]])

;; ---------------------------------------------------------------- snapshot

;; Mirror of miniforge-app-foundation-contracts SnapshotSignatureV1.
(def SnapshotSignature
  [:map {:closed false}
   [:algorithm :string]
   [:digest :string]
   [:canonicalization :string]])

(def WorkbenchSnapshot
  [:map {:closed false}
   [:schema_version :string]
   [:snapshot_id :string]
   [:generated_at :string]
   [:product :string]
   [:run_id :string]
   [:variant {:optional true} [:maybe RunVariant]]
   [:registry_ref RegistryRef]
   [:source_hashes {:optional true} [:maybe [:sequential :string]]]
   [:evaluations [:sequential StateEvaluation]]
   [:entities {:optional true} [:map-of :keyword any?]]
   [:metadata {:optional true} any?]
   [:signature SnapshotSignature]])

(comment
  WorkbenchSnapshot
  StateVarRegistry)
