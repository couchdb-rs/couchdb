use hyper;

use client::{self, ClientState};
use database;
use error::{self, Error};
use transport::{self, Command, Request};

#[doc(hidden)]
pub fn new_put_database<'a>(
    client_state: &'a ClientState,
    db_name: &'a str)
    -> PutDatabase<'a>
{
    PutDatabase {
        client_state: client_state,
        db_name: db_name,
    }
}

/// Command to create a database.
pub struct PutDatabase<'a> {
    client_state: &'a ClientState,
    db_name: &'a str,
}

impl<'a> PutDatabase<'a> {

    /// Send the command request and wait for the response.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::DatabaseExists`: The database already exists.
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<(), Error> {
        transport::run_command(self)
    }
}

impl<'a> Command for PutDatabase<'a> {

    type Output = ();

    fn make_request(self) -> Result<Request, Error> {
        let uri = database::new_uri(&self.client_state.uri, self.db_name);
        let req = try!(Request::new(hyper::method::Method::Put, uri))
            .accept_application_json();
        Ok(req)
    }

    fn take_response(mut resp: hyper::client::Response)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Created =>
                Ok(try!(client::require_content_type_application_json(&resp.headers))),
            hyper::status::StatusCode::BadRequest =>
                Err(error::new_because_invalid_database_name(&mut resp)),
            hyper::status::StatusCode::Unauthorized =>
                Err(error::new_because_unauthorized(&mut resp)),
            hyper::status::StatusCode::PreconditionFailed =>
                Err(error::new_because_database_exists(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus {
                got: resp.status,
            })
        }
    }
}
