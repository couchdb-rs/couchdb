use hyper;

use Error;
use ErrorResponse;
use IntoDatabasePath;
use client::ClientState;
use command::{self, Command, Request};
use json;

/// Command to create a database.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
///
/// * `Error::DatabaseExists`: The database already exists.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct PutDatabase<'a, P>
    where P: IntoDatabasePath
{
    client_state: &'a ClientState,
    path: P,
}

impl<'a, P: IntoDatabasePath> PutDatabase<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        PutDatabase {
            client_state: client_state,
            path: path,
        }
    }

    impl_command_public_methods!(());
}

impl<'a, P: IntoDatabasePath> Command for PutDatabase<'a, P> {
    type Output = ();
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::method::Method::Put, uri)).accept_application_json();
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Created => {
                command::content_type_must_be_application_json(&resp.headers)
            }
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, resp)),
            hyper::status::StatusCode::Unauthorized => Err(make_couchdb_error!(Unauthorized, resp)),
            hyper::status::StatusCode::PreconditionFailed => {
                Err(make_couchdb_error!(DatabaseExists, resp))
            }
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
