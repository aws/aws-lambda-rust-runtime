//! Extension methods for `http::Extensions` and `http::Request<T>` types

use aws_lambda_events::query_map::QueryMap;
use http::request::Parts;
use lambda_runtime::Context;

use crate::request::RequestContext;

/// ALB/API gateway pre-parsed http query string parameters
#[non_exhaustive]
#[derive(Clone)]
pub(crate) struct QueryStringParameters(pub(crate) QueryMap);

/// API gateway pre-extracted url path parameters
///
/// These will always be empty for ALB requests
#[non_exhaustive]
#[derive(Clone)]
pub(crate) struct PathParameters(pub(crate) QueryMap);

/// API gateway configured
/// [stage variables](https://docs.aws.amazon.com/apigateway/latest/developerguide/stage-variables.html)
///
/// These will always be empty for ALB requests
#[non_exhaustive]
#[derive(Clone)]
pub(crate) struct StageVariables(pub(crate) QueryMap);

/// ALB/API gateway raw http path without any stage information
#[non_exhaustive]
#[derive(Clone)]
pub(crate) struct RawHttpPath(pub(crate) String);

/// Extensions for [`lambda_http::Request`], `http::request::Parts`, and `http::Extensions` structs
/// that provide access to
/// [API gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format)
/// and [ALB](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/lambda-functions.html)
/// features.
///
/// [`lambda_http::Request`]: crate::Request
pub trait RequestExt {
    /// Return the raw http path for a request without any stage information.
    fn raw_http_path(&self) -> &str;

    /// Configures instance with the raw http path.
    fn with_raw_http_path<S>(self, path: S) -> Self
    where
        S: Into<String>;

    /// Return pre-parsed HTTP query string parameters, parameters
    /// provided after the `?` portion of a URL,
    /// associated with the API gateway request.
    ///
    /// The yielded value represents both single and multi-valued
    /// parameters alike. When multiple query string parameters with the same
    /// name are expected, use `query_string_parameters().all("many")` to
    /// retrieve them all.
    ///
    /// Having no query parameters will yield an empty `QueryMap`.
    fn query_string_parameters(&self) -> QueryMap;

    /// Return pre-parsed HTTP query string parameters, parameters
    /// provided after the `?` portion of a URL,
    /// associated with the API gateway request.
    ///
    /// The yielded value represents both single and multi-valued
    /// parameters alike. When multiple query string parameters with the same
    /// name are expected, use
    /// `query_string_parameters_ref().and_then(|params| params.all("many"))` to
    /// retrieve them all.
    ///
    /// Having no query parameters will yield `None`.
    fn query_string_parameters_ref(&self) -> Option<&QueryMap>;

    /// Configures instance with query string parameters
    ///
    /// This is intended for use in mock testing contexts.
    fn with_query_string_parameters<Q>(self, parameters: Q) -> Self
    where
        Q: Into<QueryMap>;

    /// Return pre-extracted path parameters, parameter provided in URL placeholders
    /// `/foo/{bar}/baz/{qux}`,
    /// associated with the API gateway request. Having no path parameters
    /// will yield an empty `QueryMap`.
    ///
    /// These will always be empty for ALB triggered requests.
    fn path_parameters(&self) -> QueryMap;

    /// Return pre-extracted path parameters, parameter provided in URL placeholders
    /// `/foo/{bar}/baz/{qux}`,
    /// associated with the API gateway request. Having no path parameters
    /// will yield `None`.
    ///
    /// These will always be `None` for ALB triggered requests.
    fn path_parameters_ref(&self) -> Option<&QueryMap>;

    /// Configures instance with path parameters
    ///
    /// This is intended for use in mock testing contexts.
    fn with_path_parameters<P>(self, parameters: P) -> Self
    where
        P: Into<QueryMap>;

    /// Return [stage variables](https://docs.aws.amazon.com/apigateway/latest/developerguide/stage-variables.html)
    /// associated with the API gateway request. Having no stage parameters
    /// will yield an empty `QueryMap`.
    ///
    /// These will always be empty for ALB triggered requests.
    fn stage_variables(&self) -> QueryMap;

    /// Return [stage variables](https://docs.aws.amazon.com/apigateway/latest/developerguide/stage-variables.html)
    /// associated with the API gateway request. Having no stage parameters
    /// will yield `None`.
    ///
    /// These will always be `None` for ALB triggered requests.
    fn stage_variables_ref(&self) -> Option<&QueryMap>;

    /// Configures instance with stage variables
    ///
    /// This is intended for use in mock testing contexts.
    fn with_stage_variables<V>(self, variables: V) -> Self
    where
        V: Into<QueryMap>;

    /// Return request context data associated with the ALB or
    /// API gateway request
    fn request_context(&self) -> RequestContext;

