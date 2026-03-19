# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.2](https://github.com/aws/aws-lambda-rust-runtime/compare/aws_lambda_events-v1.1.1...aws_lambda_events-v1.1.2) - 2026-03-19

### Fixed

- fix handling of null groupConfiguration ([#1130](https://github.com/aws/aws-lambda-rust-runtime/pull/1130))

## [1.1.1](https://github.com/aws/aws-lambda-rust-runtime/compare/aws_lambda_events-v1.0.3...aws_lambda_events-v1.1.1) - 2026-03-11

### Added

- *(lambda-events)* support X509 custom authorizer in IoT events ([#1114](https://github.com/aws/aws-lambda-rust-runtime/pull/1114))
- *(lambda-events)* add Control Tower lifecycle events module ([#1107](https://github.com/aws/aws-lambda-rust-runtime/pull/1107))
- *(lambda-events)* add VPC Lattice event structures ([#1036](https://github.com/aws/aws-lambda-rust-runtime/pull/1036))
- Add builder pattern support for event response types ([#1090](https://github.com/aws/aws-lambda-rust-runtime/pull/1090))

### Fixed

- *(sns)* support SubscriptionConfirmation and UnsubscribeConfirmation message types ([#1102](https://github.com/aws/aws-lambda-rust-runtime/pull/1102))
- *(lambda-events)* `claims_to_add_or_override` type in `CognitoEventUserPoolsPreTokenGenV2` ([#1100](https://github.com/aws/aws-lambda-rust-runtime/pull/1100))

### Other

- *(lambda-events)* deprecate authorizer-specific fields in `ApiGatewayV2httpRequest` ([#1089](https://github.com/aws/aws-lambda-rust-runtime/pull/1089))

## [1.0.3](https://github.com/aws/aws-lambda-rust-runtime/compare/aws_lambda_events-v1.0.2...aws_lambda_events-v1.0.3) - 2026-01-06

### Changed

- MSRV updated from 1.82.0 to 1.84.0, enabled MSRV-aware resolver ([#1078](https://github.com/aws/aws-lambda-rust-runtime/pull/1078))
