//! This is an example function that serves a [tonic](https://docs.rs/tonic/latest/tonic/)
//! gRPC service through the Lambda Rust runtime HTTP support.
//!
//! Lambda function URLs and API Gateway don't support native gRPC, which needs HTTP/2
//! trailers. This example uses [gRPC-Web](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md)
//! instead: it works over HTTP/1.1 and carries trailers in the response body, so
//! `tonic_web::GrpcWebLayer` translates it to regular gRPC for the tonic service.
//!
//! The tonic routes are converted into an axum `Router`, which implements `tower::Service`
//! and can be passed straight to `lambda_http::run`.
use lambda_http::{run, tracing, Error};
use tonic::{service::Routes, Request, Response, Status};
use tonic_web::GrpcWebLayer;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

use hello_world::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};

#[derive(Default)]
pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let app = Routes::new(GreeterServer::new(MyGreeter))
        .into_axum_router()
        .layer(GrpcWebLayer::new());

    run(app).await
}
