use serde_json::json;

#[test]
fn test_tenant_id_functionality_in_production() {
    let function_name =
        std::env::var("TENANT_ID_TEST_FUNCTION").expect("TENANT_ID_TEST_FUNCTION environment variable not set");

    // Test with tenant ID
    let payload_with_tenant = json!({
        "test": "tenant_id_test",
        "message": "Testing with tenant ID"
    });

    let output = std::process::Command::new("aws")
        .args([
            "lambda",
            "invoke",
            "--function-name",
            &function_name,
            "--payload",
            &payload_with_tenant.to_string(),
            "--cli-binary-format",
            "raw-in-base64-out",
            "/tmp/tenant_response.json",
        ])
        .output()
        .expect("Failed to invoke Lambda function");

    assert!(
        output.status.success(),
        "Lambda invocation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read and verify response
    let response = std::fs::read_to_string("/tmp/tenant_response.json").expect("Failed to read response file");

    let response_json: serde_json::Value = serde_json::from_str(&response).expect("Failed to parse response JSON");

    // Verify the function executed successfully
    assert_eq!(response_json["statusCode"], 200);

    // Parse the body to check tenant_id field exists (even if null)
    let body: serde_json::Value =
        serde_json::from_str(response_json["body"].as_str().expect("Body should be a string"))
            .expect("Failed to parse body JSON");

    assert!(
        body.get("tenant_id").is_some(),
        "tenant_id field should be present in response"
    );
    assert!(body.get("request_id").is_some(), "request_id should be present");
    assert_eq!(body["message"], "Tenant ID test successful");

    println!("✅ Tenant ID production test passed");
    println!("Response: {}", serde_json::to_string_pretty(&response_json).unwrap());
}
