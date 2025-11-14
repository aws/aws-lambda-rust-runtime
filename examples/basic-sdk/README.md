
## basic-sdk example

This is an sample function that uses the [AWS SDK](https://github.com/aws/aws-sdk-rust) to
list the contents of an S3 bucket specified by the invoker.  It uses standard credentials as defined
in the function's execution role to make calls against S3.

### Running Locally
You can use `cargo lambda watch` to spin up a local version of the function.  This will automatically re-compile and restart
itself when it observes changes to the code.  If you invoke `watch` with no other context then the function will not have
the environment variables necessary to supply on SDK calls.  To get around this you can manually supply a credentials file
profile for the SDK to resolve and use in your function:
```
AWS_PROFILE=my-profile cargo lambda watch
```

### Invoking
You can invoke by simply leveraging `cargo lambda invoke` with the payload expected by the function handler.
```
cargo lambda invoke --data-ascii '{"bucket":"my-bucket"}'
```
