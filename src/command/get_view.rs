use hyper;
use serde;
use serde_json;
use std;

use client::{self, ClientState};
use dbpath::DatabasePath;
use dbtype;
use error::{self, Error};
use transport::{self, Command, Request};
use viewpath::ViewPath;
use viewresult::ViewResult;

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
    #[doc(hidden)]
    pub fn new_get_view(client_state: &'a ClientState, path: ViewPath)
        -> Self
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
    type State = DatabasePath;

    fn make_request(self) -> Result<(Request, Self::State), Error> {

        let db_path = self.path.document_path().database_path().clone();

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
        Ok((req, db_path))
    }

    fn take_response(mut resp: hyper::client::Response, db_path: Self::State)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                let s = try!(client::read_json_response(&mut resp));
                let db_result = try!(client::decode_json::<dbtype::ViewResult<K, V>>(&s));
                Ok(ViewResult::from_db_view_result(&db_path, db_result))
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
