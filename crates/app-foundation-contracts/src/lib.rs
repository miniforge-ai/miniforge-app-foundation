// Title: Miniforge App Foundation
// Subtitle: domain-neutral boundary contracts shared across miniforge-ai apps
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

//! Boundary types every miniforge-ai app exposes regardless of its
//! domain (portfolio risk, career intelligence, future products).
//!
//! Each app owns its own domain crate (`risk-core/domain`,
//! `theseus-engine/domain`, ...) where the product-specific shapes
//! live. This crate owns only the *infrastructure* shapes — the ones
//! that are identical across apps because they describe how the app
//! talks to its data plane (snapshot signatures), how the app
//! authenticates (license validation), and how the app advertises
//! its distribution channel (app config).
//!
//! Schema versions are pinned by the `*_V1` string constants;
//! consumers compare against the constant rather than re-spelling
//! the literal at every callsite.

use serde::{Deserialize, Serialize};

//------------------------------------------------------------------------------
// Schema version pins

pub const APP_CONFIG_V1: &str = "app_config/v1";
pub const LICENSE_VALIDATION_V1: &str = "license_validation/v1";

//------------------------------------------------------------------------------
// Snapshot signature
//
// Used by every app's snapshot envelope. The fields are domain-
// neutral: an algorithm name, a hex digest, a canonicalization
// label so the verifier knows how the bytes were canonicalized
// before hashing.

/// Cryptographic envelope for any product's snapshot payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotSignatureV1 {
    pub algorithm: String,
    pub digest: String,
    pub canonicalization: String,
}

//------------------------------------------------------------------------------
// License validation
//
// The data-plane's `/v1/license/validate` endpoint returns this
// shape. `tier` is product-defined (free/pro/enterprise; or
// dev/local/governed; or whatever the product chooses). The
// envelope is universal.

/// Response shape for `/v1/license/validate`. `tier` is product-defined.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicenseValidationResponseV1 {
    pub valid: bool,
    pub tier: String,
}

//------------------------------------------------------------------------------
// App config
//
// Every app's data plane exposes a static configuration document
// at `/v1/app-config`. The shape is product-neutral: a schema
// version, a minimum-supported snapshot major (so the app can
// refuse to render a snapshot it can't safely interpret), and a
// distribution descriptor.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DistributionV1 {
    /// Distribution channel — `direct-download`, `app-store`,
    /// `enterprise-mdm`, etc. Product-defined string.
    pub channel: String,
    /// Target platform — `macos`, `ios`, `windows`, etc.
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfigV1 {
    pub schema_version: String,
    pub minimum_supported_snapshot_major: u32,
    pub distribution: DistributionV1,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_signature_round_trips_through_json() {
        let original = SnapshotSignatureV1 {
            algorithm: "sha256".to_string(),
            digest: "deadbeef".to_string(),
            canonicalization: "json-c14n/1".to_string(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: SnapshotSignatureV1 = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn license_validation_round_trips_through_json() {
        let original = LicenseValidationResponseV1 {
            valid: true,
            tier: "pro".to_string(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: LicenseValidationResponseV1 =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn app_config_round_trips_through_json() {
        let original = AppConfigV1 {
            schema_version: APP_CONFIG_V1.to_string(),
            minimum_supported_snapshot_major: 1,
            distribution: DistributionV1 {
                channel: "direct-download".to_string(),
                platform: "macos".to_string(),
            },
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: AppConfigV1 = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn schema_version_constants_are_stable() {
        // Pin the literals so a typo regression here surfaces as a
        // test failure rather than a silent wire-format drift.
        assert_eq!(APP_CONFIG_V1, "app_config/v1");
        assert_eq!(LICENSE_VALIDATION_V1, "license_validation/v1");
    }
}
