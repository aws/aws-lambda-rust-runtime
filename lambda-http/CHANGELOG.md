# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0-rc1.1](https://github.com/aws/aws-lambda-rust-runtime/compare/lambda_http-v1.1.0-rc1...lambda_http-v1.1.0-rc1.1) - 2026-03-12

### Other

- *(lambda-managed-instances)* warn on `run()` with `AWS_LAMBDA_MAX_CONCURRENCY, rename feature experimental-concurrency -> concurrency->tokio
- disable default features of lambda-runtime ([#1093](https://github.com/aws/aws-lambda-rust-runtime/pull/1093))
- *(deps)* update axum-extra requirement from 0.10.2 to 0.12.5 ([#1079](https://github.com/aws/aws-lambda-rust-runtime/pull/1079))
