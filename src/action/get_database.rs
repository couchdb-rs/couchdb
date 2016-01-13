use hyper;

use Database;
use Error;
use ErrorResponse;
use IntoDatabasePath;
use client::ClientState;
use action::{self, Action, Request, Response};

/// Action to get database meta-information.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
///
/// * `Error::NotFound`: The database does not exist.
///
pub struct GetDatabase<'a, P>
    where P: IntoDatabasePath
{
    client_state: &'a ClientState,
    path: P,
}

impl<'a, P: IntoDatabasePath> GetDatabase<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        GetDatabase {
            client_state: client_state,
            path: path,
        }
    }

    impl_action_public_methods!(Database);
}

impl<'a, P: IntoDatabasePath> Action for GetDatabase<'a, P> {
    type Output = Database;

    fn make_request(self) -> Result<Request, Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = db_path.into_uri(self.client_state.uri.clone());
        let request = Request::new(hyper::Get, uri).set_accept_application_json();
        Ok(request)
    }

    fn take_response<R: Response>(mut response: R) -> Result<Self::Output, Error> {
        match response.status() {
            hyper::status::StatusCode::Ok => {
                try!(response.content_type_must_be_application_json());
                response.decode_json::<Database>()
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
    use super::GetDatabase;

    #[test]
    fn make_request() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetDatabase::new(&client_state, "/foo");
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/foo");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn take_response_ok() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("db_name", "foo")
                         .insert("doc_count", 3)
                         .insert("doc_del_count", 1)
                         .insert("update_seq", 16)
                         .insert("purge_seq", 0)
                         .insert("compact_running", true)
                         .insert("disk_size", 65637)
                         .insert("data_size", 1385)
                         .insert("instance_start_time", "1452379292782729")
                         .insert("disk_format_version", 6)
                         .insert("committed_update_seq", 15)
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let got = GetDatabase::<DatabasePath>::take_response(response).unwrap();
        assert_eq!(got.db_name, "foo".into());
        assert_eq!(3, got.doc_count);
        assert_eq!(1, got.doc_del_count);
        assert_eq!(16, got.update_seq);
        assert_eq!(0, got.purge_seq);
        assert_eq!(true, got.compact_running);
        assert_eq!(65637, got.disk_size);
        assert_eq!(1385, got.data_size);
        assert_eq!(1452379292782729, got.instance_start_time);
        assert_eq!(6, got.disk_format_version);
        assert_eq!(15, got.committed_update_seq);
    }

    #[test]
    fn take_response_not_found() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "not_found")
                         .insert("reason", "no_db_file")
                         .unwrap();
        let response = JsonResponse::new(hyper::NotFound, &source);
        let got = GetDatabase::<DatabasePath>::take_response(response);
        expect_couchdb_error!(got, NotFound);
    }
}
