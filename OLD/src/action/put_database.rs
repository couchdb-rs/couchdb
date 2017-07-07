use hyper;

use Error;
use ErrorResponse;
use IntoDatabasePath;
use client::ClientState;
use action::{self, Action, Request, Response};

/// Action to create a database.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
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

    impl_action_public_methods!(());
}

impl<'a, P: IntoDatabasePath> Action for PutDatabase<'a, P> {
    type Output = ();
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let request = Request::new(hyper::method::Method::Put, uri).set_accept_application_json();
        Ok((request, ()))
    }

    fn take_response<R>(mut response: R, _state: Self::State) -> Result<Self::Output, Error>
        where R: Response
    {
        match response.status() {
            hyper::status::StatusCode::Created => response.content_type_must_be_application_json(),
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            hyper::status::StatusCode::Unauthorized => {
                Err(make_couchdb_error!(Unauthorized, response))
            }
            hyper::status::StatusCode::PreconditionFailed => {
                Err(make_couchdb_error!(DatabaseExists, response))
            }
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
    use super::PutDatabase;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = PutDatabase::new(&client_state, "/foo");
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::method::Method::Put);
        expect_request_uri!(request, "http://example.com:1234/foo");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn take_response_created() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("ok", true)
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Created, &source);
        PutDatabase::<DatabasePath>::take_response(response, ()).unwrap();
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "bad_request")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = PutDatabase::<DatabasePath>::take_response(response, ());
        expect_couchdb_error!(got, BadRequest);
    }

    #[test]
    fn take_response_precondition_failed() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "precondition_failed")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::PreconditionFailed, &source);
        let got = PutDatabase::<DatabasePath>::take_response(response, ());
        expect_couchdb_error!(got, DatabaseExists);
    }

    #[test]
    fn take_response_unauthorized() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "unauthorized")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Unauthorized, &source);
        let got = PutDatabase::<DatabasePath>::take_response(response, ());
        expect_couchdb_error!(got, Unauthorized);
    }
}
