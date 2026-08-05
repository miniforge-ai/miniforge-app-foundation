// Title: Miniforge App Foundation
// Subtitle: end-to-end test for the generic data-plane router
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

//! Drives the router with a fake provider so the five-route contract
//! is exercised end-to-end without spinning up a TCP listener.

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use miniforge_app_foundation_contracts::{
    APP_CONFIG_V1, AppConfigV1, DistributionV1, LicenseValidationResponseV1,
};
use miniforge_app_foundation_data_plane::{DataPlaneProvider, build_router};
use serde_json::{Value, json};
use tower::ServiceExt;

struct FakeProvider {
    snapshots: Vec<Value>,
}

#[async_trait]
impl DataPlaneProvider for FakeProvider {
    async fn latest_snapshot(&self) -> Result<Value, String> {
        self.snapshots
            .last()
            .cloned()
            .ok_or_else(|| "no snapshots".to_string())
    }

    async fn snapshot_by_id(&self, snapshot_id: &str) -> Result<Option<Value>, String> {
        Ok(self
            .snapshots
            .iter()
            .find(|snap| snap.get("snapshot_id").and_then(Value::as_str) == Some(snapshot_id))
            .cloned())
    }

    async fn app_config(&self) -> AppConfigV1 {
        AppConfigV1 {
            schema_version: APP_CONFIG_V1.to_string(),
            minimum_supported_snapshot_major: 1,
            distribution: DistributionV1 {
                channel: "test-channel".to_string(),
                platform: "test-platform".to_string(),
            },
        }
    }

    async fn validate_license(&self) -> LicenseValidationResponseV1 {
        LicenseValidationResponseV1 {
            valid: true,
            tier: "test-tier".to_string(),
        }
    }
}

fn fake_provider() -> FakeProvider {
    FakeProvider {
        snapshots: vec![json!({
            "schema_version": "fake_snapshot/v1",
            "snapshot_id":    "snap-001",
            "generated_at":   "2026-05-08T00:00:00Z",
            "signature": {
                "algorithm":        "sha256",
                "digest":           "deadbeef",
                "canonicalization": "json-c14n/1",
            },
            "payload": "anything-goes",
        })],
    }
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    serde_json::from_slice(&bytes).expect("parse JSON body")
}

#[tokio::test]
async fn latest_snapshot_returns_provider_payload() {
    let app = build_router(fake_provider());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/snapshots/latest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(
        body.get("snapshot_id").and_then(Value::as_str),
        Some("snap-001")
    );
}

#[tokio::test]
async fn latest_snapshot_metadata_uses_default_summary() {
    let app = build_router(fake_provider());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/snapshots/latest/metadata")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(
        body.get("schema_version").and_then(Value::as_str),
        Some("fake_snapshot/v1")
    );
    assert_eq!(
        body.get("snapshot_id").and_then(Value::as_str),
        Some("snap-001")
    );
    assert!(body.get("signature").is_some());
}

#[tokio::test]
async fn snapshot_by_id_returns_match_or_404() {
    let app = build_router(fake_provider());

    let hit = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/snapshots/snap-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(hit.status(), StatusCode::OK);

    let miss = app
        .oneshot(
            Request::builder()
                .uri("/v1/snapshots/not-real")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(miss.status(), StatusCode::NOT_FOUND);
    let body = body_json(miss).await;
    assert_eq!(
        body.get("status").and_then(Value::as_str),
        Some("not_found")
    );
}

#[tokio::test]
async fn app_config_returns_provider_descriptor() {
    let app = build_router(fake_provider());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/app-config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(
        body.get("schema_version").and_then(Value::as_str),
        Some(APP_CONFIG_V1)
    );
    assert_eq!(
        body.pointer("/distribution/channel")
            .and_then(Value::as_str),
        Some("test-channel")
    );
}

#[tokio::test]
async fn validate_license_returns_provider_tier() {
    let app = build_router(fake_provider());
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/license/validate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body.get("valid").and_then(Value::as_bool), Some(true));
    assert_eq!(body.get("tier").and_then(Value::as_str), Some("test-tier"));
}

#[tokio::test]
async fn provider_fault_surfaces_as_500() {
    struct FaultyProvider;

    #[async_trait]
    impl DataPlaneProvider for FaultyProvider {
        async fn latest_snapshot(&self) -> Result<Value, String> {
            Err("intentional fault".to_string())
        }
        async fn snapshot_by_id(&self, _: &str) -> Result<Option<Value>, String> {
            Ok(None)
        }
        async fn app_config(&self) -> AppConfigV1 {
            AppConfigV1 {
                schema_version: APP_CONFIG_V1.to_string(),
                minimum_supported_snapshot_major: 1,
                distribution: DistributionV1 {
                    channel: "x".to_string(),
                    platform: "x".to_string(),
                },
            }
        }
        async fn validate_license(&self) -> LicenseValidationResponseV1 {
            LicenseValidationResponseV1 {
                valid: false,
                tier: "x".to_string(),
            }
        }
    }

    let app = build_router(FaultyProvider);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/snapshots/latest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_json(response).await;
    assert_eq!(body.get("status").and_then(Value::as_str), Some("error"));
    assert_eq!(
        body.get("message").and_then(Value::as_str),
        Some("intentional fault")
    );
}
