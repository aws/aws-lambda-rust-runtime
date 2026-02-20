📬 *Issue #, if available:*

N/A

✍️ *Description of changes:*

This PR adds Docker-based testing infrastructure using AWS's [containerized-test-runner-for-aws-lambda](https://github.com/aws/containerized-test-runner-for-aws-lambda), enabling automated testing of Lambda functions in a containerized environment that closely mirrors the AWS Lambda execution environment.

The PR introduces a `test-dockerized` Makefile target that runs test suites defined in `test/dockerized/*.json` files. These test suites specify handlers to test (from the examples directory), request payloads, and expected response assertions with optional jq transforms for validation.

The infrastructure reuses Lambda binaries from the `/examples` folder as test handlers, demonstrating the concept with an initial test case for `basic-lambda`. Additional tests and multi-concurrency scenarios can be added by creating new test suite JSON files.

A GitHub Actions workflow provides CI/CD integration for automated testing on pull requests.

## Testing

Run dockerized tests locally:
```bash
make test-dockerized
```

Run RIE tests:
```bash
make test-rie
HANDLERS_TO_BUILD="basic-lambda basic-sqs" make test-rie
```

Build specific examples:
```bash
EXAMPLES="basic-lambda basic-lambda-concurrent" make build-examples
```

🔏 *By submitting this pull request*

- [x] I confirm that I've ran `cargo +nightly fmt`.
- [x] I confirm that I've ran `cargo clippy --fix`.
- [x] I confirm that I've made a best effort attempt to update all relevant documentation.
- [x] I confirm that my contribution is made under the terms of the Apache 2.0 license.on.
- [x] I confirm that my contribution is made under the terms of the Apache 2.0 license.
