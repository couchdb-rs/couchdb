use {DatabaseName, Error};
use futures::Future;
use transport::{ActionFuture, Method, Request, Response, ServerResponseFuture, StatusCode, Transport};

/// `GetAllDbs` is an action to get a list of all databases on a CouchDB server.
#[derive(Debug)]
pub struct GetAllDbs<'a, T: Transport + 'a> {
    transport: &'a T,
}

impl<'a, T: Transport> GetAllDbs<'a, T> {
    #[doc(hidden)]
    pub fn new(transport: &'a T) -> Self {
        GetAllDbs { transport: transport }
    }

    /// Sends the request and returns a future of the result.
    ///
    /// # Errors
    ///
    /// This action has no categorized errors.
    ///
    ///
    pub fn send(&mut self) -> ActionFuture<Vec<DatabaseName>> {

        ActionFuture::new(
            self.transport
                .request(Method::Get, Ok("/_all_dbs"))
                .and_then(|mut request| {
                    request.accept_application_json();
                    request.send_without_body()
                })
                .and_then(|mut response| {
                    response.json_body::<Vec<DatabaseName>>().map(move |x| {
                        (response, x)
                    })
                })
                .and_then(|(response, dbs)| {
                    let maybe_category = match response.status_code() {
                        StatusCode::Ok => return ServerResponseFuture::ok(dbs),
                        _ => None,
                    };
                    ServerResponseFuture::err(response, maybe_category)
                })
                .map_err(|e| {
                    Error::chain("Failed to GET all databases (/_all_dbs)", e)
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use transport::MockTransport;

    #[test]
    fn get_all_dbs_succeeds_on_200_ok() {

        use std::collections::HashSet;

        let transport = MockTransport::new();
        let action = GetAllDbs::new(&transport).send();
        let result = transport.mock(action, |mock| {
            mock.and_then(|request| {
                let request = request.expect("Client did not send request");
                assert_eq!(request.method(), Method::Get);
                assert_eq!(request.url_path(), "/_all_dbs");
                assert!(request.is_accept_application_json());
                let mut response = request.response(StatusCode::Ok);
                response.set_json_body(&json!(["_replicator", "_users", "alpha", "bravo"]));
                response.finish()
            }).and_then(|request| {
                    assert!(request.is_none());
                    MockTransport::done()
                })
        });

        let expected = ["_replicator", "_users", "alpha", "bravo"]
            .iter()
            .map(|&x| DatabaseName::from(x))
            .collect::<HashSet<_>>();

        match result {
            Ok(ref x) if x.into_iter().map(|x| x.clone()).collect::<HashSet<_>>() == expected => {}
            x => panic!("Got unexpected result {:?}", x),
        }
    }
}
