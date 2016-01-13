use hyper;
use serde;
use serde_json;
use std;

use Error;
use ErrorResponse;
use IntoViewPath;
use ViewResult;
use client::ClientState;
use action::{self, Action, Request, Response};
use error::EncodeErrorKind;

/// Action to execute a view.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
///
/// * `Error::InternalServerError`: An error occurred when executing the view.
/// * `Error::NotFound`: The view does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct GetView<'a, P, K, V>
    where P: IntoViewPath,
          K: serde::Deserialize, // serialize needed for endkey and startkey
          V: serde::Deserialize
{
    client_state: &'a ClientState,
    path: P,

    reduce: Option<bool>,
    endkey: Option<K>,
    startkey: Option<K>,

    _phantom_key: std::marker::PhantomData<K>,
    _phantom_value: std::marker::PhantomData<V>,
}

impl<'a, P, K, V> GetView<'a, P, K, V>
    where P: IntoViewPath,
          K: serde::Deserialize + serde::Serialize,
          V: serde::Deserialize
{
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        GetView {
            client_state: client_state,
            path: path,
            reduce: None,
            endkey: None,
            startkey: None,
            _phantom_key: std::marker::PhantomData,
            _phantom_value: std::marker::PhantomData,
        }
    }

    /// Sets whether to run the `reduce` view function.
    pub fn reduce(mut self, v: bool) -> Self {
        self.reduce = Some(v);
        self
    }

    /// Sets the minimum key for rows contained within the result.
    pub fn endkey(mut self, key: K) -> Self {
        self.endkey = Some(key);
        self
    }

    /// Sets the maximum key for rows contained within the result.
    pub fn startkey(mut self, key: K) -> Self {
        self.startkey = Some(key);
        self
    }

    impl_action_public_methods!(ViewResult<K, V>);
}

impl<'a, P, K, V> Action for GetView<'a, P, K, V>
    where P: IntoViewPath,
          K: serde::Deserialize + serde::Serialize,
          V: serde::Deserialize
{
    type Output = ViewResult<K, V>;

    fn make_request(self) -> Result<Request, Error> {

        let uri = {

            let mut uri = try!(self.path.into_view_path()).into_uri(self.client_state.uri.clone());

            {
                let mut query_pairs = Vec::<(&'static str, String)>::new();
                if self.reduce.is_some() {
                    match self.reduce.unwrap() {
                        true => query_pairs.push(("reduce", "true".to_string())),
                        false => query_pairs.push(("reduce", "false".to_string())),
                    };
                }
                if self.startkey.is_some() {
                    let x = try!(serde_json::to_string(&self.startkey.unwrap()).or_else(|e| {
                        Err(Error::Encode(EncodeErrorKind::Serde { cause: e }))
                    }));
                    query_pairs.push(("startkey", x));
                }
                if self.endkey.is_some() {
                    let x = try!(serde_json::to_string(&self.endkey.unwrap()).or_else(|e| {
                        Err(Error::Encode(EncodeErrorKind::Serde { cause: e }))
                    }));
                    query_pairs.push(("endkey", x));
                }
                if !query_pairs.is_empty() {
                    uri.set_query_from_pairs(query_pairs.iter()
                                                        .map(|&(k, ref v)| {
                                                            let x: (&str, &str) = (k, v);
                                                            x
                                                        }));
                }
            }

            uri
        };

        let request = Request::new(hyper::Get, uri).set_accept_application_json();
        Ok(request)
    }

    fn take_response<R: Response>(mut response: R) -> Result<Self::Output, Error> {
        match response.status() {
            hyper::status::StatusCode::Ok => {
                try!(response.content_type_must_be_application_json());
                let view_result = try!(response.decode_json::<ViewResult<K, V>>());
                Ok(view_result)
            }
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            hyper::status::StatusCode::Unauthorized => {
                Err(make_couchdb_error!(Unauthorized, response))
            }
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, response)),
            hyper::status::StatusCode::InternalServerError => {
                Err(make_couchdb_error!(InternalServerError, response))
            }
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;
    use serde_json;

    use DocumentId;
    use ViewPath;
    use ViewRow;
    use client::ClientState;
    use action::{Action, JsonResponse};
    use super::GetView;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetView::<_, String, i32>::new(&client_state, "/foo/_design/bar/_view/qux");
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/foo/_design/bar/_view/qux");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn make_request_reduce() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetView::<_, String, i32>::new(&client_state, "/foo/_design/bar/_view/qux")
                         .reduce(true);
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request,
                            "http://example.com:1234/foo/_design/bar/_view/qux?reduce=true");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn make_request_startkey() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let path = ViewPath::parse("/foo/_design/bar/_view/qux").unwrap();
        let action = GetView::<ViewPath, String, i32>::new(&client_state, path)
                         .startkey("baz".to_string());
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request,
                            "http://example.com:1234/foo/_design/bar/_view/qux?startkey=\"baz\"");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn make_request_endkey() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let path = ViewPath::parse("/foo/_design/bar/_view/qux").unwrap();
        let action = GetView::<ViewPath, String, i32>::new(&client_state, path)
                         .endkey("baz".to_string());
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request,
                            "http://example.com:1234/foo/_design/bar/_view/qux?endkey=\"baz\"");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn take_response_ok() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("offset", 17)
                         .insert_array("rows", |x| {
                             x.push_object(|x| {
                                 x.insert("id", "foo")
                                  .insert("key", "alpha")
                                  .insert("value", 5)
                             })
                         })
                         .insert("total_rows", 42)
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let got = GetView::<ViewPath, String, i32>::take_response(response).unwrap();
        assert_eq!(got.total_rows, Some(42));
        assert_eq!(got.offset, Some(17));
        assert_eq!(got.rows,
                   vec![{
                            let mut x = ViewRow::new(5);
                            x.id = Some(DocumentId::Normal("foo".into()));
                            x.key = Some("alpha".to_string());
                            x
                        }]);
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "bad_request")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = GetView::<ViewPath, String, i32>::take_response(response);
        expect_couchdb_error!(got, BadRequest);
    }

    #[test]
    fn take_response_internal_server_error() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "internal_server_error")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::InternalServerError, &source);
        let got = GetView::<ViewPath, String, i32>::take_response(response);
        expect_couchdb_error!(got, InternalServerError);
    }

    #[test]
    fn take_response_not_found() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "not_found")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::NotFound, &source);
        let got = GetView::<ViewPath, String, i32>::take_response(response);
        expect_couchdb_error!(got, NotFound);
    }

    #[test]
    fn take_response_unauthorized() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "unauthorized")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Unauthorized, &source);
        let got = GetView::<ViewPath, String, i32>::take_response(response);
        expect_couchdb_error!(got, Unauthorized);
    }
}
