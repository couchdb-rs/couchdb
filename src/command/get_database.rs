use hyper;

use Database;
use Error;
use ErrorResponse;
use IntoDatabasePath;
use client::ClientState;
use command::{self, Command, Request};
use json;

/// Command to get database meta-information.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
///
/// * `Error::NotFound`: The database does not exist.
///
pub struct GetDatabase<'a, P>
    where P: IntoDatabasePath
{
    client_state: &'a ClientState,
    path: P,
}

impl<'a, P: IntoDatabasePath> GetDatabase<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        GetDatabase {
            client_state: client_state,
            path: path,
        }
    }

    impl_command_public_methods!(Database);
}

impl<'a, P: IntoDatabasePath> Command for GetDatabase<'a, P> {
    type Output = Database;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Get, uri)).accept_application_json();
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                try!(command::content_type_must_be_application_json(&resp.headers));
                json::decode_json::<_, Database>(resp)
            }
            hyper::status::StatusCode::NotFound => {
                Err(Error::NotFound(Some(try!(json::decode_json::<_, ErrorResponse>(resp)))))
            }
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
