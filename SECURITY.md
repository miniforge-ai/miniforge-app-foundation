<!--
  Title: Miniforge App Foundation
  Author: Christopher Lester (christopher@miniforge.ai)
  Copyright 2025-2026 Christopher Lester. Licensed under Apache 2.0.
-->

# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| latest  | ✅        |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

To report a security vulnerability, email **security@miniforge.ai** with:

- A description of the vulnerability and its potential impact
- Steps to reproduce or proof-of-concept code
- Any suggested mitigations you are aware of

You should receive an acknowledgement within 48 hours. We will keep you informed
as we investigate and address the report.

## Scope

This repository is types and one router, so the surface is narrow but it sits
under everything else. In scope:

- Deserialization flaws reachable from snapshot or registry JSON — a malformed or
  hostile document causing memory unsafety, unbounded allocation, or a panic a
  caller cannot contain
- Anything in `build_router` that leaks provider error detail, serves a snapshot
  the caller should not reach, or lets a crafted `snapshot_id` path segment
  escape its route
- A change to a `*_V1` type or schema-version constant that silently alters the
  wire format for existing consumers

## Out of Scope

- Issues that require physical access to a machine
- Social engineering attacks
- Vulnerabilities in third-party dependencies (please report those upstream)
- Transport security, authentication, and authorization. This crate provides no
  authentication and terminates no TLS; how a router built here is bound and
  exposed is the hosting application's responsibility. Consumers bind loopback.
- Fixtures under `fixtures/` — synthetic reference data, not a security boundary

## Preferred Languages

We prefer reports in English.
