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
        // API Gateway caches the policy by token and uses it for every route of the API.
        // Thus the policy allows every route in the stage, not only the called method.
        let resource = stage_arn(&event.payload.method_arn).ok_or("unexpected method ARN")?;
        return Ok(allow(&resource));
    }
    panic!("token is not valid");
}

/// Returns the ARN for all methods in the stage of `method_arn`. A method ARN has the format
/// `arn:<partition>:execute-api:<region>:<account>:<api-id>/<stage>/<verb>/<path>`.
fn stage_arn(method_arn: &str) -> Option<String> {
    let mut parts = method_arn.splitn(3, '/');
    Some(format!("{}/{}/*/*", parts.next()?, parts.next()?))
}

fn allow(resource: &str) -> ApiGatewayCustomAuthorizerResponse {
    let stmt = {
        let mut statement = IamPolicyStatement::default();
        statement.action = vec!["execute-api:Invoke".to_string()];
        statement.resource = vec![resource.to_owned()];
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

#[cfg(test)]
mod tests {
    use super::stage_arn;

    #[test]
    fn stage_arn_allows_all_methods_in_the_stage() {
        assert_eq!(
            stage_arn("arn:aws:execute-api:us-west-2:123456789012:abc123/integ-test/GET/hello").as_deref(),
            Some("arn:aws:execute-api:us-west-2:123456789012:abc123/integ-test/*/*")
        );
        assert_eq!(stage_arn("arn:aws:execute-api:us-west-2:123456789012:abc123"), None);
    }
}