    /// Return a reference to the request context data associated with the ALB or
    /// API gateway request
    fn request_context_ref(&self) -> Option<&RequestContext>;

    /// Configures instance with request context
    ///
    /// This is intended for use in mock testing contexts.
    fn with_request_context(self, context: RequestContext) -> Self;

    /// Return Lambda function context data associated with the
    /// request
    fn lambda_context(&self) -> Context;

    /// Return a reference to the Lambda function context data associated with the
    /// request
    fn lambda_context_ref(&self) -> Option<&Context>;

    /// Configures instance with lambda context
    fn with_lambda_context(self, context: Context) -> Self;
}

impl RequestExt for http::Extensions {
    fn raw_http_path(&self) -> &str {
        self.get::<RawHttpPath>()
            .map(|RawHttpPath(path)| path.as_str())
            .unwrap_or_default()
    }

    fn with_raw_http_path<S>(self, path: S) -> Self
    where
        S: Into<String>,
    {
        let mut s = self;
        s.insert(RawHttpPath(path.into()));
        s
    }

    fn query_string_parameters(&self) -> QueryMap {
        self.query_string_parameters_ref().cloned().unwrap_or_default()
    }

    fn query_string_parameters_ref(&self) -> Option<&QueryMap> {
        self.get::<QueryStringParameters>().and_then(
            |QueryStringParameters(params)| {
                if params.is_empty() {
                    None
                } else {
                    Some(params)
                }
            },
        )
    }

    fn with_query_string_parameters<Q>(self, parameters: Q) -> Self
    where
        Q: Into<QueryMap>,
    {
        let mut s = self;
        s.insert(QueryStringParameters(parameters.into()));
        s
    }

    fn path_parameters(&self) -> QueryMap {
        self.path_parameters_ref().cloned().unwrap_or_default()
    }

    fn path_parameters_ref(&self) -> Option<&QueryMap> {
        self.get::<PathParameters>().and_then(
            |PathParameters(params)| {
                if params.is_empty() {
                    None
                } else {
                    Some(params)
                }
            },
        )
    }

    fn with_path_parameters<P>(self, parameters: P) -> Self
    where
        P: Into<QueryMap>,
    {
        let mut s = self;
        s.insert(PathParameters(parameters.into()));
        s
    }

    fn stage_variables(&self) -> QueryMap {
        self.stage_variables_ref().cloned().unwrap_or_default()
    }

    fn stage_variables_ref(&self) -> Option<&QueryMap> {
        self.get::<StageVariables>()
            .and_then(|StageVariables(vars)| if vars.is_empty() { None } else { Some(vars) })
    }

    fn with_stage_variables<V>(self, variables: V) -> Self
    where
        V: Into<QueryMap>,
    {
        let mut s = self;
        s.insert(StageVariables(variables.into()));
        s
    }

    fn request_context(&self) -> RequestContext {
        self.request_context_ref()
            .cloned()
            .expect("Request did not contain a request context")
    }

    fn request_context_ref(&self) -> Option<&RequestContext> {
        self.get::<RequestContext>()
    }

    fn with_request_context(self, context: RequestContext) -> Self {
        let mut s = self;
        s.insert(context);
        s
    }

    fn lambda_context(&self) -> Context {
        self.lambda_context_ref()
            .cloned()
            .expect("Request did not contain a lambda context")
    }

    fn lambda_context_ref(&self) -> Option<&Context> {
        self.get::<Context>()
    }

    fn with_lambda_context(self, context: Context) -> Self {
        let mut s = self;
        s.insert(context);
        s
    }
}

impl RequestExt for Parts {
    fn raw_http_path(&self) -> &str {
        self.extensions.raw_http_path()
    }

    fn with_raw_http_path<S>(self, path: S) -> Self
    where
        S: Into<String>,
    {
        let mut s = self;
        s.extensions = s.extensions.with_raw_http_path(path);

        s
    }

    fn query_string_parameters(&self) -> QueryMap {
        self.extensions.query_string_parameters()
    }

    fn query_string_parameters_ref(&self) -> Option<&QueryMap> {
        self.extensions.query_string_parameters_ref()
    }

    fn with_query_string_parameters<Q>(self, parameters: Q) -> Self
    where
        Q: Into<QueryMap>,
    {
        let mut s = self;
        s.extensions = s.extensions.with_query_string_parameters(parameters);

        s
    }

    fn path_parameters(&self) -> QueryMap {
        self.extensions.path_parameters()
    }

    fn path_parameters_ref(&self) -> Option<&QueryMap> {
        self.extensions.path_parameters_ref()
    }

    fn with_path_parameters<P>(self, parameters: P) -> Self
    where
        P: Into<QueryMap>,
    {
        let mut s = self;
        s.extensions = s.extensions.with_path_parameters(parameters);

        s
    }

