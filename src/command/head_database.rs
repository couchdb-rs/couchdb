use hyper;

use client::ClientState;
use database;
use error::Error;
use transport::{self, Command, Request};

#[doc(hidden)]
pub fn new_head_database<'a>(
    client_state: &'a ClientState,
    db_name: &'a str)
    -> HeadDatabase<'a>
{
    HeadDatabase {
        client_state: client_state,
        db_name: db_name,
    }
}

/// Command to get database meta-information.
pub struct HeadDatabase<'a> {
    client_state: &'a ClientState,
    db_name: &'a str,
}

impl<'a> HeadDatabase<'a> {

    /// Send the command request and wait for the response.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::NotFound`: The database does not exist.
    ///
    pub fn run(self) -> Result<(), Error> {
        transport::run_command(self)
    }
}

impl<'a> Command for HeadDatabase<'a> {

    type Output = ();

    fn make_request(self) -> Result<Request, Error> {
        let uri = database::new_uri(&self.client_state.uri, self.db_name);
        Request::new(hyper::Head, uri)
    }

    fn take_response(resp: hyper::client::Response)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => Ok(()),
            hyper::status::StatusCode::NotFound =>
                Err(Error::NotFound { response: None } ),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}
