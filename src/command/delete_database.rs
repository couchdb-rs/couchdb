use hyper;

use Error;
use ErrorResponse;
use IntoDatabasePath;
use client::ClientState;
use command::{self, Command, Request};
use json;

/// Command to delete a database.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
///
/// * `Error::NotFound`: The database does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct DeleteDatabase<'a, P>
    where P: IntoDatabasePath
{
    client_state: &'a ClientState,
    path: P,
}

impl<'a, P: IntoDatabasePath> DeleteDatabase<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        DeleteDatabase {
            client_state: client_state,
            path: path,
        }
    }

    impl_command_public_methods!(());
}

impl<'a, P: IntoDatabasePath> Command for DeleteDatabase<'a, P> {
    type Output = ();
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Delete, uri)).accept_application_json();
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                command::content_type_must_be_application_json(&resp.headers)
            }
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, resp)),
            hyper::status::StatusCode::Unauthorized => Err(make_couchdb_error!(Unauthorized, resp)),
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
