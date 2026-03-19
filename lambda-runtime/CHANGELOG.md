# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.2](https://github.com/aws/aws-lambda-rust-runtime/compare/lambda_runtime-v1.1.1...lambda_runtime-v1.1.2) - 2026-03-19

### Fixed

- fix handling of null groupConfiguration ([#1130](https://github.com/aws/aws-lambda-rust-runtime/pull/1130))

### Other

- release ([#1118](https://github.com/aws/aws-lambda-rust-runtime/pull/1118))

## [1.1.1](https://github.com/aws/aws-lambda-rust-runtime/compare/v1.0.2...lambda_runtime-v1.1.1) - 2026-03-11

Thank you to all the contributors who helped make this release possible. We appreciate your time, effort, and passion for the Rust Lambda community. ❤️

### Lambda Managed Instances

The runtime now supports Lambda Managed Instances via the `concurrency-tokio` feature flag (previously `experimental-concurrency`). Lambda Managed Instances allow multiple concurrent requests to be processed within a single execution environment. When `AWS_LAMBDA_MAX_CONCURRENCY` is set, the runtime spawns multiple independent long-poll workers to handle concurrent invocations. If the env var is unset or <= 1, it falls back to sequential behavior automatically — so the same handler works on both classic Lambda and Lambda Managed Instances. ([#1067](https://github.com/aws/aws-lambda-rust-runtime/pull/1067))

```toml
[dependencies]
lambda_runtime = { version = "1.1", features = ["concurrency-tokio"] }
```

```rust
lambda_runtime::run_concurrent(service_fn(my_handler)).await?;
```

For a complete working example, see [examples/basic-lambda-concurrent](https://github.com/aws/aws-lambda-rust-runtime/tree/main/examples/basic-lambda-concurrent). For detailed guidance on building functions for multi-concurrency, including shared state patterns and database connection pools, see the [Rust runtime for Lambda Managed Instances](https://docs.aws.amazon.com/lambda/latest/dg/lambda-managed-instances-rust.html) documentation.

We would like also to involve the community in a broader discussion about improving our approach on multiconcurrency. You can find the discussion in ([#1120](https://github.com/aws/aws-lambda-rust-runtime/issues/1120))

### Added

- *(lambda-managed-instances)* log non-2xx Lambda Runtime API responses with status and body ([#1109](https://github.com/aws/aws-lambda-rust-runtime/pull/1109))
- tenant ID propagation for multi-tenant Lambda use cases. The `tenant_id` is available in the `context.tenant_id` field (`Option<String>`) and is automatically extracted from the `lambda-runtime-aws-tenant-id` header when present ([#1082](https://github.com/aws/aws-lambda-rust-runtime/pull/1082))
- Add builder pattern support for event response types ([#1090](https://github.com/aws/aws-lambda-rust-runtime/pull/1090))

### Fixed

- *(test)* fix test_concurrent_structured_logging_isolation ([#1121](https://github.com/aws/aws-lambda-rust-runtime/pull/1121))

### Changed

- MSRV updated from 1.82.0 to 1.84.0, enabled MSRV-aware resolver ([#1078](https://github.com/aws/aws-lambda-rust-runtime/pull/1078))
- X-Ray trace ID now sourced from `Context` instead of environment variables ([#1067](https://github.com/aws/aws-lambda-rust-runtime/pull/1067))

### Other

- *(lambda-managed-instances)* add `tokio_unstable` to known cfgs to avoid linter warns ([#1095](https://github.com/aws/aws-lambda-rust-runtime/pull/1095))
- *(lambda-managed-instances)* verify request-ID isolation in concurrent exec ([#1086](https://github.com/aws/aws-lambda-rust-runtime/pull/1086))
- *(lambda-managed-instances)* warn on `run()` with `AWS_LAMBDA_MAX_CONCURRENCY`, rename feature `experimental-concurrency` to `concurrency-tokio` ([#1095](https://github.com/aws/aws-lambda-rust-runtime/pull/1095))
- Introducing Harness Testing ([#1103](https://github.com/aws/aws-lambda-rust-runtime/pull/1103))

## [1.1.0-rc1](https://github.com/aws/aws-lambda-rust-runtime/compare/v1.0.2...lambda_runtime-v1.1.0-rc1) - 2026-02-04

### Added

- *(lambda-managed-instances)* Lambda Managed Instances support via `concurrency-tokio` feature flag ([#1067](https://github.com/aws/aws-lambda-rust-runtime/pull/1067))
- *(lambda-managed-instances)* tenant ID propagation for multi-tenant Lambda use cases ([#1082](https://github.com/aws/aws-lambda-rust-runtime/pull/1082))

### Changed

- MSRV updated from 1.82.0 to 1.84.0, enabled MSRV-aware resolver ([#1078](https://github.com/aws/aws-lambda-rust-runtime/pull/1078))
- X-Ray trace ID now sourced from `Context` instead of environment variables ([#1067](https://github.com/aws/aws-lambda-rust-runtime/pull/1067))
