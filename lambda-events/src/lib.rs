#![deny(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub use http;
#[cfg(feature = "query_map")]
#[cfg_attr(docsrs, doc(cfg(feature = "query_map")))]
pub use query_map;

mod custom_serde;
/// Encodings used in AWS Lambda json event values.
pub mod encodings;
#[cfg(feature = "chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
pub mod time_window;

/// AWS Lambda event definitions.
pub mod event;

/// AWS Lambda event definitions for activemq.
#[cfg(feature = "activemq")]
#[cfg_attr(docsrs, doc(cfg(feature = "activemq")))]
pub use event::activemq;

/// AWS Lambda event definitions for alb.
#[cfg(feature = "alb")]
#[cfg_attr(docsrs, doc(cfg(feature = "alb")))]
pub use event::alb;

/// AWS Lambda event definitions for apigw.
#[cfg(feature = "apigw")]
#[cfg_attr(docsrs, doc(cfg(feature = "apigw")))]
pub use event::apigw;

/// AWS Lambda event definitions for appsync.
#[cfg(feature = "appsync")]
#[cfg_attr(docsrs, doc(cfg(feature = "appsync")))]
pub use event::appsync;

/// AWS Lambda event definitions for autoscaling.
#[cfg(feature = "autoscaling")]
#[cfg_attr(docsrs, doc(cfg(feature = "autoscaling")))]
pub use event::autoscaling;

/// AWS Lambda event definitions for chime_bot.
#[cfg(feature = "chime_bot")]
#[cfg_attr(docsrs, doc(cfg(feature = "chime_bot")))]
pub use event::chime_bot;

/// AWS Lambda event definitions for clientvpn.
#[cfg(feature = "clientvpn")]
#[cfg_attr(docsrs, doc(cfg(feature = "clientvpn")))]
pub use event::clientvpn;

/// AWS Lambda event definitions for cloudformation
#[cfg(feature = "cloudformation")]
#[cfg_attr(docsrs, doc(cfg(feature = "cloudformation")))]
pub use event::cloudformation;

/// AWS Lambda event definitions for CloudWatch alarms.
#[cfg(feature = "cloudwatch_alarms")]
#[cfg_attr(docsrs, doc(cfg(feature = "cloudwatch_alarms")))]
pub use event::cloudwatch_alarms;

/// AWS Lambda event definitions for CloudWatch events.
#[cfg(feature = "cloudwatch_events")]
#[cfg_attr(docsrs, doc(cfg(feature = "cloudwatch_events")))]
pub use event::cloudwatch_events;

/// AWS Lambda event definitions for cloudwatch_logs.
#[cfg(feature = "cloudwatch_logs")]
#[cfg_attr(docsrs, doc(cfg(feature = "cloudwatch_logs")))]
pub use event::cloudwatch_logs;

/// AWS Lambda event definitions for code_commit.
#[cfg(feature = "code_commit")]
#[cfg_attr(docsrs, doc(cfg(feature = "code_commit")))]
pub use event::code_commit;

/// AWS Lambda event definitions for codebuild.
#[cfg(feature = "codebuild")]
#[cfg_attr(docsrs, doc(cfg(feature = "codebuild")))]
pub use event::codebuild;

/// AWS Lambda event definitions for codedeploy.
#[cfg(feature = "codedeploy")]
#[cfg_attr(docsrs, doc(cfg(feature = "codedeploy")))]
pub use event::codedeploy;

/// AWS Lambda event definitions for codepipeline_cloudwatch.
#[cfg(feature = "codepipeline_cloudwatch")]
#[cfg_attr(docsrs, doc(cfg(feature = "codepipeline_cloudwatch")))]
pub use event::codepipeline_cloudwatch;

/// AWS Lambda event definitions for codepipeline_job.
#[cfg(feature = "codepipeline_job")]
#[cfg_attr(docsrs, doc(cfg(feature = "codepipeline_job")))]
pub use event::codepipeline_job;

/// AWS Lambda event definitions for cognito.
#[cfg(feature = "cognito")]
#[cfg_attr(docsrs, doc(cfg(feature = "cognito")))]
pub use event::cognito;

/// AWS Lambda event definitions for config.
#[cfg(feature = "config")]
#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
pub use event::config;

/// AWS Lambda event definitions for connect.
#[cfg(feature = "connect")]
#[cfg_attr(docsrs, doc(cfg(feature = "connect")))]
pub use event::connect;

/// AWS Lambda event definitions for dynamodb.
#[cfg(feature = "dynamodb")]
#[cfg_attr(docsrs, doc(cfg(feature = "dynamodb")))]
pub use event::dynamodb;

