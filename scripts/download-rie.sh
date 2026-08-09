#!/bin/sh

set -eu

if [ "$#" -ne 5 ]; then
    echo "Usage: $0 TARGETARCH RIE_VERSION RIE_SHA256_AMD64 RIE_SHA256_ARM64 RIE_PATH" >&2
    exit 1
fi

TARGETARCH=$1
RIE_VERSION=$2
RIE_SHA256_AMD64=$3
RIE_SHA256_ARM64=$4
RIE_PATH=$5

case "${TARGETARCH}" in
    amd64)
        RIE_ASSET=aws-lambda-rie
        RIE_SHA256=${RIE_SHA256_AMD64}
        ;;
    arm64)
        RIE_ASSET=aws-lambda-rie-arm64
        RIE_SHA256=${RIE_SHA256_ARM64}
        ;;
    *)
        echo "Unsupported target architecture: ${TARGETARCH}" >&2
        exit 1
        ;;
esac

: "${RIE_PATH:?RIE_PATH must be set}"
RIE_TMP=$(mktemp)
trap 'rm -f "${RIE_TMP}"' EXIT

curl \
    --fail \
    --location \
    --silent \
    --show-error \
    --retry 3 \
    --retry-all-errors \
    --output "${RIE_TMP}" \
    "https://github.com/aws/aws-lambda-runtime-interface-emulator/releases/download/v${RIE_VERSION}/${RIE_ASSET}"

echo "${RIE_SHA256}  ${RIE_TMP}" | sha256sum --check --status
install -m 0755 "${RIE_TMP}" "${RIE_PATH}"
