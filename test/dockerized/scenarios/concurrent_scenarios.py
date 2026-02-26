"""
Multi-concurrency test scenarios for basic-lambda-concurrent.

The handler expects: { "command": "<string>" }
and responds with:   { "req_id": "<id>", "msg": "Command <command> executed." }
"""

import os
from containerized_test_runner.models import Request, ConcurrentTest

HANDLER = "basic-lambda-concurrent"
IMAGE = os.environ.get("TEST_IMAGE", "local/test-base")
DEFAULT_CONCURRENCY = 10


def _make_env(concurrency: int = DEFAULT_CONCURRENCY) -> dict:
    return {
        "_HANDLER": HANDLER,
        "AWS_LAMBDA_MAX_CONCURRENCY": str(concurrency),
        "AWS_LAMBDA_LOG_FORMAT": "JSON",
    }


def get_concurrent_scenarios():
    scenarios = []

    # Happy path: DEFAULT_CONCURRENCY unique commands all succeed concurrently
    batch = [
        Request(
            payload={"command": f"task-{i}"},
            assertions=[{"transform": "{msg: .msg}", "response": {"msg": f"Command task-{i} executed."}}],
        )
        for i in range(DEFAULT_CONCURRENCY)
    ]
    scenarios.append(ConcurrentTest(
        name="concurrent_happy_path",
        handler=HANDLER,
        environment_variables=_make_env(),
        request_batches=[batch],
        image=IMAGE,
    ))

    # Error isolation: N-1 failing requests + 1 valid — the valid one must still succeed
    mixed_batch = [
        Request(
            payload={"command": "fail"},
            assertions=[{"errorType": "HandlerError"}],
        )
        for _ in range(DEFAULT_CONCURRENCY - 1)
    ] + [
        Request(
            payload={"command": "survivor"},
            assertions=[{"transform": "{msg: .msg}", "response": {"msg": "Command survivor executed."}}],
        )
    ]
    scenarios.append(ConcurrentTest(
        name="concurrent_error_isolation",
        handler=HANDLER,
        environment_variables=_make_env(),
        request_batches=[mixed_batch],
        image=IMAGE,
    ))

    return scenarios
