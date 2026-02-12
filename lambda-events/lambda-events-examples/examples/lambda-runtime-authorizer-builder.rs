// Example showing how builders work with generic types and custom context structs
//
// Demonstrates:
// 1. Generic types (ApiGatewayV2CustomAuthorizerSimpleResponse<T>)
// 2. Custom context struct WITHOUT Default implementation
// 3. Custom context struct WITH Default implementation

use aws_lambda_events::event::apigw::{
    ApiGatewayV2CustomAuthorizerSimpleResponse, ApiGatewayV2CustomAuthorizerV2Request,
};
use lambda_runtime::{Error, LambdaEvent};
use serde::{Deserialize, Serialize};

// Custom context WITHOUT Default - requires builder pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWithoutDefault {
    pub user_id: String,
    pub api_key: String,
    pub permissions: Vec<String>,
}

// Custom context WITH Default - works both ways
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContextWithDefault {
    pub user_id: String,
    pub role: String,
}

// Handler using context WITHOUT Default - builder pattern required
pub async fn handler_without_default(
    _event: LambdaEvent<ApiGatewayV2CustomAuthorizerV2Request>,
) -> Result<ApiGatewayV2CustomAuthorizerSimpleResponse<ContextWithoutDefault>, Error> {
    let context = ContextWithoutDefault {
        user_id: "user-123".to_string(),
        api_key: "secret-key".to_string(),
        permissions: vec!["read".to_string()],
    };

    let response = ApiGatewayV2CustomAuthorizerSimpleResponse::builder()
        .is_authorized(true)
        .context(context)
        .build();

    Ok(response)
}

// Handler using context WITH Default - builder pattern still preferred
pub async fn handler_with_default(
    _event: LambdaEvent<ApiGatewayV2CustomAuthorizerV2Request>,
) -> Result<ApiGatewayV2CustomAuthorizerSimpleResponse<ContextWithDefault>, Error> {
    let context = ContextWithDefault {
        user_id: "user-456".to_string(),
        role: "admin".to_string(),
    };

    let response = ApiGatewayV2CustomAuthorizerSimpleResponse::builder()
        .is_authorized(true)
        .context(context)
        .build();

    Ok(response)
}

fn main() {
    // Example 1: Context WITHOUT Default
    let context_no_default = ContextWithoutDefault {
        user_id: "user-123".to_string(),
        api_key: "secret-key".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
    };

    let response1 = ApiGatewayV2CustomAuthorizerSimpleResponse::builder()
        .is_authorized(true)
        .context(context_no_default)
        .build();

    println!("Response with context WITHOUT Default:");
    println!("  User: {}", response1.context.user_id);
    println!("  Authorized: {}", response1.is_authorized);

    // Example 2: Context WITH Default
    let context_with_default = ContextWithDefault {
        user_id: "user-456".to_string(),
        role: "admin".to_string(),
    };

    let response2 = ApiGatewayV2CustomAuthorizerSimpleResponse::builder()
        .is_authorized(false)
        .context(context_with_default)
        .build();

    println!("\nResponse with context WITH Default:");
    println!("  User: {}", response2.context.user_id);
    println!("  Role: {}", response2.context.role);
    println!("  Authorized: {}", response2.is_authorized);
}
