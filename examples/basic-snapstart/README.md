# AWS Lambda SnapStart example (lambda_runtime)

This example shows how to use AWS Lambda **SnapStart** with the `lambda_runtime`
crate. It implements the [`SnapStartResource`] trait on an `AppState` and
registers it on the runtime, so the runtime runs the resource's `before_snapshot`
hook before the VM snapshot and its `after_restore` hook after each restore.

SnapStart reduces cold-start latency by snapshotting the initialized execution
environment and restoring from it. Resources created during init (connections,
credentials, unique values) may be invalid after restore — `before_snapshot` /
`after_restore` are where you release and re-establish them. When SnapStart is
not enabled, the hooks are never called and the function behaves normally.

## Why a container image?

SnapStart for functions packaged as OCI images requires a custom base image
whose runtime implements the restore lifecycle — which `lambda_runtime` now does.
This example ships a `Dockerfile` that builds the binary as the Lambda
`bootstrap` on top of `public.ecr.aws/lambda/provided:al2023`, which supports
SnapStart.

## Build & Deploy

Build the image from the **repository root** (the example depends on the
workspace crates via path, so the build context must include them):

```sh
docker build -f examples/basic-snapstart/Dockerfile -t basic-snapstart .
```

Push it to ECR and create the function from the image:

```sh
aws ecr create-repository --repository-name basic-snapstart
docker tag basic-snapstart:latest <ACCOUNT>.dkr.ecr.<REGION>.amazonaws.com/basic-snapstart:latest
docker push <ACCOUNT>.dkr.ecr.<REGION>.amazonaws.com/basic-snapstart:latest

aws lambda create-function \
  --function-name basic-snapstart \
  --package-type Image \
  --code ImageUri=<ACCOUNT>.dkr.ecr.<REGION>.amazonaws.com/basic-snapstart:latest \
  --role <YOUR_EXECUTION_ROLE_ARN> \
  --snap-start ApplyOn=PublishedVersions
```

SnapStart applies to **published versions**, so publish a version to trigger
snapshot creation, then invoke that version (or an alias pointing at it):

```sh
aws lambda publish-version --function-name basic-snapstart
aws lambda invoke --function-name basic-snapstart:1 --payload '{"name":"world"}' out.json
```

## Architecture

The compiled `bootstrap` is architecture-specific. Build on (or for) the same
architecture as the target function — `provided:al2023` is available for both
`x86_64` and `arm64`.
