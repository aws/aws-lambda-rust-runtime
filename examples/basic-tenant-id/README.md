# Basic Tenant ID Example

This example demonstrates how to access and use tenant ID information in a Lambda function.

## Key Features

- Extracts tenant ID from Lambda runtime headers
- Includes tenant ID in tracing logs
- Returns tenant ID in the response
- Handles cases where tenant ID is not provided

## Usage

The tenant ID is automatically extracted from the `lambda-runtime-aws-tenant-id` header and made available in the Lambda context.

```rust
async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, context) = event.into_parts();
    
    // Access tenant ID from context
    match &context.tenant_id {
        Some(tenant_id) => println!("Processing for tenant: {}", tenant_id),
        None => println!("No tenant ID provided"),
    }
    
    // ... rest of function logic
}
```

## Testing

You can test this function locally using cargo lambda:

```bash
cargo lambda invoke --data-ascii '{"test": "data"}'
```

The tenant ID will be None when testing locally unless you set up a mock runtime environment with the appropriate headers.
