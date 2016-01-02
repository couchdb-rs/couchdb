use hyper;

use DatabaseName;
use Error;
use client::ClientState;
use command::{self, Command, Request};
use json;

/// Command to get all database names.
///
/// # Errors
///
/// All errors that occur as a result of executing this command are private.
///
pub struct GetAllDatabases<'a> {
    client_state: &'a ClientState,
}

impl<'a> GetAllDatabases<'a> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState) -> Self {
        GetAllDatabases { client_state: client_state }
    }

    impl_command_public_methods!(Vec<DatabaseName>);
}

impl<'a> Command for GetAllDatabases<'a> {
    type Output = Vec<DatabaseName>;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = {
            let mut uri = self.client_state.uri.clone();
            uri.path_mut().unwrap()[0] = "_all_dbs".to_string();
            uri
        };
        let req = try!(Request::new(hyper::Get, uri)).accept_application_json();
        Ok((req, ()))
    }

    fn take_response(resp: hyper::client::Response,
                     _state: Self::State)
                     -> Result<Self::Output, Error> {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                try!(command::content_type_must_be_application_json(&resp.headers));
                json::decode_json::<_, Vec<DatabaseName>>(resp)
            }
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status }),
        }
    }
}