/// AWS Lambda event definitions for ecr_scan.
#[cfg(feature = "ecr_scan")]
#[cfg_attr(docsrs, doc(cfg(feature = "ecr_scan")))]
pub use event::ecr_scan;

/// AWS Lambda event definitions for firehose.
#[cfg(feature = "firehose")]
#[cfg_attr(docsrs, doc(cfg(feature = "firehose")))]
pub use event::firehose;

/// AWS Lambda event definitions for iam.
#[cfg(feature = "iam")]
#[cfg_attr(docsrs, doc(cfg(feature = "iam")))]
pub use event::iam;

/// AWS Lambda event definitions for iot.
#[cfg(feature = "iot")]
#[cfg_attr(docsrs, doc(cfg(feature = "iot")))]
pub use event::iot;

/// AWS Lambda event definitions for iot_1_click.
#[cfg(feature = "iot_1_click")]
#[cfg_attr(docsrs, doc(cfg(feature = "iot_1_click")))]
pub use event::iot_1_click;

/// AWS Lambda event definitions for iot_button.
#[cfg(feature = "iot_button")]
#[cfg_attr(docsrs, doc(cfg(feature = "iot_button")))]
pub use event::iot_button;

/// AWS Lambda event definitions for iot_deprecated.
#[cfg(feature = "iot_deprecated")]
#[cfg_attr(docsrs, doc(cfg(feature = "iot_deprecated")))]
pub use event::iot_deprecated;

/// AWS Lambda event definitions for kafka.
#[cfg(feature = "kafka")]
#[cfg_attr(docsrs, doc(cfg(feature = "kafka")))]
pub use event::kafka;

/// AWS Lambda event definitions for kinesis.
#[cfg(feature = "kinesis")]
#[cfg_attr(docsrs, doc(cfg(feature = "kinesis")))]
pub use event::kinesis;

/// AWS Lambda event definitions for kinesis_analytics.
#[cfg(feature = "kinesis_analytics")]
#[cfg_attr(docsrs, doc(cfg(feature = "kinesis_analytics")))]
pub use event::kinesis::analytics as kinesis_analytics;

/// AWS Lambda event definitions for lambda_function_urls.
#[cfg(feature = "lambda_function_urls")]
#[cfg_attr(docsrs, doc(cfg(feature = "lambda_function_urls")))]
pub use event::lambda_function_urls;

/// AWS Lambda event definitions for lex.
#[cfg(feature = "lex")]
#[cfg_attr(docsrs, doc(cfg(feature = "lex")))]
pub use event::lex;

/// AWS Lambda event definitions for rabbitmq.
#[cfg(feature = "rabbitmq")]
#[cfg_attr(docsrs, doc(cfg(feature = "rabbitmq")))]
pub use event::rabbitmq;

/// AWS Lambda event definitions for s3.
#[cfg(feature = "s3")]
#[cfg_attr(docsrs, doc(cfg(feature = "s3")))]
pub use event::s3;

/// AWS Lambda event definitions for s3_batch_job.
#[cfg(feature = "s3")]
#[cfg_attr(docsrs, doc(cfg(feature = "s3")))]
pub use event::s3::batch_job as s3_batch_job;

/// AWS Lambda event definitions for secretsmanager.
#[cfg(feature = "secretsmanager")]
#[cfg_attr(docsrs, doc(cfg(feature = "secretsmanager")))]
pub use event::secretsmanager;

/// AWS Lambda event definitions for ses.
#[cfg(feature = "ses")]
#[cfg_attr(docsrs, doc(cfg(feature = "ses")))]
pub use event::ses;

/// AWS Lambda event definitions for SNS.
#[cfg(feature = "sns")]
#[cfg_attr(docsrs, doc(cfg(feature = "sns")))]
pub use event::sns;

/// AWS Lambda event definitions for SQS.
#[cfg(feature = "sqs")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqs")))]
pub use event::sqs;

/// AWS Lambda event definitions for streams.
#[cfg(feature = "streams")]
#[cfg_attr(docsrs, doc(cfg(feature = "streams")))]
pub use event::streams;

/// AWS Lambda event definitions for documentdb.
#[cfg(feature = "documentdb")]
#[cfg_attr(docsrs, doc(cfg(feature = "documentdb")))]
pub use event::documentdb;

/// AWS Lambda event definitions for EventBridge.
#[cfg(feature = "eventbridge")]
#[cfg_attr(docsrs, doc(cfg(feature = "eventbridge")))]
pub use event::eventbridge;

/// AWS Lambda event definitions for VPC lattice.
#[cfg(feature = "vpc_lattice")]
#[cfg_attr(docsrs, doc(cfg(feature = "vpc_lattice")))]
pub use event::vpc_lattice;
