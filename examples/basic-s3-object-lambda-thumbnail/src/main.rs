use std::error;

use aws_lambda_events::s3::object_lambda::{GetObjectContext, S3ObjectLambdaEvent};
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use s3::{GetFile, SendFile};

mod s3;

/**
This s3 object lambda handler
    * downloads the asked file
    * creates a PNG thumbnail from it
    * forwards it to the browser
*/
pub(crate) async fn function_handler<T: SendFile + GetFile>(
    event: LambdaEvent<S3ObjectLambdaEvent>,
    size: u32,
    client: &T,
) -> Result<String, Box<dyn error::Error>> {
    tracing::info!("handler starts");

    let context: GetObjectContext = event.payload.get_object_context.unwrap();

    let route = context.output_route;
    let token = context.output_token;
    let s3_url = context.input_s3_url;

    tracing::info!("Route: {}, s3_url: {}", route, s3_url);

    let image = client.get_file(s3_url)?;
    tracing::info!("Image loaded. Length: {}", image.len());

    let thumbnail = get_thumbnail(image, size);
    tracing::info!("thumbnail created. Length: {}", thumbnail.len());

    client.send_file(route, token, thumbnail).await
}

#[cfg(not(test))]
fn get_thumbnail(vec: Vec<u8>, size: u32) -> Vec<u8> {
    let reader = std::io::Cursor::new(vec);
    let mut thumbnails = thumbnailer::create_thumbnails(
        reader,
        mime::IMAGE_PNG,
        [thumbnailer::ThumbnailSize::Custom((size, size))],
    )
    .unwrap();

    let thumbnail = thumbnails.pop().unwrap();
    let mut buf = std::io::Cursor::new(Vec::new());
    thumbnail.write_png(&mut buf).unwrap();

    buf.into_inner()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let shared_config = aws_config::load_from_env().await;
    let client = S3Client::new(&shared_config);
    let client_ref = &client;

    let func = service_fn(move |event| async move { function_handler(event, 128, client_ref).await });

    let _ = run(func).await;

    Ok(())
}

#[cfg(test)]
fn get_thumbnail(vec: Vec<u8>, _size: u32) -> Vec<u8> {
    let s = unsafe { std::str::from_utf8_unchecked(&vec) };

    match s {
        "IMAGE" => "THUMBNAIL".into(),
        _ => "Input is not IMAGE".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use aws_lambda_events::s3::object_lambda::{
        Configuration, HeadObjectContext, ListObjectsContext, ListObjectsV2Context, UserIdentity, UserRequest,
    };
    use lambda_runtime::{Context, LambdaEvent};
    use mockall::mock;
    use s3::{GetFile, SendFile};
    use serde_json::json;

    #[tokio::test]
    async fn response_is_good() {
        mock! {
            FakeS3Client {}

            impl GetFile for FakeS3Client {
                fn get_file(&self, url: String) -> Result<Vec<u8>, Box<dyn error::Error>>;
            }
            #[async_trait]
            impl SendFile for FakeS3Client {
                async fn send_file(&self, route: String, token: String, vec: Vec<u8>) -> Result<String, Box<dyn error::Error>>;
            }
        }

        let mut mock = MockFakeS3Client::new();

        mock.expect_get_file()
            .withf(|u| u.eq("S3_URL"))
            .returning(|_1| Ok("IMAGE".into()));

        mock.expect_send_file()
            .withf(|r, t, by| r.eq("O_ROUTE") && t.eq("O_TOKEN") && by == "THUMBNAIL".as_bytes())
            .returning(|_1, _2, _3| Ok("File sent.".to_string()));

        let payload = get_s3_event();
        let context = Context::default();
        let event = LambdaEvent { payload, context };

        let result = function_handler(event, 10, &mock).await.unwrap();

        assert_eq!(("File sent."), result);
    }

    fn get_s3_event() -> S3ObjectLambdaEvent {
        let mut event = S3ObjectLambdaEvent::default();
        event.x_amz_request_id = "ID".to_string();
        event.head_object_context = Some(HeadObjectContext::default());
        event.list_objects_context = Some(ListObjectsContext::default());
        event.get_object_context = Some({
            let mut context = GetObjectContext::default();
            context.input_s3_url = "S3_URL".to_string();
            context.output_route = "O_ROUTE".to_string();
            context.output_token = "O_TOKEN".to_string();
            context
        });
        event.list_objects_v2_context = Some(ListObjectsV2Context::default());
        event.protocol_version = "VERSION".to_string();
        event.user_identity = UserIdentity::default();
        event.user_request = UserRequest::default();
        event.configuration = {
            let mut configuration = Configuration::default();
            configuration.access_point_arn = "APRN".to_string();
            configuration.supporting_access_point_arn = "SAPRN".to_string();
            configuration.payload = json!(null);
            configuration
        };
        event
    }
}
