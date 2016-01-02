use hyper;

use Error;
use IntoDatabasePath;
use client::ClientState;
use command::{self, Command, Request};

/// Command to get database meta-information.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
///
/// * `Error::NotFound`: The database does not exist.
///
pub struct HeadDatabase<'a, P>
    where P: IntoDatabasePath
{
    client_state: &'a ClientState,
    path: P,
}

impl<'a, P: IntoDatabasePath> HeadDatabase<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        HeadDatabase {
            client_state: client_state,
            path: path,
        }
    }

    impl_command_public_methods!(());
}

impl<'a, P: IntoDatabasePath> Command for HeadDatabase<'a, P> {
    type Output = ();
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Head, uri));
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => Ok(()),
            hyper::status::StatusCode::NotFound => Err(Error::NotFound(None)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
