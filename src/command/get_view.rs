use hyper;
use serde;
use serde_json;
use std;

use Error;
use ErrorResponse;
use IntoViewPath;
use ViewResult;
use client::ClientState;
use command::{self, Command, Request};
use error::EncodeErrorKind;
use json;

/// Command to run a view.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
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

    /// Set whether to run the `reduce` view function.
    pub fn reduce(mut self, v: bool) -> Self {
        self.reduce = Some(v);
        self
    }

    /// Set the minimum key for rows contained within the result.
    pub fn endkey(mut self, key: K) -> Self {
        self.endkey = Some(key);
        self
    }

    /// Set the maximum key for rows contained within the result.
    pub fn startkey(mut self, key: K) -> Self {
        self.startkey = Some(key);
        self
    }

    impl_command_public_methods!(ViewResult<K, V>);
}

impl<'a, P, K, V> Command for GetView<'a, P, K, V>
    where P: IntoViewPath,
          K: serde::Deserialize + serde::Serialize,
          V: serde::Deserialize
{
    type Output = ViewResult<K, V>;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {

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
                uri.set_query_from_pairs(query_pairs.iter()
                                                    .map(|&(k, ref v)| {
                                                        let x: (&str, &str) = (k, v);
                                                        x
                                                    }));
            }

            uri
        };

        let req = try!(Request::new(hyper::Get, uri)).accept_application_json();
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response, _: Self::State) -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                try!(command::content_type_must_be_application_json(&resp.headers));
                let view_result = try!(json::decode_json::<_, ViewResult<K, V>>(resp));
                Ok(view_result)
            }
            hyper::status::StatusCode::BadRequest => {
                Err(Error::BadRequest(try!(json::decode_json::<_, ErrorResponse>(resp))))
            }
            hyper::status::StatusCode::Unauthorized => {
                Err(Error::Unauthorized(Some(try!(json::decode_json::<_, ErrorResponse>(resp)))))
            }
            hyper::status::StatusCode::NotFound => {
                Err(Error::NotFound(Some(try!(json::decode_json::<_, ErrorResponse>(resp)))))
            }
            hyper::status::StatusCode::InternalServerError => {
                Err(Error::InternalServerError(try!(json::decode_json::<_, ErrorResponse>(resp))))
            }
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
