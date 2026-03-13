# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

<<<<<<< HEAD
## [1.1.0-rc1.1](https://github.com/aws/aws-lambda-rust-runtime/compare/lambda_http-v1.1.0-rc1...lambda_http-v1.1.0-rc1.1) - 2026-03-12

### Other

- *(lambda-managed-instances)* warn on `run()` with `AWS_LAMBDA_MAX_CONCURRENCY, rename feature experimental-concurrency -> concurrency->tokio
=======
## [1.1.1](https://github.com/aws/aws-lambda-rust-runtime/compare/v1.0.2...lambda_http-v1.1.1) - 2026-03-11

### Added

- *(lambda-managed-instances)* Lambda Managed Instances support via `concurrency-tokio` feature flag with `run_concurrent()` and `BoxCloneService` for concurrent streaming responses ([#1067](https://github.com/aws/aws-lambda-rust-runtime/pull/1067))
- Add builder pattern support for event response types ([#1090](https://github.com/aws/aws-lambda-rust-runtime/pull/1090))

### Changed

- MSRV updated from 1.82.0 to 1.84.0, enabled MSRV-aware resolver ([#1078](https://github.com/aws/aws-lambda-rust-runtime/pull/1078))

### Other

- *(lambda-managed-instances)* add `tokio_unstable` to known cfgs to avoid linter warns ([#1095](https://github.com/aws/aws-lambda-rust-runtime/pull/1095))
- *(lambda-managed-instances)* warn on `run()` with `AWS_LAMBDA_MAX_CONCURRENCY`, rename feature `experimental-concurrency` to `concurrency-tokio` ([#1095](https://github.com/aws/aws-lambda-rust-runtime/pull/1095))
- *(lambda-managed-instances)* verify request-ID isolation in concurrent exec ([#1086](https://github.com/aws/aws-lambda-rust-runtime/pull/1086))
- Introducing Harness Testing ([#1103](https://github.com/aws/aws-lambda-rust-runtime/pull/1103))
>>>>>>> 7f3f758 (chore: some other changes)
- disable default features of lambda-runtime ([#1093](https://github.com/aws/aws-lambda-rust-runtime/pull/1093))
- *(deps)* update axum-extra requirement from 0.10.2 to 0.12.5 ([#1079](https://github.com/aws/aws-lambda-rust-runtime/pull/1079))
