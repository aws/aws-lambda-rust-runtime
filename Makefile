INTEG_STACK_NAME ?= rust-lambda-integration-tests
INTEG_FUNCTIONS_BUILD := runtime-fn runtime-trait http-fn http-trait
INTEG_FUNCTIONS_INVOKE := RuntimeFn RuntimeFnAl2 RuntimeTrait RuntimeTraitAl2 Python PythonAl2
INTEG_API_INVOKE := RestApiUrl HttpApiUrl
INTEG_EXTENSIONS := extension-fn extension-trait logs-trait
# Using musl to run extensions on both AL1 and AL2
INTEG_ARCH := x86_64-unknown-linux-musl
RIE_MAX_CONCURRENCY ?= 4
OUTPUT_DIR ?= test/dockerized/tasks
HANDLERS_TO_BUILD ?=
HANDLER ?=

# Load environment variables from .env file if it exists
-include .env
export

.PHONY: help pr-check integration-tests check-event-features fmt build-examples test-rie test-rie-lmi nuke test-dockerized

.DEFAULT_GOAL := help

define uppercase
$(shell sed -r 's/(^|-)(\w)/\U\2/g' <<< $(1))
endef

pr-check:
	cargo +1.54.0 check --all
	cargo +stable fmt --all -- --check
	cargo +stable clippy
	cargo +1.54.0 test
	cargo +stable test

integration-tests:
# Build Integration functions
	cargo zigbuild --release --target $(INTEG_ARCH) -p lambda_integration_tests
	rm -rf ./build
	mkdir -p ./build
	${MAKE} ${MAKEOPTS} $(foreach function,${INTEG_FUNCTIONS_BUILD}, build-integration-function-${function})
	${MAKE} ${MAKEOPTS} $(foreach extension,${INTEG_EXTENSIONS}, build-integration-extension-${extension})
# Deploy to AWS
	sam deploy \
		--template lambda-integration-tests/template.yaml \
		--stack-name ${INTEG_STACK_NAME} \
		--capabilities CAPABILITY_IAM \
		--resolve-s3 \
		--no-fail-on-empty-changeset
# Invoke functions
	${MAKE} ${MAKEOPTS} $(foreach function,${INTEG_FUNCTIONS_INVOKE}, invoke-integration-function-${function})
	${MAKE} ${MAKEOPTS} $(foreach api,${INTEG_API_INVOKE}, invoke-integration-api-${api})

build-integration-function-%:
	mkdir -p ./build/$*
	cp -v ./target/$(INTEG_ARCH)/release/$* ./build/$*/bootstrap

build-integration-extension-%:
	mkdir -p ./build/$*/extensions
	cp -v ./target/$(INTEG_ARCH)/release/$* ./build/$*/extensions/$(call uppercase,$*)

