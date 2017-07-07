use hyper;

use Error;
use ErrorResponse;
use IntoDatabasePath;
use client::ClientState;
use action::{self, Action, Request, Response};

/// Action to delete a database.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
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

    impl_action_public_methods!(());
}

impl<'a, P: IntoDatabasePath> Action for DeleteDatabase<'a, P> {
    type Output = ();
    type State = ();

    fn make_request(self) -> Result<((Request), Self::State), Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let request = Request::new(hyper::Delete, uri).set_accept_application_json();
        Ok((request, ()))
    }

    fn take_response<R>(mut response: R, _state: Self::State) -> Result<Self::Output, Error>
        where R: Response
    {
        match response.status() {
            hyper::status::StatusCode::Ok => response.content_type_must_be_application_json(),
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            hyper::status::StatusCode::Unauthorized => {
                Err(make_couchdb_error!(Unauthorized, response))
            }
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, response)),
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;
    use serde_json;

    use DatabasePath;
    use client::ClientState;
    use action::{Action, JsonResponse};
    use super::DeleteDatabase;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = DeleteDatabase::new(&client_state, "/foo");
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::method::Method::Delete);
        expect_request_uri!(request, "http://example.com:1234/foo");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn take_response_ok() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("ok", true)
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        DeleteDatabase::<DatabasePath>::take_response(response, ()).unwrap();
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "illegal_database_name")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = DeleteDatabase::<DatabasePath>::take_response(response, ());
        expect_couchdb_error!(got, BadRequest);
    }

    #[test]
    fn take_response_not_found() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "not_found")
                         .insert("reason", "missing")
                         .unwrap();
        let response = JsonResponse::new(hyper::NotFound, &source);
        let got = DeleteDatabase::<DatabasePath>::take_response(response, ());
        expect_couchdb_error!(got, NotFound);
    }

    #[test]
    fn take_response_unauthorized() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "unauthorized")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Unauthorized, &source);
        let got = DeleteDatabase::<DatabasePath>::take_response(response, ());
        expect_couchdb_error!(got, Unauthorized);
    }
}