    fn stage_variables(&self) -> QueryMap {
        self.extensions.stage_variables()
    }

    fn stage_variables_ref(&self) -> Option<&QueryMap> {
        self.extensions.stage_variables_ref()
    }

    fn with_stage_variables<V>(self, variables: V) -> Self
    where
        V: Into<QueryMap>,
    {
        let mut s = self;
        s.extensions = s.extensions.with_stage_variables(variables);

        s
    }

    fn request_context(&self) -> RequestContext {
        self.extensions.request_context()
    }

    fn request_context_ref(&self) -> Option<&RequestContext> {
        self.extensions.request_context_ref()
    }

    fn with_request_context(self, context: RequestContext) -> Self {
        let mut s = self;
        s.extensions = s.extensions.with_request_context(context);

        s
    }

    fn lambda_context(&self) -> Context {
        self.extensions.lambda_context()
    }

    fn lambda_context_ref(&self) -> Option<&Context> {
        self.extensions.lambda_context_ref()
    }

    fn with_lambda_context(self, context: Context) -> Self {
        let mut s = self;
        s.extensions = s.extensions.with_lambda_context(context);

        s
    }
}

fn map_req_ext<B, F>(req: http::Request<B>, f: F) -> http::Request<B>
where
    F: FnOnce(http::Extensions) -> http::Extensions,
{
    let (mut parts, body) = req.into_parts();
    parts.extensions = (f)(parts.extensions);

    http::Request::from_parts(parts, body)
}

impl<B> RequestExt for http::Request<B> {
    fn raw_http_path(&self) -> &str {
        self.extensions().raw_http_path()
    }

    fn with_raw_http_path<S>(self, path: S) -> Self
    where
        S: Into<String>,
    {
        map_req_ext(self, |ext| ext.with_raw_http_path(path))
    }

    fn query_string_parameters(&self) -> QueryMap {
        self.extensions().query_string_parameters()
    }

    fn query_string_parameters_ref(&self) -> Option<&QueryMap> {
        self.extensions().query_string_parameters_ref()
    }

    fn with_query_string_parameters<Q>(self, parameters: Q) -> Self
    where
        Q: Into<QueryMap>,
    {
        map_req_ext(self, |ext| ext.with_query_string_parameters(parameters))
    }

    fn path_parameters(&self) -> QueryMap {
        self.extensions().path_parameters()
    }

    fn path_parameters_ref(&self) -> Option<&QueryMap> {
        self.extensions().path_parameters_ref()
    }

    fn with_path_parameters<P>(self, parameters: P) -> Self
    where
        P: Into<QueryMap>,
    {
        map_req_ext(self, |ext| ext.with_path_parameters(parameters))
    }

    fn stage_variables(&self) -> QueryMap {
        self.extensions().stage_variables()
    }

    fn stage_variables_ref(&self) -> Option<&QueryMap> {
        self.extensions().stage_variables_ref()
    }

    fn with_stage_variables<V>(self, variables: V) -> Self
    where
        V: Into<QueryMap>,
    {
        map_req_ext(self, |ext| ext.with_stage_variables(variables))
    }

    fn request_context(&self) -> RequestContext {
        self.extensions().request_context()
    }

    fn request_context_ref(&self) -> Option<&RequestContext> {
        self.extensions().request_context_ref()
    }

    fn with_request_context(self, context: RequestContext) -> Self {
        map_req_ext(self, |ext| ext.with_request_context(context))
    }

    fn lambda_context(&self) -> Context {
        self.extensions().lambda_context()
    }

    fn lambda_context_ref(&self) -> Option<&Context> {
        self.extensions().lambda_context_ref()
    }

    fn with_lambda_context(self, context: Context) -> Self {
        map_req_ext(self, |ext| ext.with_lambda_context(context))
    }
}

#[cfg(test)]
mod tests {
    use aws_lambda_events::query_map::QueryMap;
    use http::Extensions;

    use crate::Request;

    use super::RequestExt;

    #[test]
    fn extensions_can_mock_query_string_parameters_ext() {
        let ext = Extensions::default();
        assert_eq!(ext.query_string_parameters_ref(), None);
        assert_eq!(ext.query_string_parameters(), QueryMap::default());

        let mocked: QueryMap = hashmap! {
            "foo".into() => vec!["bar".into()]
        }
        .into();

        let ext = ext.with_query_string_parameters(mocked.clone());
        assert_eq!(ext.query_string_parameters_ref(), Some(&mocked));
        assert_eq!(ext.query_string_parameters(), mocked);
    }

    #[test]
    fn parts_can_mock_query_string_parameters_ext() {
        let (parts, _) = Request::default().into_parts();
        assert_eq!(parts.query_string_parameters_ref(), None);
        assert_eq!(parts.query_string_parameters(), QueryMap::default());

        let mocked: QueryMap = hashmap! {
            "foo".into() => vec!["bar".into()]
        }
        .into();

        let parts = parts.with_query_string_parameters(mocked.clone());
        assert_eq!(parts.query_string_parameters_ref(), Some(&mocked));
        assert_eq!(parts.query_string_parameters(), mocked);
    }

