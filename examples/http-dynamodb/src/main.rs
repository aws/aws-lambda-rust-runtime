use aws_sdk_dynamodb::Client;
use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use serde_dynamo::to_item;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub account_type: String,
    pub age: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
}

/// This is the main body for the function.
/// Write your code inside it.
/// You can see more examples in Runtime's repository:
/// - <https://github.com/aws/aws-lambda-rust-runtime/tree/main/examples>
async fn handle_request(db_client: &Client, event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
    let body = event.body();
    let s = std::str::from_utf8(body).expect("invalid utf-8 sequence");
    //Log into Cloudwatch
    tracing::info!(payload = %s, "JSON Payload received");

    //Serialze JSON into struct.
    //If JSON is incorrect, send back 400 with error.
    let item = match serde_json::from_str::<Item>(s) {
        Ok(item) => item,
        Err(err) => {
            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body(err.to_string().into())
                .map_err(Box::new)?;
            return Ok(resp);
        }
    };

    //Insert into the table.
    add_item(db_client, item.clone(), "lambda_dyno_example").await?;

    //Deserialize into json to return in the Response
    let j = serde_json::to_string(&item)?;

    //Send back a 200 - success
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(j.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    //Get config from environment.
    let config = aws_config::load_from_env().await;
    //Create the DynamoDB client.
    let client = Client::new(&config);

    run(service_fn(|event: Request| async {
        handle_request(&client, event).await
    }))
    .await
}

// Add an item to a table.
// snippet-start:[dynamodb.rust.add-item]
pub async fn add_item(client: &Client, item: Item, table: &str) -> Result<(), Error> {
    let item = to_item(item)?;

    let request = client.put_item().table_name(table).set_item(Some(item));

    tracing::info!("adding item to DynamoDB");

    let _resp = request.send().await?;

    Ok(())
}
