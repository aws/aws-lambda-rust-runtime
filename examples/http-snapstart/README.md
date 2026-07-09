# AWS Lambda SnapStart example (lambda_http)

This example shows how to use AWS Lambda **SnapStart** with the `lambda_http`
crate. It wraps a shared connection pool in a `SnapStartResource` and registers
it via the [`lambda_http::runtime`] helper, so the runtime drains the pool before
the VM snapshot and reconnects it after each restore. The HTTP handler keeps the
usual `lambda_http` request/response ergonomics.

SnapStart reduces cold-start latency by snapshotting the initialized execution
environment and restoring from it. Resources created during init (connections,
credentials, unique values) may be invalid after restore — `before_snapshot` /
`after_restore` are where you release and re-establish them. When SnapStart is
not enabled, the hooks are never called and the function behaves normally.

If you don't need custom hooks, plain `lambda_http::run(...)` already gets
SnapStart support for free: the runtime calls `/restore/next` and rebuilds its
internal RAPID connection pool on restore without any extra code.

## Why a container image?

SnapStart for functions packaged as OCI images requires a custom base image
whose runtime implements the restore lifecycle — which `lambda_runtime` (used by
`lambda_http`) now does. This example ships a `Dockerfile` that builds the binary
as the Lambda `bootstrap` on top of `public.ecr.aws/lambda/provided:al2023`,
which supports SnapStart.

## Build & Deploy

Build the image from the **repository root** (the example depends on the
workspace crates via path, so the build context must include them):

```sh
docker build -f examples/http-snapstart/Dockerfile -t http-snapstart .
```

Push it to ECR and create the function from the image:

```sh
aws ecr create-repository --repository-name http-snapstart
docker tag http-snapstart:latest <ACCOUNT>.dkr.ecr.<REGION>.amazonaws.com/http-snapstart:latest
docker push <ACCOUNT>.dkr.ecr.<REGION>.amazonaws.com/http-snapstart:latest

aws lambda create-function \
  --function-name http-snapstart \
  --package-type Image \
  --code ImageUri=<ACCOUNT>.dkr.ecr.<REGION>.amazonaws.com/http-snapstart:latest \
  --role <YOUR_EXECUTION_ROLE_ARN> \
  --snap-start ApplyOn=PublishedVersions
```

SnapStart applies to **published versions**, so publish a version to trigger
snapshot creation, then expose it behind a function URL or API Gateway and invoke
it (e.g. `?name=world`).

## Architecture

The compiled `bootstrap` is architecture-specific. Build on (or for) the same
architecture as the target function — `provided:al2023` is available for both
`x86_64` and `arm64`.
