// Example showing how builders solve the Default trait requirement problem
// when using lambda_runtime with API Gateway custom authorizers
//
// ❌ OLD WAY (with Default requirement):
//    #[derive(Default)]
//    struct MyContext {
//        // Had to use Option just for Default
//        some_thing: Option<ThirdPartyThing>,
//    }
//
//    let mut output = Response::default();
//    output.is_authorized = true;
//    output.context = MyContext {
//        some_thing: Some(thing), // ❌ Unnecessary Some()
//    };
//
// ✅ NEW WAY (with Builder pattern):
//    struct MyContext {
//        // No Option needed!
//        some_thing: ThirdPartyThing,
//    }
//
//    let output = ResponseBuilder::default()
//        .is_authorized(true)
//        .context(context)
//        .build()?;
//
// Benefits:
// • No Option<T> wrapper for fields that always exist
// • Type-safe construction
// • Works seamlessly with lambda_runtime::LambdaEvent
// • Cleaner, more idiomatic Rust code

#[cfg(feature = "builders")]
use aws_lambda_events::event::apigw::{
    ApiGatewayV2CustomAuthorizerSimpleResponse,
    ApiGatewayV2CustomAuthorizerSimpleResponseBuilder,
    ApiGatewayV2CustomAuthorizerV2Request,
};
#[cfg(feature = "builders")]
use lambda_runtime::{Error, LambdaEvent};
#[cfg(feature = "builders")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "builders")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomeThirdPartyThingWithoutDefaultValue {
    pub api_key: String,
    pub endpoint: String,
    pub timeout_ms: u64,
}

// ❌ OLD WAY: Had to use Option to satisfy Default requirement
#[cfg(feature = "builders")]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MyContextOldWay {
    // NOT IDEAL: Need to wrap with Option just for Default
    some_thing_always_exists: Option<SomeThirdPartyThingWithoutDefaultValue>,
}

// ✅ NEW WAY: No Option needed with builder pattern!
#[cfg(feature = "builders")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyContext {
    // IDEAL: Can use the actual type directly!
    some_thing_always_exists: SomeThirdPartyThingWithoutDefaultValue,
    user_id: String,
    permissions: Vec<String>,
}

// ❌ OLD IMPLEMENTATION: Using Default
#[cfg(feature = "builders")]
pub async fn function_handler_old_way(
    _event: LambdaEvent<ApiGatewayV2CustomAuthorizerV2Request>,
) -> Result<ApiGatewayV2CustomAuthorizerSimpleResponse<MyContextOldWay>, Error> {
    let mut output: ApiGatewayV2CustomAuthorizerSimpleResponse<MyContextOldWay> =
        ApiGatewayV2CustomAuthorizerSimpleResponse::default();

    output.is_authorized = true;
    output.context = MyContextOldWay {
        // ❌ Had to wrap in Some() even though it always exists
        some_thing_always_exists: Some(SomeThirdPartyThingWithoutDefaultValue {
            api_key: "secret-key-123".to_string(),
            endpoint: "https://api.example.com".to_string(),
            timeout_ms: 5000,
        }),
    };

    Ok(output)
}

// ✅ NEW IMPLEMENTATION: Using Builder
#[cfg(feature = "builders")]
pub async fn function_handler(
    _event: LambdaEvent<ApiGatewayV2CustomAuthorizerV2Request>,
) -> Result<ApiGatewayV2CustomAuthorizerSimpleResponse<MyContext>, Error> {
    let context = MyContext {
        // ✅ No Option wrapper needed!
        some_thing_always_exists: SomeThirdPartyThingWithoutDefaultValue {
            api_key: "secret-key-123".to_string(),
            endpoint: "https://api.example.com".to_string(),
            timeout_ms: 5000,
        },
        user_id: "user-123".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
    };

    // ✅ Clean builder pattern - no Default required!
    let output = ApiGatewayV2CustomAuthorizerSimpleResponseBuilder::default()
        .is_authorized(true)
        .context(context)
        .build()
        .map_err(|e| format!("Failed to build response: {}", e))?;

    Ok(output)
}

#[cfg(feature = "builders")]
fn main() {
    let context = MyContext {
        some_thing_always_exists: SomeThirdPartyThingWithoutDefaultValue {
            api_key: "secret-key-123".to_string(),
            endpoint: "https://api.example.com".to_string(),
            timeout_ms: 5000,
        },
        user_id: "user-123".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
    };

    let response = ApiGatewayV2CustomAuthorizerSimpleResponseBuilder::<MyContext>::default()
        .is_authorized(true)
        .context(context)
        .build()
        .unwrap();

    println!("✅ Built authorizer response for user: {}", response.context.user_id);
    println!("   Authorized: {}", response.is_authorized);
    println!("   Permissions: {:?}", response.context.permissions);
    println!(
        "   Third-party endpoint: {}",
        response.context.some_thing_always_exists.endpoint
    );
}

#[cfg(not(feature = "builders"))]
fn main() {
    println!("This example requires the 'builders' feature to be enabled.");
    println!("Run with: cargo run --example lambda-runtime-authorizer-builder --features builders");
}
