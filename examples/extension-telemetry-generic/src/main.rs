use lambda_extension::{
    service_fn, tracing, Error, Extension, GenericLambdaTelemetry, GenericLambdaTelemetryRecord, SharedService,
};

async fn handler(events: Vec<GenericLambdaTelemetry<serde_json::Value>>) -> Result<(), Error> {
    for event in events {
        match event.record {
            GenericLambdaTelemetryRecord::Function(record) => tracing::info!("[logs] [function] {}", record),
            GenericLambdaTelemetryRecord::Extension(record) => tracing::info!("[extension] [function] {}", record),
            _ => (),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let telemetry_processor = SharedService::new(service_fn(handler));

    Extension::new()
        .with_generic_telemetry_processor(telemetry_processor)
        .run()
        .await?;

    Ok(())
}
