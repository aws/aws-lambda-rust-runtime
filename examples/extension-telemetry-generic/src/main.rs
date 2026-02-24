use lambda_extension::{service_fn, tracing, Error, Extension, LambdaTelemetry, LambdaTelemetryRecord, SharedService};

async fn handler(events: Vec<LambdaTelemetry<serde_json::Value>>) -> Result<(), Error> {
    for event in events {
        match event.record {
            LambdaTelemetryRecord::Function(record) => tracing::info!("[logs] [function] {}", record),
            LambdaTelemetryRecord::Extension(record) => tracing::info!("[extension] [function] {}", record),
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
        .with_telemetry_record_type::<serde_json::Value>()
        .with_telemetry_processor(telemetry_processor)
        .run()
        .await?;

    Ok(())
}
