# AWS Lambda Function example

This example serves a [tonic] gRPC service from Lambda.

Lambda function URLs and API Gateway don't support native gRPC, because it needs HTTP/2 trailers. The function speaks
[gRPC-Web] instead, which works over HTTP/1.1. Clients must use a gRPC-Web transport, such as [`tonic-web`] for Rust,
[`grpc-web`] or [Connect] for browsers, or [`grpcurl`] through a gRPC-Web proxy.

If you use an API Gateway REST API, add `application/grpc-web+proto` to its binary media types. Function URLs and HTTP
APIs need no extra setup.

The build script compiles `proto/helloworld.proto` with a vendored `protoc`, so you don't need to install it.

## Build & Deploy

1. Install [cargo-lambda]
2. Build the function with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with `cargo lambda deploy`. It creates an execution role with
   `AWSLambdaBasicExecutionRole`; pass `--iam-role YOUR_ROLE` to use an existing one.

## Call the function

Create a function URL that requires IAM authentication. Don't use `cargo lambda deploy --enable-function-url`, which
creates a URL that anyone can call.

```sh
aws lambda create-function-url-config --function-name http-tonic --auth-type AWS_IAM
```

Send a SigV4-signed gRPC-Web request for `SayHello { name: "Lambda" }` with curl (7.75 or later):

```sh
printf '\x00\x00\x00\x00\x08\x0a\x06Lambda' > request.bin
eval "$(aws configure export-credentials --format env)"
curl -s -X POST "https://<url-id>.lambda-url.<region>.on.aws/helloworld.Greeter/SayHello" \
  --aws-sigv4 "aws:amz:<region>:lambda" \
  --user "$AWS_ACCESS_KEY_ID:$AWS_SECRET_ACCESS_KEY" \
  -H "x-amz-security-token: $AWS_SESSION_TOKEN" \
  -H "content-type: application/grpc-web+proto" \
  -H "x-grpc-web: 1" \
  --data-binary @request.bin --output - | xxd
```

The response body holds a data frame with `Hello Lambda!` followed by a trailer frame with `grpc-status:0`.

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`

[tonic]: https://github.com/hyperium/tonic
[gRPC-Web]: https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md
[`tonic-web`]: https://docs.rs/tonic-web
[`grpc-web`]: https://github.com/grpc/grpc-web
[Connect]: https://connectrpc.com/docs/web/
[`grpcurl`]: https://github.com/fullstorydev/grpcurl
[cargo-lambda]: https://github.com/cargo-lambda/cargo-lambda#installation
