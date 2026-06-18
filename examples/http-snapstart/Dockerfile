# Container image for the http-snapstart example.
#
# SnapStart for functions packaged as OCI images requires a custom base image
# whose runtime implements the restore lifecycle — which `lambda_runtime` now
# does. This image builds the example as the Lambda `bootstrap` on top of the
# AWS-provided base (`provided:al2023`), which supports SnapStart.
#
# Build from the REPOSITORY ROOT (the example uses path dependencies on the
# workspace crates, so the build context must include them):
#
#   docker build -f examples/http-snapstart/Dockerfile -t http-snapstart .
#
# Then push to ECR and create the function with SnapStart enabled
# (PublishedVersions) and the architecture matching your build host.

# ---- build stage ----
FROM public.ecr.aws/lambda/provided:al2023 AS builder

RUN dnf install -y gcc && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy only the workspace crates this example depends on via path, then the
# example itself: lambda_http -> lambda_runtime -> lambda_runtime_api_client,
# plus lambda-events (the aws_lambda_events crate used by lambda_http).
COPY Cargo.* /build/
COPY lambda-http /build/lambda-http
COPY lambda-runtime /build/lambda-runtime
COPY lambda-runtime-api-client /build/lambda-runtime-api-client
COPY lambda-events /build/lambda-events
COPY examples/http-snapstart /build/examples/http-snapstart

WORKDIR /build/examples/http-snapstart
RUN cargo build --release

# ---- runtime stage ----
FROM public.ecr.aws/lambda/provided:al2023

COPY --from=builder /build/examples/http-snapstart/target/release/http-snapstart ${LAMBDA_RUNTIME_DIR}/bootstrap

ENTRYPOINT [ "/var/runtime/bootstrap" ]
