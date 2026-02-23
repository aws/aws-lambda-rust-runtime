# AWS Lambda Telemetry extension example

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the extension with `cargo lambda build --release --extension`
3. Deploy the extension as a layer with `cargo lambda deploy --extension`

The last command will give you an ARN for the extension layer that you can use in your functions.

## Build for ARM 64

Build the extension with `cargo lambda build --release --extension --arm64`
