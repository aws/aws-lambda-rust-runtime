use base64::prelude::*;
use serde_json::json;

fn function_name() -> String {
    std::env::var("W3C_TEST_FUNCTION").expect("W3C_TEST_FUNCTION environment variable not set")
}

/// Invoke the deployed Lambda function and return the parsed body JSON.
fn invoke(payload: &serde_json::Value, client_context_json: Option<&serde_json::Value>) -> serde_json::Value {
    let response_path = format!(
        "/tmp/w3c_response_{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let mut args: Vec<String> = vec![
        "lambda".into(),
        "invoke".into(),
        "--function-name".into(),
        function_name(),
        "--payload".into(),
        payload.to_string(),
        "--cli-binary-format".into(),
        "raw-in-base64-out".into(),
    ];

    if let Some(cc) = client_context_json {
        let encoded = BASE64_STANDARD.encode(cc.to_string());
        args.push("--client-context".into());
        args.push(encoded);
    }

    args.push(response_path.clone());

    let output = std::process::Command::new("aws")
        .args(&args)
        .output()
        .expect("Failed to invoke Lambda function");

    assert!(
        output.status.success(),
        "Lambda invocation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let response = std::fs::read_to_string(&response_path).expect("Failed to read response file");
    let response_json: serde_json::Value = serde_json::from_str(&response).expect("Failed to parse response JSON");

    assert_eq!(
        response_json["statusCode"],
        200,
        "handler returned non-200: {}",
        serde_json::to_string_pretty(&response_json).unwrap()
    );

    let body: serde_json::Value =
        serde_json::from_str(response_json["body"].as_str().expect("Body should be a string"))
            .expect("Failed to parse body JSON");

    let _ = std::fs::remove_file(&response_path);
    body
}

#[test]
fn test_w3c_propagation_in_production() {
    // 1. No client context — `context.w3c()` must be empty.
    let body = invoke(&json!({ "test": "w3c_no_client_context" }), None);
    assert_eq!(body["message"], "W3C test successful");
    assert_eq!(body["has_client_context"], false);
    assert_eq!(
        body["w3c"],
        json!({}),
        "w3c should be empty when no clientContext header is present, got: {}",
        body["w3c"]
    );

    // 2. Client context carrying `w3c` + a sibling `custom` field — `w3c()`
    //    must surface all three allowlisted fields, and the sibling `custom`
    //    must still be reachable on `context.client_context` (the `w3c` key
    //    was stripped during Context construction, not the whole object).
    let client_context = json!({
        "custom": { "source": "integ-test" },
        "w3c": {
            "traceparent": "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
            "tracestate": "rojo=00f067aa0ba902b7",
            "baggage": "userId=alice"
        }
    });

    let body = invoke(&json!({ "test": "w3c_with_client_context" }), Some(&client_context));
    assert_eq!(body["message"], "W3C test successful");
    assert_eq!(body["has_client_context"], true);
    assert_eq!(
        body["client_context_has_custom"], true,
        "sibling clientContext fields must still be reachable after w3c is stripped"
    );
    assert_eq!(
        body["w3c"]["traceparent"],
        "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
    );
    assert_eq!(body["w3c"]["tracestate"], "rojo=00f067aa0ba902b7");
    assert_eq!(body["w3c"]["baggage"], "userId=alice");

    println!("✅ W3C trace-context propagation test passed");
}
