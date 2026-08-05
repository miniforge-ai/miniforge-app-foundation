// Title: Miniforge App Foundation
// Subtitle: Centralized user-facing + envelope strings for the data-plane router
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

//! Centralized user-facing + envelope strings for the
//! `miniforge-app-foundation-data-plane` router.
//!
//! Per `.standards/languages/rust.mdc` § String constants:
//! "All user-facing and advisory text lives in a `strings.rs`
//! module as `pub const`. Domain logic references
//! `crate::strings::*` — never inline string literals in business
//! logic." A future localization pass swaps values here without
//! touching handler logic.

// ── HTTP envelope statuses ─────────────────────────────────────────
pub const ENVELOPE_STATUS_NOT_FOUND: &str = "not_found";
pub const ENVELOPE_STATUS_ERROR: &str = "error";

// ── Snapshot-by-id 404 message ─────────────────────────────────────
pub const SNAPSHOT_NOT_FOUND_PREFIX: &str = "Snapshot ";
pub const SNAPSHOT_NOT_FOUND_SUFFIX: &str = " was not found.";

pub fn snapshot_not_found(id: &str) -> String {
    format!("{SNAPSHOT_NOT_FOUND_PREFIX}{id}{SNAPSHOT_NOT_FOUND_SUFFIX}")
}
