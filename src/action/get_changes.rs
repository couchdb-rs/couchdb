use hyper;

use Changes;
use Error;
use ErrorResponse;
use IntoDatabasePath;
use action::{self, Action, Request, Response};
use client::ClientState;

/// Action to get changes made to documents in a database.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
///
/// * `Error::BadRequest`: Bad request.
///
pub struct GetChanges<'a, P>
    where P: IntoDatabasePath
{
    client_state: &'a ClientState,
    path: P,
}

impl<'a, P: IntoDatabasePath> GetChanges<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        GetChanges {
            client_state: client_state,
            path: path,
        }
    }

    impl_action_public_methods!(Changes);
}

impl<'a, P: IntoDatabasePath> Action for GetChanges<'a, P> {
    type Output = Changes;

    fn make_request(self) -> Result<Request, Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = {
            let mut uri = db_path.into_uri(self.client_state.uri.clone());
            uri.path_mut().unwrap().push("_changes".to_string());
            uri
        };
        let request = Request::new(hyper::Get, uri).set_accept_application_json();
        Ok(request)
    }

    fn take_response<R: Response>(mut response: R) -> Result<Self::Output, Error> {
        match response.status() {
            hyper::Ok => {
                try!(response.content_type_must_be_application_json());
                response.decode_json::<Changes>()
            }
            hyper::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;
    use serde_json;

    use ChangesBuilder;
    use DatabasePath;
    use action::{Action, JsonResponse};
    use client::ClientState;
    use super::GetChanges;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetChanges::new(&client_state, "/db");
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/db/_changes");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn take_response_ok() {
        let expected = ChangesBuilder::new(11)
                           .build_result(6, "6478c2ae800dfc387396d14e1fc39626", |x| {
                               x.build_change_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337",
                                                           |x| x)
                           })
                           .unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("last_seq", 11)
                         .insert_array("results", |x| {
                             x.push_object(|x| {
                                 x.insert_array("changes", |x| {
                                      x.push_object(|x| {
                                          x.insert("rev", "2-7051cbe5c8faecd085a3fa619e6e6337")
                                      })
                                  })
                                  .insert("id", "6478c2ae800dfc387396d14e1fc39626")
                                  .insert("seq", 6)
                             })
                         })
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let got = GetChanges::<DatabasePath>::take_response(response).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "bad_request")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = GetChanges::<DatabasePath>::take_response(response);
        expect_couchdb_error!(got, BadRequest);
    }
}
