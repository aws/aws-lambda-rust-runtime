use std::env;

use aws_lambda_events::{
    apigw::{ApiGatewayCustomAuthorizerPolicy, ApiGatewayCustomAuthorizerResponse},
    event::iam::IamPolicyStatement,
};
use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct APIGatewayCustomAuthorizerRequest {
    authorization_token: String,
    method_arn: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(
    event: LambdaEvent<APIGatewayCustomAuthorizerRequest>,
) -> Result<ApiGatewayCustomAuthorizerResponse, Error> {
    let expected_token = env::var("SECRET_TOKEN").expect("could not read the secret token");
    if event.payload.authorization_token == expected_token {
        return Ok(allow(&event.payload.method_arn));
    }
    panic!("token is not valid");
}

fn allow(method_arn: &str) -> ApiGatewayCustomAuthorizerResponse {
    let stmt = {
        let mut statement = IamPolicyStatement::default();
        statement.action = vec!["execute-api:Invoke".to_string()];
        statement.resource = vec![method_arn.to_owned()];
        statement.effect = aws_lambda_events::iam::IamPolicyEffect::Allow;
        statement.condition = None;
        #[cfg(feature = "catch-all-fields")]
        {
            statement.other = Default::default();
        }
        statement
    };
    let policy = {
        let mut policy = ApiGatewayCustomAuthorizerPolicy::default();
        policy.version = Some("2012-10-17".to_string());
        policy.statement = vec![stmt];
        #[cfg(feature = "catch-all-fields")]
        {
            policy.other = Default::default();
        }
        policy
    };
    let mut response = ApiGatewayCustomAuthorizerResponse::default();
    response.principal_id = Some("user".to_owned());
    response.policy_document = policy;
    response.context = json!({ "hello": "world" });
    response.usage_identifier_key = None;
    #[cfg(feature = "catch-all-fields")]
    {
        response.other = Default::default();
    }
    response
}
