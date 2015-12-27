use hyper;

use client::ClientState;
use dbpath::DatabasePath;
use error::Error;
use transport::{self, Command, Request};

/// Command to get database meta-information.
pub struct HeadDatabase<'a> {
    client_state: &'a ClientState,
    path: DatabasePath,
}

impl<'a> HeadDatabase<'a> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: DatabasePath) -> Self {
        HeadDatabase {
            client_state: client_state,
            path: path,
        }
    }

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
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = self.path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Head, uri));
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response, _state: Self::State) -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => Ok(()),
            hyper::status::StatusCode::NotFound => Err(Error::NotFound(None)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
