use hyper;
use serde;
use serde_json;
use std;

use client::{self, ClientState};
use design::{ViewResult, ViewRow};
use error::{self, Error};
use transport::{self, Command, Request};
use viewpath::ViewPath;

#[doc(hidden)]
pub fn new_get_view<'a, K, V>(client_state: &'a ClientState, path: ViewPath)
    -> GetView<'a, K, V>
    where K: serde::Deserialize,
          V: serde::Deserialize
{
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

/// Command to run a view.
pub struct GetView<'a, K, V>
    where K: serde::Deserialize, // serialize needed for endkey and startkey
          V: serde::Deserialize
{
    client_state: &'a ClientState,
    path: ViewPath,

    reduce: Option<bool>,
    endkey: Option<K>,
    startkey: Option<K>,

    _phantom_key: std::marker::PhantomData<K>,
    _phantom_value: std::marker::PhantomData<V>,
}

impl<'a, K, V> GetView<'a, K, V>
    where K: serde::Deserialize + serde::Serialize,
          V: serde::Deserialize
{
    pub fn reduce(mut self, v: bool) -> Self {
        self.reduce = Some(v);
        self
    }

    pub fn endkey(mut self, key: K) -> Self
    {
        self.endkey = Some(key);
        self
    }

    pub fn startkey(mut self, key: K) -> Self
    {
        self.startkey = Some(key);
        self
    }

    /// Send the command request and wait for the response.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::InternalServerError`: An error occurred when executing the
    ///   view.
    /// * `Error::NotFound`: The view does not exist.
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<ViewResult<K, V>, Error>
    {
        transport::run_command(self)
    }
}

impl<'a, K, V> Command for GetView<'a, K, V>
    where K: serde::Deserialize + serde::Serialize,
          V: serde::Deserialize
{
    type Output = ViewResult<K, V>;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {

        let uri = {

            let mut uri = self.path.into_uri(self.client_state.uri.clone());

            {
                let mut query_pairs = Vec::<(&'static str, String)>::new();
                if self.reduce.is_some() {
                    match self.reduce.unwrap() {
                        true => query_pairs.push(("reduce", "true".to_string())),
                        false => query_pairs.push(("reduce", "false".to_string())),
                    };
                }
                if self.startkey.is_some() {
                    let x = try!(
                        serde_json::to_string(&self.startkey.unwrap())
                        .or_else(|e| { Err(Error::Encode{ cause: e }) }));
                    query_pairs.push(("startkey", x));
                }
                if self.endkey.is_some() {
                    let x = try!(
                        serde_json::to_string(&self.endkey.unwrap())
                        .or_else(|e| { Err(Error::Encode { cause: e } ) }));
                    query_pairs.push(("endkey", x));
                }
                uri.set_query_from_pairs(
                    query_pairs.iter()
                    .map(|&(k, ref v)| {
                        let x: (&str, &str) = (k, v);
                        x
                    })
                );
            }

            uri
        };

        let req = try!(Request::new(hyper::Get, uri))
            .accept_application_json();
        Ok((req, ()))
    }

    fn take_response(mut resp: hyper::client::Response, _state: Self::State)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                let s = try!(client::read_json_response(&mut resp));
                let mut resp_body = try!(client::decode_json::<serde_json::Value>(&s));
                match (|| {
                    let (total_rows, offset, mut raw_rows) = {
                        let mut dot = match resp_body.as_object_mut() {
                            None => { return None; },
                            Some(x) => x,
                        };
                        let total_rows = match dot.remove("total_rows") {
                            None => 0u64,
                            Some(x) => match x.as_u64() {
                                None => { return None; },
                                Some(x) => x,
                            }
                        };
                        let offset = match dot.remove("offset") {
                            None => 0u64,
                            Some(x) => match x.as_u64() {
                                None => { return None; },
                                Some(x) => x,
                            }
                        };
                        let raw_rows = match dot.remove("rows") {
                            None => { return None; },
                            Some(mut x) => match x.as_array_mut() {
                                None => { return None; },
                                Some(x) => std::mem::replace(x, Vec::<serde_json::Value>::new()),
                            }
                        };
                        (total_rows, offset, raw_rows)
                    };
                    let rows = raw_rows.iter_mut()
                        .map(|x| {
                            let v = std::mem::replace(x, serde_json::Value::Null);
                            serde_json::from_value::<ViewRow<K, V>>(v)
                        })
                        .take_while(|x| x.is_ok() )
                        .map(|x| x.unwrap() )
                        .collect::<Vec<ViewRow<K, V>>>();
                    if rows.len() != raw_rows.len() {
                        return None; // at least one element didn't deserialize
                    }
                    Some(
                        ViewResult::<K, V> {
                            total_rows: total_rows,
                            offset: offset,
                            rows: rows,
                        }
                    )
                })() {
                    None => Err(Error::UnexpectedContent { got: s } ),
                    Some(x) => Ok(x),
                }
            },
            hyper::status::StatusCode::BadRequest =>
                Err(error::new_because_invalid_request(&mut resp)),
            hyper::status::StatusCode::Unauthorized =>
                Err(error::new_because_unauthorized(&mut resp)),
            hyper::status::StatusCode::NotFound =>
                Err(error::new_because_not_found(&mut resp)),
            hyper::status::StatusCode::InternalServerError =>
                Err(error::new_because_internal_server_error(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}
