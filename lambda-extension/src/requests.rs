use crate::{Error, LogBuffering};
use http::{Method, Request};
use lambda_runtime_api_client::{body::Body, build_request};
use serde::Serialize;

const EXTENSION_NAME_HEADER: &str = "Lambda-Extension-Name";
pub(crate) const EXTENSION_ID_HEADER: &str = "Lambda-Extension-Identifier";
const EXTENSION_ERROR_TYPE_HEADER: &str = "Lambda-Extension-Function-Error-Type";
const CONTENT_TYPE_HEADER_NAME: &str = "Content-Type";
const CONTENT_TYPE_HEADER_VALUE: &str = "application/json";

// Comma separated list of features the extension supports.
// `accountId` is currently the only supported feature.
const EXTENSION_ACCEPT_FEATURE: &str = "Lambda-Extension-Accept-Feature";
const EXTENSION_ACCEPT_FEATURE_VALUE: &str = "accountId";

pub(crate) fn next_event_request(extension_id: &str) -> Result<Request<Body>, Error> {
    let req = build_request()
        .method(Method::GET)
        .header(EXTENSION_ID_HEADER, extension_id)
        .uri("/2020-01-01/extension/event/next")
        .body(Body::empty())?;
    Ok(req)
}

pub(crate) fn register_request(extension_name: &str, events: &[&str]) -> Result<Request<Body>, Error> {
    let events = serde_json::json!({ "events": events });

    let req = build_request()
        .method(Method::POST)
        .uri("/2020-01-01/extension/register")
        .header(EXTENSION_NAME_HEADER, extension_name)
        .header(EXTENSION_ACCEPT_FEATURE, EXTENSION_ACCEPT_FEATURE_VALUE)
        .header(CONTENT_TYPE_HEADER_NAME, CONTENT_TYPE_HEADER_VALUE)
        .body(Body::from(serde_json::to_string(&events)?))?;

    Ok(req)
}

pub(crate) enum Api {
    LogsApi,
    TelemetryApi,
}

/// Environment variable Lambda sets to the function's initialization type
const AWS_LAMBDA_INITIALIZATION_TYPE: &str = "AWS_LAMBDA_INITIALIZATION_TYPE";
/// Value of `AWS_LAMBDA_INITIALIZATION_TYPE` for functions running on Lambda Managed Instances
const LAMBDA_MANAGED_INSTANCES_INIT_TYPE: &str = "lambda-managed-instances";

/// Telemetry API schema version for functions running on the default (on-demand) Lambda compute
const TELEMETRY_SCHEMA_VERSION_ON_DEMAND: &str = "2022-07-01";
/// Telemetry API schema version required by functions running on Lambda Managed Instances
const TELEMETRY_SCHEMA_VERSION_MANAGED_INSTANCES: &str = "2025-01-29";

/// Returns the Telemetry API schema version matching the function's initialization type
fn telemetry_schema_version(initialization_type: Option<&str>) -> &'static str {
    match initialization_type {
        Some(LAMBDA_MANAGED_INSTANCES_INIT_TYPE) => TELEMETRY_SCHEMA_VERSION_MANAGED_INSTANCES,
        _ => TELEMETRY_SCHEMA_VERSION_ON_DEMAND,
    }
}

impl Api {
    pub(crate) fn schema_version(&self) -> &str {
        match *self {
            Api::LogsApi => "2021-03-18",
            Api::TelemetryApi => {
                telemetry_schema_version(std::env::var(AWS_LAMBDA_INITIALIZATION_TYPE).ok().as_deref())
            }
        }
    }

    pub(crate) fn uri(&self) -> &str {
        match *self {
            Api::LogsApi => "/2020-08-15/logs",
            Api::TelemetryApi => "/2022-07-01/telemetry",
        }
    }
}

pub(crate) fn subscribe_request(
    api: Api,
    extension_id: &str,
    types: Option<&[&str]>,
    buffering: Option<LogBuffering>,
    port_number: u16,
) -> Result<Request<Body>, Error> {
    let types = types.unwrap_or(&["platform", "function"]);

    let data = serde_json::json!({
        "schemaVersion": api.schema_version(),
        "types": types,
        "buffering": buffering.unwrap_or_default(),
        "destination": {
            "protocol": "HTTP",
            "URI": format!("http://sandbox.localdomain:{port_number}"),
        }
    });

    let req = build_request()
        .method(Method::PUT)
        .uri(api.uri())
        .header(EXTENSION_ID_HEADER, extension_id)
        .header(CONTENT_TYPE_HEADER_NAME, CONTENT_TYPE_HEADER_VALUE)
        .body(Body::from(serde_json::to_string(&data)?))?;

    Ok(req)
}

/// Payload to send error information to the Extensions API.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRequest<'a> {
    /// Human readable error description
    pub error_message: &'a str,
    /// The type of error to categorize
    pub error_type: &'a str,
    /// The error backtrace
    pub stack_trace: Vec<&'a str>,
}

/// Create a new init error request to send to the Extensions API
pub fn init_error(
    extension_id: &str,
    error_type: &str,
    request: Option<ErrorRequest<'_>>,
) -> Result<Request<Body>, Error> {
    error_request("init", extension_id, error_type, request)
}

/// Create a new exit error request to send to the Extensions API
pub fn exit_error(
    extension_id: &str,
    error_type: &str,
    request: Option<ErrorRequest<'_>>,
) -> Result<Request<Body>, Error> {
    error_request("exit", extension_id, error_type, request)
}

fn error_request(
    error_type: &str,
    extension_id: &str,
    error_str: &str,
    request: Option<ErrorRequest<'_>>,
) -> Result<Request<Body>, Error> {
    let uri = format!("/2020-01-01/extension/{error_type}/error");

    let body = match request {
        None => Body::empty(),
        Some(err) => Body::from(serde_json::to_string(&err)?),
    };

    let req = build_request()
        .method(Method::POST)
        .uri(uri)
        .header(EXTENSION_ID_HEADER, extension_id)
        .header(EXTENSION_ERROR_TYPE_HEADER, error_str)
        .body(body)?;
    Ok(req)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_schema_version_defaults_to_on_demand() {
        assert_eq!(telemetry_schema_version(None), "2022-07-01");
        assert_eq!(telemetry_schema_version(Some("on-demand")), "2022-07-01");
        assert_eq!(telemetry_schema_version(Some("provisioned-concurrency")), "2022-07-01");
        assert_eq!(telemetry_schema_version(Some("snap-start")), "2022-07-01");
    }

    #[test]
    fn telemetry_schema_version_for_managed_instances() {
        assert_eq!(telemetry_schema_version(Some("lambda-managed-instances")), "2025-01-29");
    }
}
