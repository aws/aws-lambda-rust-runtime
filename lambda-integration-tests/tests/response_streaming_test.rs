#[test]
fn test_streamed_response_should_return_prelude_and_body() {
    let test_endpoint = std::env::var("STREAM_TEST_ENDPOINT").expect("could not read STREAM_TEST_ENDPOINT");
    let secret_token = std::env::var("SECRET_TOKEN").expect("could not read SECRET_TOKEN");
    let client = reqwest::blocking::Client::new();
    let res = client
        .get(test_endpoint)
        .header("Authorization", secret_token)
        .send()
        .expect("could not send the request");
    let status = res.status();
    let headers = res.headers().clone();
    let body = res.text().expect("could not read the response body");

    // The status code and headers come from the metadata prelude of the streamed response.
    assert_eq!(status, 201, "unexpected status, body: {body}");
    assert_eq!(headers["content-type"], "text/plain");
    assert_eq!(headers["x-response-mode"], "streaming");
    assert_eq!(body, "Hello from a streamed response");
}
