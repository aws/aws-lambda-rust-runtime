fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a vendored `protoc` so the example builds without a system-wide install.
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);

    tonic_prost_build::configure()
        .build_client(false)
        .compile_protos(&["proto/helloworld.proto"], &["proto"])?;
    Ok(())
}
