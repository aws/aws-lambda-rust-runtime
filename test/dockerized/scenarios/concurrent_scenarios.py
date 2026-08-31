"""Multi-concurrency test scenarios."""

import os
from containerized_test_runner.models import ConcurrentTest, Request

HANDLER = "basic-lambda-concurrent"
INVOCATION_ID_HANDLER = "invocation-id-concurrent"
IMAGE = os.environ.get("TEST_IMAGE", "local/test-base")
SAME_REQUEST_ID = "shared-request-id"
DEFAULT_CONCURRENCY = 10
TIMEOUT = 5


def _make_env(handler: str = HANDLER, concurrency: int = DEFAULT_CONCURRENCY) -> dict:
    return {
        "_HANDLER": handler,
        "AWS_LAMBDA_MAX_CONCURRENCY": str(concurrency),
        "AWS_LAMBDA_LOG_FORMAT": "JSON",
    }


def _invocation_id_env(
    handler: str = INVOCATION_ID_HANDLER,
    concurrency: int = DEFAULT_CONCURRENCY,
    timeout: int = TIMEOUT,
) -> dict:
    return _make_env(handler, concurrency) | {
        "AWS_LAMBDA_FUNCTION_TIMEOUT": str(timeout),
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


def get_invocation_id_scenarios():
    batches = [
        [Request.create(
            payload={"command": "invoke-A", "sleep": TIMEOUT + 2},
            assertions=[{"transform": ".errorType", "error": "Sandbox.Timedout"}],
            headers={"X-Amzn-RequestId": SAME_REQUEST_ID},
        )],
        [Request.create(
            payload={"command": "invoke-B", "sleep": TIMEOUT - 1},
            assertions=[{"response": {"from": "invoke-B"}}],
            headers={"X-Amzn-RequestId": SAME_REQUEST_ID},
        )],
    ]


    return [ConcurrentTest(
        name="invocation_id",
        handler=INVOCATION_ID_HANDLER,
        environment_variables=_invocation_id_env(handler=INVOCATION_ID_HANDLER, timeout=TIMEOUT),
        request_batches=batches,
        image=IMAGE,
    )]

