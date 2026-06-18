# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `PooledClient`: a Runtime API client that can rebuild its connection pool after a
  SnapStart VM restore via `reset_pool()`, so stale connections to the Runtime API
  (RAPID) are discarded. Implemented lock-free with `OnceLock`; `call()` has no
  hot-path lock. Construct via `PooledClient::builder().build()`.
- `PooledClientBuilder`, returned by `PooledClient::builder()`.
- `RuntimeApiClient` trait abstracting `call`, implemented by both `Client` and
  `PooledClient`.

### Deprecated

- `Client` and `ClientBuilder::build` are superseded by `PooledClient` /
  `PooledClient::builder().build()`. `Client` is unchanged and fully functional; it
  simply cannot reset its connection pool after a SnapStart restore. They are
  documented as superseded but do **not** carry a `#[deprecated]` attribute, so no
  compiler warnings are emitted; they are expected to be removed in the next major
  version (2.0.0). This keeps the crate non-breaking (no major version bump).

## [1.0.3](https://github.com/aws/aws-lambda-rust-runtime/compare/lambda_runtime_api_client-v1.0.2...lambda_runtime_api_client-v1.0.3) - 2026-03-19

### Other

- release ([#1118](https://github.com/aws/aws-lambda-rust-runtime/pull/1118))

## [1.0.2](https://github.com/aws/aws-lambda-rust-runtime/compare/v1.0.2...lambda_runtime_api_client-v1.0.2) - 2026-01-06

### Added

- *(lambda-managed-instances)* API client connection pooling for concurrent requests ([#1067](https://github.com/aws/aws-lambda-rust-runtime/pull/1067))

### Changed

- MSRV updated from 1.82.0 to 1.84.0, enabled MSRV-aware resolver ([#1078](https://github.com/aws/aws-lambda-rust-runtime/pull/1078))
