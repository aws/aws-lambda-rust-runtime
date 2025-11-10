/// AWS Lambda event definitions for activemq.
#[cfg(feature = "activemq")]
#[cfg_attr(docsrs, doc(cfg(feature = "activemq")))]
pub mod activemq;

/// AWS Lambda event definitions for alb.
#[cfg(feature = "alb")]
#[cfg_attr(docsrs, doc(cfg(feature = "alb")))]
pub mod alb;
/// AWS Lambda event definitions for apigw.
#[cfg(feature = "apigw")]
#[cfg_attr(docsrs, doc(cfg(feature = "apigw")))]
pub mod apigw;

/// AWS Lambda event definitions for appsync.
#[cfg(feature = "appsync")]
#[cfg_attr(docsrs, doc(cfg(feature = "appsync")))]
pub mod appsync;

/// AWS Lambda event definitions for autoscaling.
#[cfg(feature = "autoscaling")]
#[cfg_attr(docsrs, doc(cfg(feature = "autoscaling")))]
pub mod autoscaling;

/// AWS Lambda event definitions for agent for amazon bedrock
#[cfg(feature = "bedrock_agent_runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "bedrock_agent_runtime")))]
pub mod bedrock_agent_runtime;

/// AWS Lambda event definitions for chime_bot.
#[cfg(feature = "chime_bot")]
#[cfg_attr(docsrs, doc(cfg(feature = "chime_bot")))]
pub mod chime_bot;

/// AWS Lambda event definitions for clientvpn.
#[cfg(feature = "clientvpn")]
#[cfg_attr(docsrs, doc(cfg(feature = "clientvpn")))]
pub mod clientvpn;

/// AWS Lambda event definitions for cloudformation.
#[cfg(feature = "cloudformation")]
#[cfg_attr(docsrs, doc(cfg(feature = "cloudformation")))]
pub mod cloudformation;

/// AWS Lambda event definitions for CloudWatch alarms.
#[cfg(feature = "cloudwatch_alarms")]
#[cfg_attr(docsrs, doc(cfg(feature = "cloudwatch_alarms")))]
pub mod cloudwatch_alarms;

/// AWS Lambda event definitions for CloudWatch events.
#[cfg(feature = "cloudwatch_events")]
#[cfg_attr(docsrs, doc(cfg(feature = "cloudwatch_events")))]
pub mod cloudwatch_events;

/// AWS Lambda event definitions for cloudwatch_logs.
#[cfg(feature = "cloudwatch_logs")]
#[cfg_attr(docsrs, doc(cfg(feature = "cloudwatch_logs")))]
pub mod cloudwatch_logs;

/// AWS Lambda event definitions for code_commit.
#[cfg(feature = "code_commit")]
#[cfg_attr(docsrs, doc(cfg(feature = "code_commit")))]
pub mod code_commit;

/// AWS Lambda event definitions for codebuild.
#[cfg(feature = "codebuild")]
#[cfg_attr(docsrs, doc(cfg(feature = "codebuild")))]
pub mod codebuild;

/// AWS Lambda event definitions for codedeploy.
#[cfg(feature = "codedeploy")]
#[cfg_attr(docsrs, doc(cfg(feature = "codedeploy")))]
pub mod codedeploy;

/// AWS Lambda event definitions for codepipeline_cloudwatch.
#[cfg(feature = "codepipeline_cloudwatch")]
#[cfg_attr(docsrs, doc(cfg(feature = "codepipeline_cloudwatch")))]
pub mod codepipeline_cloudwatch;

/// AWS Lambda event definitions for codepipeline_job.
#[cfg(feature = "codepipeline_job")]
#[cfg_attr(docsrs, doc(cfg(feature = "codepipeline_job")))]
pub mod codepipeline_job;

/// AWS Lambda event definitions for cognito.
#[cfg(feature = "cognito")]
#[cfg_attr(docsrs, doc(cfg(feature = "cognito")))]
pub mod cognito;

/// AWS Lambda event definitions for config.
#[cfg(feature = "config")]
#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
pub mod config;

/// AWS Lambda event definitions for connect.
#[cfg(feature = "connect")]
#[cfg_attr(docsrs, doc(cfg(feature = "connect")))]
pub mod connect;

