# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0-rc1.1](https://github.com/aws/aws-lambda-rust-runtime/compare/lambda_runtime-v1.1.0-rc1...lambda_runtime-v1.1.0-rc1.1) - 2026-03-12

### Added

- *(lambda-runtime)* log non-2xx Lambda Runtime API responses with status and body ([#1109](https://github.com/aws/aws-lambda-rust-runtime/pull/1109))

### Fixed

- *(test)* fix test_concurrent_structured_logging_isolation ([#1121](https://github.com/aws/aws-lambda-rust-runtime/pull/1121))

### Other

- Introducing Harness Testing ([#1103](https://github.com/aws/aws-lambda-rust-runtime/pull/1103))
- add tokio_unstable to known cfgs to avoid linter warns
- *(lambda-managed-instances)* warn on `run()` with `AWS_LAMBDA_MAX_CONCURRENCY, rename feature experimental-concurrency -> concurrency->tokio
- Add builder pattern support for event response types ([#1090](https://github.com/aws/aws-lambda-rust-runtime/pull/1090))
- verify request-ID isolation in concurrent exec ([#1086](https://github.com/aws/aws-lambda-rust-runtime/pull/1086))