    #[test]
    fn requests_can_mock_query_string_parameters_ext() {
        let request = Request::default();
        assert_eq!(request.query_string_parameters_ref(), None);
        assert_eq!(request.query_string_parameters(), QueryMap::default());

        let mocked: QueryMap = hashmap! {
            "foo".into() => vec!["bar".into()]
        }
        .into();

        let request = request.with_query_string_parameters(mocked.clone());
        assert_eq!(request.query_string_parameters_ref(), Some(&mocked));
        assert_eq!(request.query_string_parameters(), mocked);
    }

    #[test]
    fn extensions_can_mock_path_parameters_ext() {
        let ext = Extensions::default();
        assert_eq!(ext.path_parameters_ref(), None);
        assert_eq!(ext.path_parameters(), QueryMap::default());

        let mocked: QueryMap = hashmap! {
            "foo".into() => vec!["bar".into()]
        }
        .into();

        let ext = ext.with_path_parameters(mocked.clone());
        assert_eq!(ext.path_parameters_ref(), Some(&mocked));
        assert_eq!(ext.path_parameters(), mocked);
    }

    #[test]
    fn parts_can_mock_path_parameters_ext() {
        let (parts, _) = Request::default().into_parts();
        assert_eq!(parts.path_parameters_ref(), None);
        assert_eq!(parts.path_parameters(), QueryMap::default());

        let mocked: QueryMap = hashmap! {
            "foo".into() => vec!["bar".into()]
        }
        .into();

        let parts = parts.with_path_parameters(mocked.clone());
        assert_eq!(parts.path_parameters_ref(), Some(&mocked));
        assert_eq!(parts.path_parameters(), mocked);
    }

    #[test]
    fn requests_can_mock_path_parameters_ext() {
        let request = Request::default();
        assert_eq!(request.path_parameters_ref(), None);
        assert_eq!(request.path_parameters(), QueryMap::default());

        let mocked: QueryMap = hashmap! {
            "foo".into() => vec!["bar".into()]
        }
        .into();

        let request = request.with_path_parameters(mocked.clone());
        assert_eq!(request.path_parameters_ref(), Some(&mocked));
        assert_eq!(request.path_parameters(), mocked);
    }

    #[test]
    fn extensions_can_mock_stage_variables_ext() {
        let ext = Extensions::default();
        assert_eq!(ext.stage_variables_ref(), None);
        assert_eq!(ext.stage_variables(), QueryMap::default());

        let mocked: QueryMap = hashmap! {
            "foo".into() => vec!["bar".into()]
        }
        .into();

        let ext = ext.with_stage_variables(mocked.clone());
        assert_eq!(ext.stage_variables_ref(), Some(&mocked));
        assert_eq!(ext.stage_variables(), mocked);
    }

    #[test]
    fn parts_can_mock_stage_variables_ext() {
        let (parts, _) = Request::default().into_parts();
        assert_eq!(parts.stage_variables_ref(), None);
        assert_eq!(parts.stage_variables(), QueryMap::default());

        let mocked: QueryMap = hashmap! {
            "foo".into() => vec!["bar".into()]
        }
        .into();

        let parts = parts.with_stage_variables(mocked.clone());
        assert_eq!(parts.stage_variables_ref(), Some(&mocked));
        assert_eq!(parts.stage_variables(), mocked);
    }

    #[test]
    fn requests_can_mock_stage_variables_ext() {
        let request = Request::default();
        assert_eq!(request.stage_variables_ref(), None);
        assert_eq!(request.stage_variables(), QueryMap::default());

        let mocked: QueryMap = hashmap! {
            "foo".into() => vec!["bar".into()]
        }
        .into();

        let request = request.with_stage_variables(mocked.clone());
        assert_eq!(request.stage_variables_ref(), Some(&mocked));
        assert_eq!(request.stage_variables(), mocked);
    }

    #[test]
    fn extensions_can_mock_raw_http_path_ext() {
        let ext = Extensions::default().with_raw_http_path("/raw-path");
        assert_eq!("/raw-path", ext.raw_http_path());
    }

    #[test]
    fn parts_can_mock_raw_http_path_ext() {
        let (parts, _) = Request::default().into_parts();
        let parts = parts.with_raw_http_path("/raw-path");
        assert_eq!("/raw-path", parts.raw_http_path());
    }

    #[test]
    fn requests_can_mock_raw_http_path_ext() {
        let request = Request::default().with_raw_http_path("/raw-path");
        assert_eq!("/raw-path", request.raw_http_path());
    }
}
