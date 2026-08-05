// Title: Miniforge App Foundation
// Subtitle: generic axum data-plane router shared across miniforge-ai apps
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

//! Five-route axum data plane that every miniforge-ai app exposes
//! verbatim: `/v1/snapshots/latest`, `/v1/snapshots/latest/metadata`,
//! `/v1/snapshots/:snapshot_id`, `/v1/app-config`, and
//! `/v1/license/validate`.
//!
//! The product-specific bits — how to load a snapshot, what the
//! snapshot payload looks like, what tier the license validator
//! returns, what the app-config advertises — are factored behind
//! the `DataPlaneProvider` trait. Implementations live in each
//! product's own data-plane crate; `minibench-data-plane` is the
//! worked example.
//!
//! # Example
//!
//! ```ignore
//! use miniforge_app_foundation_contracts::*;
//! use miniforge_app_foundation_data_plane::{DataPlaneProvider, build_router};
//! use serde_json::Value;
//!
//! struct RiskProvider;
//!
//! #[async_trait::async_trait]
//! impl DataPlaneProvider for RiskProvider {
//!     async fn latest_snapshot(&self) -> Result<Value, String> { ... }
//!     async fn snapshot_by_id(&self, id: &str) -> Result<Option<Value>, String> { ... }
//!     async fn app_config(&self) -> AppConfigV1 { ... }
//!     async fn validate_license(&self) -> LicenseValidationResponseV1 { ... }
//! }
//!
//! let app = build_router(RiskProvider);
//! ```

mod strings;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use miniforge_app_foundation_contracts::{AppConfigV1, LicenseValidationResponseV1};
use serde_json::{Value, json};

//------------------------------------------------------------------------------ Layer 0
// Provider trait

/// Product-specific behavior for the five standard data-plane routes.
///
/// Snapshots are exchanged as `serde_json::Value` so this crate
/// stays domain-neutral — each product serializes its own snapshot
/// type to JSON before handing it to the router. The provider
/// returns errors as `String` for the same reason; routing
/// translates them into 5xx JSON responses with a stable shape.
#[async_trait::async_trait]
pub trait DataPlaneProvider: Send + Sync + 'static {
    /// Fetch the most recent snapshot.
    async fn latest_snapshot(&self) -> Result<Value, String>;

    /// Fetch a snapshot by its `snapshot_id`. `Ok(None)` distinguishes
    /// "not found" (→ 404) from a fault (→ 500).
    async fn snapshot_by_id(&self, snapshot_id: &str) -> Result<Option<Value>, String>;

    /// Static distribution metadata for the running app build.
    async fn app_config(&self) -> AppConfigV1;

    /// Validate the configured license. The response's `tier` is
    /// product-defined.
    async fn validate_license(&self) -> LicenseValidationResponseV1;

    /// Build a metadata-only summary of the latest snapshot. The
    /// default implementation derives common fields
    /// (`schema_version`, `snapshot_id`, `generated_at`, `signature`)
    /// from the full snapshot JSON. Products with richer metadata
    /// (counts, derived stats, confidence, etc.) override.
    async fn latest_snapshot_metadata(&self) -> Result<Value, String> {
        let snapshot = self.latest_snapshot().await?;
        Ok(json!({
            "schema_version": snapshot.get("schema_version"),
            "snapshot_id":    snapshot.get("snapshot_id"),
            "generated_at":   snapshot.get("generated_at"),
            "signature":      snapshot.get("signature"),
        }))
    }
}

//------------------------------------------------------------------------------ Layer 1
// Router builder

/// Build the standard five-route axum router around `provider`.
pub fn build_router<P: DataPlaneProvider>(provider: P) -> Router {
    Router::new()
        .route("/v1/snapshots/latest", get(handle_latest_snapshot::<P>))
        .route(
            "/v1/snapshots/latest/metadata",
            get(handle_latest_snapshot_metadata::<P>),
        )
        .route(
            "/v1/snapshots/:snapshot_id",
            get(handle_snapshot_by_id::<P>),
        )
        .route("/v1/app-config", get(handle_app_config::<P>))
        .route("/v1/license/validate", post(handle_validate_license::<P>))
        .with_state(Arc::new(provider))
}

//------------------------------------------------------------------------------ Layer 2
// Handlers

async fn handle_latest_snapshot<P: DataPlaneProvider>(
    State(provider): State<Arc<P>>,
) -> impl IntoResponse {
    match provider.latest_snapshot().await {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(message) => internal_error(message),
    }
}

async fn handle_latest_snapshot_metadata<P: DataPlaneProvider>(
    State(provider): State<Arc<P>>,
) -> impl IntoResponse {
    match provider.latest_snapshot_metadata().await {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(message) => internal_error(message),
    }
}

async fn handle_snapshot_by_id<P: DataPlaneProvider>(
    State(provider): State<Arc<P>>,
    Path(snapshot_id): Path<String>,
) -> impl IntoResponse {
    match provider.snapshot_by_id(&snapshot_id).await {
        Ok(Some(value)) => (StatusCode::OK, Json(value)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status":  strings::ENVELOPE_STATUS_NOT_FOUND,
                "message": strings::snapshot_not_found(&snapshot_id),
            })),
        )
            .into_response(),
        Err(message) => internal_error(message),
    }
}

async fn handle_app_config<P: DataPlaneProvider>(
    State(provider): State<Arc<P>>,
) -> impl IntoResponse {
    Json(provider.app_config().await)
}

async fn handle_validate_license<P: DataPlaneProvider>(
    State(provider): State<Arc<P>>,
) -> impl IntoResponse {
    Json(provider.validate_license().await)
}

fn internal_error(message: String) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "status":  strings::ENVELOPE_STATUS_ERROR,
            "message": message,
        })),
    )
        .into_response()
}
