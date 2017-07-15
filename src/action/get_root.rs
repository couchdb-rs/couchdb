use {Error, Root};
use futures::Future;
use transport::{ActionFuture, Method, Request, Response, ServerResponseFuture, StatusCode, Transport};

/// `GetRoot` is an action to get the CouchDB server version and other
/// meta-information.
#[derive(Debug)]
pub struct GetRoot<'a, T: Transport + 'a> {
    transport: &'a T,
}

impl<'a, T: Transport> GetRoot<'a, T> {
    #[doc(hidden)]
    pub fn new(transport: &'a T) -> Self {
        GetRoot { transport: transport }
    }

    /// Sends the request and returns a future of the result.
    ///
    /// # Errors
    ///
    /// This action has no categorized errors.
    ///
    ///
    pub fn send(&mut self) -> ActionFuture<Root> {

        ActionFuture::new(
            self.transport
                .request(Method::Get, Ok("/"))
                .and_then(|mut request| {
                    request.accept_application_json();
                    request.send_without_body()
                })
                .and_then(|mut response| {
                    response.json_body::<Root>().map(move |x| (response, x))
                })
                .and_then(|(response, root)| {
                    let maybe_category = match response.status_code() {
                        StatusCode::Ok => return ServerResponseFuture::ok(root),
                        _ => None,
                    };
                    ServerResponseFuture::err(response, maybe_category)
                })
                .map_err(|e| Error::chain("Failed to PUT database", e)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use transport::MockTransport;
    use uuid::Uuid;

    #[test]
    fn put_database_succeeds_on_200_ok() {

        let transport = MockTransport::new();
        let action = GetRoot::new(&transport).send();
        let result = transport.mock(action, |mock| {
            mock.and_then(|request| {
                let request = request.expect("Client did not send request");
                assert_eq!(request.method(), Method::Get);
                assert_eq!(request.url_path(), "/");
                assert!(request.is_accept_application_json());
                let mut response = request.response(StatusCode::Ok);
                response.set_json_body(&json!({
                    "couchdb": "Welcome",
                    "uuid": "85fb71bf700c17267fef77535820e371",
                    "vendor": {
                        "name": "The Apache Software Foundation",
                        "version": "1.3.1"
                    },
                    "version": "1.3.1",
                }));
                response.finish()
            }).and_then(|request| {
                    assert!(request.is_none());
                    MockTransport::done()
                })
        });

        fn is_expected(x: &Root) -> bool {
            x.welcome() == "Welcome" && x.uuid() == &Uuid::parse_str("85fb71bf700c17267fef77535820e371").unwrap() &&
                x.version() == "1.3.1" && x.version_triple() == Some((1, 3, 1))
        }

        match result {
            Ok(ref root) if is_expected(root) => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }
}