/// AWS Lambda event definitions for dynamodb.
#[cfg(feature = "dynamodb")]
#[cfg_attr(docsrs, doc(cfg(feature = "dynamodb")))]
pub mod dynamodb;

/// AWS Lambda event definitions for ecr_scan.
#[cfg(feature = "ecr_scan")]
#[cfg_attr(docsrs, doc(cfg(feature = "ecr_scan")))]
pub mod ecr_scan;

/// AWS Lambda event definitions for firehose.
#[cfg(feature = "firehose")]
#[cfg_attr(docsrs, doc(cfg(feature = "firehose")))]
pub mod firehose;

/// AWS Lambda event definitions for iam.
#[cfg(feature = "iam")]
#[cfg_attr(docsrs, doc(cfg(feature = "iam")))]
pub mod iam;

/// AWS Lambda event definitions for iot.
#[cfg(feature = "iot")]
#[cfg_attr(docsrs, doc(cfg(feature = "iot")))]
pub mod iot;

/// AWS Lambda event definitions for iot_1_click.
#[cfg(feature = "iot_1_click")]
#[cfg_attr(docsrs, doc(cfg(feature = "iot_1_click")))]
pub mod iot_1_click;

/// AWS Lambda event definitions for iot_button.
#[cfg(feature = "iot_button")]
#[cfg_attr(docsrs, doc(cfg(feature = "iot_button")))]
pub mod iot_button;

/// AWS Lambda event definitions for iot_deprecated.
#[cfg(feature = "iot_deprecated")]
#[cfg_attr(docsrs, doc(cfg(feature = "iot_deprecated")))]
pub mod iot_deprecated;

/// AWS Lambda event definitions for kafka.
#[cfg(feature = "kafka")]
#[cfg_attr(docsrs, doc(cfg(feature = "kafka")))]
pub mod kafka;

/// AWS Lambda event definitions for kinesis.
#[cfg(feature = "kinesis")]
#[cfg_attr(docsrs, doc(cfg(feature = "kinesis")))]
pub mod kinesis;

/// AWS Lambda event definitions for lambda_function_urls.
#[cfg(feature = "lambda_function_urls")]
#[cfg_attr(docsrs, doc(cfg(feature = "lambda_function_urls")))]
pub mod lambda_function_urls;

/// AWS Lambda event definitions for lex.
#[cfg(feature = "lex")]
#[cfg_attr(docsrs, doc(cfg(feature = "lex")))]
pub mod lex;

/// AWS Lambda event definitions for rabbitmq.
#[cfg(feature = "rabbitmq")]
#[cfg_attr(docsrs, doc(cfg(feature = "rabbitmq")))]
pub mod rabbitmq;

/// AWS Lambda event definitions for s3.
#[cfg(feature = "s3")]
#[cfg_attr(docsrs, doc(cfg(feature = "s3")))]
pub mod s3;

/// AWS Lambda event definitions for secretsmanager.
#[cfg(feature = "secretsmanager")]
#[cfg_attr(docsrs, doc(cfg(feature = "secretsmanager")))]
pub mod secretsmanager;

/// AWS Lambda event definitions for ses.
#[cfg(feature = "ses")]
#[cfg_attr(docsrs, doc(cfg(feature = "ses")))]
pub mod ses;

/// AWS Lambda event definitions for SNS.
#[cfg(feature = "sns")]
#[cfg_attr(docsrs, doc(cfg(feature = "sns")))]
pub mod sns;

/// AWS Lambda event definitions for SQS.
#[cfg(feature = "sqs")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqs")))]
pub mod sqs;

/// AWS Lambda event definitions for streams.
#[cfg(feature = "streams")]
#[cfg_attr(docsrs, doc(cfg(feature = "streams")))]
pub mod streams;

// AWS Lambda event definitions for DocumentDB
#[cfg(feature = "documentdb")]
#[cfg_attr(docsrs, doc(cfg(feature = "documentdb")))]
pub mod documentdb;

/// AWS Lambda event definitions for EventBridge.
#[cfg(feature = "eventbridge")]
#[cfg_attr(docsrs, doc(cfg(feature = "eventbridge")))]
pub mod eventbridge;

/// AWS Lambda event definitions for VPC Lattice.
#[cfg(feature = "vpc_lattice")]
#[cfg_attr(docsrs, doc(cfg(feature = "vpc_lattice")))]
pub mod vpc_lattice;