invoke-integration-function-%:
	aws lambda invoke --function-name $$(aws cloudformation describe-stacks --stack-name $(INTEG_STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`$*`].OutputValue' \
		--output text) --payload '{"command": "hello"}' --cli-binary-format raw-in-base64-out /dev/stdout

invoke-integration-api-%:
	$(eval API_URL := $(shell aws cloudformation describe-stacks --stack-name $(INTEG_STACK_NAME) \
		--query 'Stacks[0].Outputs[?OutputKey==`$*`].OutputValue' \
		--output text))
	curl $(API_URL)/get
	curl $(API_URL)/trait/get
	curl $(API_URL)/al2/get
	curl $(API_URL)/al2-trait/get
	curl -X POST -d '{"command": "hello"}' $(API_URL)/post
	curl -X POST -d '{"command": "hello"}' $(API_URL)/trait/post
	curl -X POST -d '{"command": "hello"}' $(API_URL)/al2/post
	curl -X POST -d '{"command": "hello"}' $(API_URL)/al2-trait/post

# Test individual event features to ensure optional dependencies
# are correctly loaded when all default features are disabled.
check-event-features:
	cargo test --package aws_lambda_events --no-default-features --features activemq
	cargo test --package aws_lambda_events --no-default-features --features alb
	cargo test --package aws_lambda_events --no-default-features --features apigw
	cargo test --package aws_lambda_events --no-default-features --features appsync
	cargo test --package aws_lambda_events --no-default-features --features autoscaling
	cargo test --package aws_lambda_events --no-default-features --features bedrock_agent_runtime
	cargo test --package aws_lambda_events --no-default-features --features chime_bot
	cargo test --package aws_lambda_events --no-default-features --features clientvpn
	cargo test --package aws_lambda_events --no-default-features --features cloudwatch_alarms
	cargo test --package aws_lambda_events --no-default-features --features cloudwatch_events
	cargo test --package aws_lambda_events --no-default-features --features cloudwatch_logs
	cargo test --package aws_lambda_events --no-default-features --features code_commit
	cargo test --package aws_lambda_events --no-default-features --features codebuild
	cargo test --package aws_lambda_events --no-default-features --features codedeploy
	cargo test --package aws_lambda_events --no-default-features --features codepipeline_cloudwatch
	cargo test --package aws_lambda_events --no-default-features --features codepipeline_job
	cargo test --package aws_lambda_events --no-default-features --features cognito
	cargo test --package aws_lambda_events --no-default-features --features config
	cargo test --package aws_lambda_events --no-default-features --features connect
	cargo test --package aws_lambda_events --no-default-features --features documentdb
	cargo test --package aws_lambda_events --no-default-features --features dynamodb
	cargo test --package aws_lambda_events --no-default-features --features ecr_scan
	cargo test --package aws_lambda_events --no-default-features --features eventbridge
	cargo test --package aws_lambda_events --no-default-features --features firehose
	cargo test --package aws_lambda_events --no-default-features --features iam
	cargo test --package aws_lambda_events --no-default-features --features iot
	cargo test --package aws_lambda_events --no-default-features --features iot_1_click
	cargo test --package aws_lambda_events --no-default-features --features iot_button
	cargo test --package aws_lambda_events --no-default-features --features iot_deprecated
	cargo test --package aws_lambda_events --no-default-features --features kafka
	cargo test --package aws_lambda_events --no-default-features --features kinesis
	cargo test --package aws_lambda_events --no-default-features --features kinesis_analytics
	cargo test --package aws_lambda_events --no-default-features --features lambda_function_urls
	cargo test --package aws_lambda_events --no-default-features --features lex
	cargo test --package aws_lambda_events --no-default-features --features rabbitmq
	cargo test --package aws_lambda_events --no-default-features --features s3
	cargo test --package aws_lambda_events --no-default-features --features s3_batch_job
	cargo test --package aws_lambda_events --no-default-features --features secretsmanager
	cargo test --package aws_lambda_events --no-default-features --features ses
	cargo test --package aws_lambda_events --no-default-features --features sns
	cargo test --package aws_lambda_events --no-default-features --features sqs
	cargo test --package aws_lambda_events --no-default-features --features streams
	cargo test --package aws_lambda_events --no-default-features --features vpc_lattice

fmt:
	cargo +nightly fmt --all

build-examples:
	HANDLERS_TO_BUILD=${HANDLERS_TO_BUILD} OUTPUT_DIR=${OUTPUT_DIR} ./scripts/build-examples.sh

nuke:
	docker kill $$(docker ps -q)

test-dockerized: build-examples
	@echo "Running dockerized tests locally..."
	
	@echo "Building base Docker image with RIE and custom entrypoint..."
	docker build \
	-t local/test-base \
	-f Dockerfile.test \
	.
	
	@echo "Setting up containerized test runner..."
	@if [ ! -d ".test-runner" ]; then \
		echo "Cloning containerized-test-runner-for-aws-lambda..."; \
		git clone --quiet https://github.com/aws/containerized-test-runner-for-aws-lambda.git .test-runner; \
	fi
	@echo "Building test runner Docker image..."
	@docker build -t test-runner:local -f .test-runner/Dockerfile .test-runner
	
	@echo "Running tests in Docker..."
	@docker run --rm \
		-e INPUT_SUITE_FILE_ARRAY='["./test/dockerized/suites/*.json"]' \
		-e DOCKER_IMAGE_NAME=local/test-base \
		-e TASK_FOLDER=./test/dockerized/tasks \
		-e GITHUB_WORKSPACE=/workspace \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-v "$(CURDIR):/workspace" \
		-w /workspace \
		test-runner:local

test-rie:
	HANDLER="$(HANDLER)" ./scripts/test-rie.sh

# Run RIE in Lambda Managed Instance (LMI) mode with concurrent polling.
test-rie-lmi:
	RIE_MAX_CONCURRENCY=$(RIE_MAX_CONCURRENCY) HANDLER="$(HANDLER)" ./scripts/test-rie.sh $(EXAMPLE)

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@echo '  pr-check              Run pre-commit checks (fmt, clippy, tests)'
	@echo '  integration-tests     Build and run AWS integration tests'
	@echo '  check-event-features  Test individual event features'
	@echo '  fmt                   Format code with cargo fmt'
	@echo '  build-examples        Build example Lambda functions'
	@echo '                        Usage: EXAMPLES="basic-lambda" OUTPUT_DIR=/make build-examples'
	@echo '  test-rie              Test Lambda with Runtime Interface Emulator'
	@echo '                        Usage: HANDLERS_TO_BUILD="basic-lambda basic-sqs" make test-rie'
	@echo '                        Usage: HANDLERS_TO_BUILD="basic-lambda" HANDLER="basic-lambda" make test-rie'
	@echo '  test-rie-lmi          Test RIE in Lambda Managed Instance mode'
	@echo '                        Usage: RIE_MAX_CONCURRENCY=4 HANDLERS_TO_BUILD="basic-lambda-concurrent" make test-rie-lmi'
	@echo '  test-dockerized       Run dockerized test harness'
	@echo '  nuke                  Kill all running Docker containers'
	@echo ''
	@echo 'Environment variables:'
	@echo '  EXAMPLES              Space-separated list of examples to build (for build-examples)'
	@echo '  HANDLERS_TO_BUILD     Space-separated list of handlers to build for RIE (for test-rie)'
	@echo '  HANDLER               Specific handler to run (defaults to first in HANDLERS_TO_BUILD)'
	@echo '  OUTPUT_DIR            Directory for built binaries (default: /tmp/var-task for build-examples, /var/task for Docker)'
	@echo '  RIE_MAX_CONCURRENCY   Max concurrent Lambda invocations for LMI mode (for test-rie-lmi)'
