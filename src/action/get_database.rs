use {Database, Error, IntoDatabasePath};
use action::E_ACTION_USED;
use error::ErrorCategory;
use futures::Future;
use transport::{ActionFuture, Method, Request, Response, ServerResponseFuture, StatusCode, Transport};

/// `GetDatabase` is an action to get meta-information about a database.
#[derive(Debug)]
pub struct GetDatabase<'a, T: Transport + 'a> {
    transport: &'a T,
    inner: Option<Inner>,
}

#[derive(Debug)]
struct Inner {
    url_path: Result<String, Error>,
}

impl<'a, T: Transport> GetDatabase<'a, T> {
    #[doc(hidden)]
    pub fn new<P: IntoDatabasePath>(transport: &'a T, db_path: P) -> Self {
        GetDatabase {
            transport: transport,
            inner: Some(Inner {
                url_path: db_path.into_database_path().map(|x| x.to_string()),
            }),
        }
    }

    /// Sends the request and returns a future of the result.
    ///
    /// # Errors
    ///
    /// Some possible errors:
    ///
    /// * `Error::is_not_found`
    ///
    pub fn send(&mut self) -> ActionFuture<Database> {

        let inner = self.inner.take().expect(E_ACTION_USED);

        ActionFuture::new(
            self.transport
                .request(Method::Get, inner.url_path)
                .and_then(|mut request| {
                    request.accept_application_json();
                    request.send_without_body()
                })
                .and_then(|response| {
                    let maybe_category = match response.status_code() {
                        StatusCode::Ok => return ServerResponseFuture::ok(response),
                        StatusCode::NotFound => Some(ErrorCategory::NotFound),
                        _ => None,
                    };
                    ServerResponseFuture::err(response, maybe_category)
                })
                .and_then(|mut response| response.json_body())
                .map_err(|e| Error::chain("Failed to GET database", e)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use DatabaseName;
    use futures::Future;
    use transport::MockTransport;

    #[test]
    fn get_database_succeeds_on_200_ok() {

        let transport = MockTransport::new();
        let action = GetDatabase::new(&transport, "/foo").send();
        let result = transport.mock(action, |mock| {
            mock.and_then(|request| {
                let request = request.expect("Client did not send request");
                assert_eq!(request.method(), Method::Get);
                assert_eq!(request.url_path(), "/foo");
                assert!(request.is_accept_application_json());
                let mut response = request.response(StatusCode::Ok);
                response.set_json_body(&json!({
                    "committed_update_seq": 292786,
                    "compact_running": false,
                    "data_size": 65031503,
                    "db_name": "receipts",
                    "disk_format_version": 6,
                    "disk_size": 137433211,
                    "doc_count": 6146,
                    "doc_del_count": 64637,
                    "instance_start_time": "1376269325408900",
                    "purge_seq": 0,
                    "update_seq": 292786
                }));
                response.finish()
            }).and_then(|request| {
                    assert!(request.is_none());
                    MockTransport::done()
                })
        });

        fn is_expected(db: &Database) -> bool {
            db.committed_update_seq == 292786 && db.compact_running == false && db.data_size == 65031503 &&
                db.db_name == DatabaseName::from("receipts") &&
                db.disk_format_version == 6 && db.disk_size == 137433211 && db.doc_count == 6146 &&
                db.doc_del_count == 64637 &&
                db.instance_start_time == "1376269325408900" && db.purge_seq == 0 && db.update_seq == 292786
        }

        match result {
            Ok(ref db) if is_expected(db) => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }

    #[test]
    fn get_database_fails_on_404_not_found() {

        let transport = MockTransport::new();
        let action = GetDatabase::new(&transport, "/foo").send();
        let result = transport.mock(action, |mock| {
            mock.and_then(|request| {
                let request = request.expect("Client did not send request");
                let mut response = request.response(StatusCode::NotFound);
                response.set_json_body(&json!({
                    "error": "not_found",
                    "reason": "no_db_file"
                }));
                response.finish()
            }).and_then(|request| {
                    assert!(request.is_none());
                    MockTransport::done()
                })
        });

        match result {
            Err(ref e) if e.is_not_found() => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }
}
