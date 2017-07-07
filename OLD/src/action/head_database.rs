use hyper;

use Error;
use IntoDatabasePath;
use client::ClientState;
use action::{self, Action, Request, Response};

/// Action to check whether a database exists.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
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

    impl_action_public_methods!(());
}

impl<'a, P: IntoDatabasePath> Action for HeadDatabase<'a, P> {
    type Output = ();
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let request = Request::new(hyper::Head, uri);
        Ok((request, ()))
    }

    fn take_response<R>(response: R, _state: Self::State) -> Result<Self::Output, Error>
        where R: Response
    {
        match response.status() {
            hyper::status::StatusCode::Ok => Ok(()),
            hyper::status::StatusCode::NotFound => Err(Error::NotFound(None)),
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;

    use DatabasePath;
    use client::ClientState;
    use action::{Action, NoContentResponse};
    use super::HeadDatabase;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = HeadDatabase::new(&client_state, "/foo");
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::method::Method::Head);
        expect_request_uri!(request, "http://example.com:1234/foo");
    }

    #[test]
    fn take_response_ok() {
        let response = NoContentResponse::new(hyper::Ok);
        HeadDatabase::<DatabasePath>::take_response(response, ()).unwrap();
    }

    #[test]
    fn take_response_not_found() {
        let response = NoContentResponse::new(hyper::NotFound);
        let got = HeadDatabase::<DatabasePath>::take_response(response, ());
        expect_couchdb_error!(got, NotFound);
    }
}
