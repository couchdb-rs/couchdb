use hyper;

use Error;
use ErrorResponse;
use IntoDocumentPath;
use Revision;
use client::{self, ClientState};
use command::{self, Command, Request, Response};
use dbtype::DeleteDocumentResponse;

/// Command to delete a document.
///
/// # Return
///
/// This command returns the document's new revision.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this command:
///
/// * `Error::DocumentConflict`: The revision is not the latest for the
///   document.
/// * `Error::NotFound`: The document does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct DeleteDocument<'a, P: IntoDocumentPath> {
    client_state: &'a client::ClientState,
    path: P,
    rev: &'a Revision,
}

impl<'a, P: IntoDocumentPath> DeleteDocument<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P, rev: &'a Revision) -> Self {
        DeleteDocument {
            client_state: client_state,
            path: path,
            rev: rev,
        }
    }

    impl_command_public_methods!(Revision);
}

impl<'a, P: IntoDocumentPath> Command for DeleteDocument<'a, P> {
    type Output = Revision;

    fn make_request(self) -> Result<Request, Error> {
        let doc_path = try!(self.path.into_document_path());
        let uri = doc_path.into_uri(self.client_state.uri.clone());
        let request = Request::new(hyper::Delete, uri)
                          .set_accept_application_json()
                          .set_if_match_revision(Some(self.rev));
        Ok(request)
    }

    fn take_response<R: Response>(mut response: R) -> Result<Self::Output, Error> {
        match response.status() {
            hyper::status::StatusCode::Ok => {
                try!(response.content_type_must_be_application_json());
                let content = try!(response.decode_json::<DeleteDocumentResponse>());
                Ok(content.rev)
            }
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            hyper::status::StatusCode::Unauthorized => {
                Err(make_couchdb_error!(Unauthorized, response))
            }
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, response)),
            hyper::status::StatusCode::Conflict => {
                Err(make_couchdb_error!(DocumentConflict, response))
            }
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;
    use serde_json;

    use DocumentPath;
    use Revision;
    use client::ClientState;
    use command::{Command, JsonResponse};
    use super::DeleteDocument;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let rev = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let command = DeleteDocument::new(&client_state, "/foo/bar", &rev);
        let request = command.make_request().unwrap();
        expect_request_method!(request, hyper::method::Method::Delete);
        expect_request_uri!(request, "http://example.com:1234/foo/bar");
        expect_request_accept_application_json!(request);
        // expect_request_if_match_revision!(request, "42-1234567890abcdef1234567890abcdef");
        expect_request_if_match_revision!(request, &rev.to_string());
    }

    #[test]
    fn take_response_ok() {
        let expected = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("id", "bar")
                         .insert("ok", true)
                         .insert("rev", expected.to_string())
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let got = DeleteDocument::<DocumentPath>::take_response(response).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "bad_request")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = DeleteDocument::<DocumentPath>::take_response(response);
        expect_couchdb_error!(got, BadRequest);
    }

    #[test]
    fn take_response_conflict() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "conflict")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Conflict, &source);
        let got = DeleteDocument::<DocumentPath>::take_response(response);
        expect_couchdb_error!(got, DocumentConflict);
    }

    #[test]
    fn take_response_not_found() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "not_found")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::NotFound, &source);
        let got = DeleteDocument::<DocumentPath>::take_response(response);
        expect_couchdb_error!(got, NotFound);
    }

    #[test]
    fn take_response_unauthorized() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "unauthorized")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Unauthorized, &source);
        let got = DeleteDocument::<DocumentPath>::take_response(response);
        expect_couchdb_error!(got, Unauthorized);
    }
}
