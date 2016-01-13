use hyper;

use DatabaseName;
use Error;
use client::ClientState;
use action::{self, Action, Request, Response};

/// Action to get all database names.
///
/// # Errors
///
/// All errors that occur as a result of executing this action are private.
///
pub struct GetAllDatabases<'a> {
    client_state: &'a ClientState,
}

impl<'a> GetAllDatabases<'a> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState) -> Self {
        GetAllDatabases { client_state: client_state }
    }

    impl_action_public_methods!(Vec<DatabaseName>);
}

impl<'a> Action for GetAllDatabases<'a> {
    type Output = Vec<DatabaseName>;

    fn make_request(self) -> Result<Request, Error> {
        let uri = {
            let mut uri = self.client_state.uri.clone();
            uri.path_mut().unwrap()[0] = "_all_dbs".to_string();
            uri
        };
        let request = Request::new(hyper::Get, uri).set_accept_application_json();
        Ok(request)
    }

    fn take_response<R: Response>(mut response: R) -> Result<Self::Output, Error> {
        match response.status() {
            hyper::status::StatusCode::Ok => {
                try!(response.content_type_must_be_application_json());
                response.decode_json::<Vec<DatabaseName>>()
            }
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;
    use serde_json;

    use DatabaseName;
    use client::ClientState;
    use action::{Action, JsonResponse};
    use super::GetAllDatabases;

    #[test]
    fn make_request() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetAllDatabases::new(&client_state);
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/_all_dbs");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn take_response_ok() {
        let source = serde_json::builder::ArrayBuilder::new()
                         .push("_replicator")
                         .push("_users")
                         .push("foo")
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let expected = vec!["_replicator", "_users", "foo"]
                           .into_iter()
                           .map(|x| DatabaseName::from(x))
                           .collect::<Vec<_>>();
        let got = GetAllDatabases::take_response(response).unwrap();
        assert_eq!(expected, got);
    }
}
