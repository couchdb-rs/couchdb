use hyper;
use serde;
use serde_json;

use DocumentId;
use Error;
use ErrorResponse;
use IntoDatabasePath;
use Revision;
use client::ClientState;
use action::{self, Action, Request, Response};
use dbtype::PostToDatabaseResponse;
use error::EncodeErrorKind;

/// Action to create a document.
///
/// # Return
///
/// This action returns the new document's revision and id.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
///
/// * `Error::NotFound`: The database does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct PostToDatabase<'a, P, T>
    where P: IntoDatabasePath,
          T: 'a + serde::Serialize
{
    client_state: &'a ClientState,
    path: P,
    doc_content: &'a T,
}

impl<'a, P: IntoDatabasePath, T: 'a + serde::Serialize> PostToDatabase<'a, P, T> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P, doc_content: &'a T) -> Self
        where T: serde::Serialize
    {
        PostToDatabase {
            client_state: client_state,
            path: path,
            doc_content: doc_content,
        }
    }

    impl_action_public_methods!((Revision, DocumentId));
}

impl<'a, P: IntoDatabasePath, T: 'a + serde::Serialize> Action for PostToDatabase<'a, P, T> {
    type Output = (Revision, DocumentId);

    fn make_request(self) -> Result<Request, Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let body = try!(serde_json::to_vec(self.doc_content)
                            .map_err(|e| Error::Encode(EncodeErrorKind::Serde { cause: e })));
        let request = Request::new(hyper::method::Method::Post, uri)
                          .set_accept_application_json()
                          .set_content_type_application_json()
                          .set_body(body);
        Ok(request)
    }

    fn take_response<R: Response>(mut response: R) -> Result<Self::Output, Error> {
        match response.status() {
            hyper::status::StatusCode::Created => {
                try!(response.content_type_must_be_application_json());
                let content = try!(response.decode_json::<PostToDatabaseResponse>());
                let id = DocumentId::from(String::from(content.id));
                let rev: Revision = content.rev.into();
                Ok((rev, id))
            }
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            hyper::status::StatusCode::Unauthorized => {
                Err(make_couchdb_error!(Unauthorized, response))
            }
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, response)),
            hyper::status::StatusCode::Conflict => {
                // Need to include this error variant in the action's
                // documentation if we ever add support for an explicit id.
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

    use DatabasePath;
    use DocumentId;
    use Revision;
    use client::ClientState;
    use action::{Action, JsonResponse};
    use super::PostToDatabase;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let content = serde_json::builder::ObjectBuilder::new()
                          .insert("foo", 17)
                          .insert("bar", "hello")
                          .unwrap();
        let action = PostToDatabase::new(&client_state, "/db", &content);
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Post);
        expect_request_uri!(request, "http://example.com:1234/db");
        expect_request_accept_application_json!(request);
        expect_request_content_type_application_json!(request);
        let expected_body = serde_json::to_vec(&content).unwrap();
        expect_request_body!(request, expected_body);
    }

    #[test]
    fn take_response_created() {
        let source_rev = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("id", "doc-id")
                         .insert("ok", true)
                         .insert("rev", source_rev.to_string())
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Created, &source);
        let (rev, id) = PostToDatabase::<DatabasePath, serde_json::Value>::take_response(response)
                            .unwrap();
        assert_eq!(id, DocumentId::Normal("doc-id".into()));
        assert_eq!(rev, source_rev);
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "bad_request")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = PostToDatabase::<DatabasePath, serde_json::Value>::take_response(response);
        expect_couchdb_error!(got, BadRequest);
    }

    #[test]
    fn take_response_conflict() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "conflict")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Conflict, &source);
        let got = PostToDatabase::<DatabasePath, serde_json::Value>::take_response(response);
        expect_couchdb_error!(got, DocumentConflict);
    }

    #[test]
    fn take_response_not_found() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "not_found")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::NotFound, &source);
        let got = PostToDatabase::<DatabasePath, serde_json::Value>::take_response(response);
        expect_couchdb_error!(got, NotFound);
    }

    #[test]
    fn take_response_unauthorized() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "unauthorized")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Unauthorized, &source);
        let got = PostToDatabase::<DatabasePath, serde_json::Value>::take_response(response);
        expect_couchdb_error!(got, Unauthorized);
    }
}
