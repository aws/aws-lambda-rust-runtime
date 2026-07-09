# Container image for the basic-snapstart example.
#
# SnapStart for functions packaged as OCI images requires a custom base image
# whose runtime implements the restore lifecycle — which `lambda_runtime` now
# does. This image builds the example as the Lambda `bootstrap` on top of the
# AWS-provided base (`provided:al2023`), which supports SnapStart.
#
# Build from the REPOSITORY ROOT (the example uses path dependencies on the
# workspace crates, so the build context must include them):
#
#   docker build -f examples/basic-snapstart/Dockerfile -t basic-snapstart .
#
# Then push to ECR and create the function with SnapStart enabled
# (PublishedVersions) and the architecture matching your build host.

# ---- build stage ----
FROM public.ecr.aws/lambda/provided:al2023 AS builder

RUN dnf install -y gcc && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy only the workspace crates this example depends on via path, then the
# example itself: lambda_runtime -> lambda_runtime_api_client.
COPY Cargo.* /build/
COPY lambda-runtime /build/lambda-runtime
COPY lambda-runtime-api-client /build/lambda-runtime-api-client
COPY examples/basic-snapstart /build/examples/basic-snapstart

WORKDIR /build/examples/basic-snapstart
RUN cargo build --release

# ---- runtime stage ----
FROM public.ecr.aws/lambda/provided:al2023

COPY --from=builder /build/examples/basic-snapstart/target/release/basic-snapstart ${LAMBDA_RUNTIME_DIR}/bootstrap

ENTRYPOINT [ "/var/runtime/bootstrap" ]
