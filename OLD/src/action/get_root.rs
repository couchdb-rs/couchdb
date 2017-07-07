use hyper;

use Error;
use Root;
use client::ClientState;
use action::{self, Action, Request, Response};

/// Action to get meta-information about the CouchDB server.
///
/// # Errors
///
/// All errors that occur as a result of executing this action are private.
///
/// # Examples
///
/// The following example shows how to get the version of the CouchDB server.
///
/// ```no_run
/// extern crate couchdb;
///
/// let client = couchdb::Client::new("http://couchdb-server:5984").unwrap();
/// let root = client.get_root().run().unwrap();
/// assert_eq!("1.6.1", root.version.to_string());
/// ```
///
pub struct GetRoot<'a> {
    client_state: &'a ClientState,
}

impl<'a> GetRoot<'a> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState) -> Self {
        GetRoot { client_state: client_state }
    }

    impl_action_public_methods!(Root);
}

impl<'a> Action for GetRoot<'a> {
    type Output = Root;
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = {
            let mut uri = self.client_state.uri.clone();
            uri.path_mut().unwrap()[0] = "".to_string();
            uri
        };
        let request = Request::new(hyper::Get, uri).set_accept_application_json();
        Ok((request, ()))
    }

    fn take_response<R>(mut response: R, _state: Self::State) -> Result<Self::Output, Error>
        where R: Response
    {
        match response.status() {
            hyper::status::StatusCode::Ok => {
                try!(response.content_type_must_be_application_json());
                response.decode_json_all::<Root>()
            }
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;
    use serde_json;

    use dbtype;
    use action::{Action, JsonResponse};
    use client::ClientState;
    use super::GetRoot;

    #[test]
    fn make_request() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetRoot::new(&client_state);
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn take_response_ok() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("couchdb", "Welcome")
                         .insert("uuid", "85fb71bf700c17267fef77535820e371")
                         .insert_object("vendor", |x| {
                             x.insert("name", "The Apache Software Foundation")
                              .insert("version", "1.3.1")
                         })
                         .insert("version", "1.3.1")
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let expected = dbtype::RootBuilder::new("Welcome",
                                                "85fb71bf700c17267fef77535820e371",
                                                "The Apache Software Foundation",
                                                "1.3.1")
                           .unwrap();
        let got = GetRoot::take_response(response, ()).unwrap();
        assert_eq!(expected, got);
    }
}
