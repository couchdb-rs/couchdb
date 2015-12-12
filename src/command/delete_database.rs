use hyper;

use client::ClientState;
use dbpath::DatabasePath;
use error::{Error, ErrorResponse};
use transport::{self, Command, Request};

/// Command to delete a database.
pub struct DeleteDatabase<'a> {
    client_state: &'a ClientState,
    path: DatabasePath,
}

impl<'a> DeleteDatabase<'a> {
    #[doc(hidden)]
    pub fn new_delete_database(client_state: &'a ClientState, path: DatabasePath) -> Self {
        DeleteDatabase {
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
    /// * `Error::Unauthorized`: The client is unauthorized.
    ///
    pub fn run(self) -> Result<(), Error> {
        transport::run_command(self)
    }
}

impl<'a> Command for DeleteDatabase<'a> {
    type Output = ();
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = self.path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Delete, uri)).accept_application_json();
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response, _state: Self::State) -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => transport::content_type_must_be_application_json(&resp.headers),
            hyper::status::StatusCode::BadRequest => {
                // The CouchDB spec says this status may also mean the document
                // id has been "forgotten"--whatever that means!
                Err(Error::InvalidDatabaseName { response: try!(ErrorResponse::from_reader(resp)) })
            }
            hyper::status::StatusCode::Unauthorized => {
                Err(Error::Unauthorized { response: try!(ErrorResponse::from_reader(resp)) })
            }
            hyper::status::StatusCode::NotFound => {
                Err(Error::NotFound { response: Some(try!(ErrorResponse::from_reader(resp))) })
            }
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
