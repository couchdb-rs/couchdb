use {Error, futures, reqwest, serde_json, tokio_core, transport};
use futures::Future;
use serde::{Deserialize, Serialize};
use transport::{Method, StatusCode};

fn must_receive(canceled: futures::unsync::oneshot::Canceled) -> Error {
    panic!("Mock transport channel canceled: {:?}", canceled);
}

#[derive(Debug)]
pub struct MockTransport {}

impl MockTransport {
    pub fn new() -> Self {
        MockTransport {}
    }

    pub fn mock<A, F>(&self, action: A, server_mocker: F) -> Result<A::Item, Error>
    where
        A: transport::Action,
        F: FnOnce(MockRequest) -> MockResponse,
    {
        let (request_tx, request_rx) = futures::unsync::oneshot::channel();
        let (response_tx, response_rx) = futures::unsync::oneshot::channel();

        let request_maker = MockRequestMaker::new(request_tx, response_rx);
        let action_future = action.act(request_maker);

        let mock_future = request_rx.map_err(must_receive).and_then(|request| {
            let response = server_mocker(request);
            response_tx.send(response).unwrap();
            Ok(())
        });

        let mut reactor = tokio_core::reactor::Core::new().unwrap();
        reactor.run(action_future.join(mock_future)).map(
            |(action_item, _)| action_item,
        )
    }

    // Defined here instead of in MockResponse as a convenience for test code so
    // that test code needn't import MockResponse.
    pub fn new_response(status_code: StatusCode) -> MockResponse {
        MockResponse {
            status_code: status_code,
            headers: reqwest::header::Headers::new(),
            body: None,
        }
    }
}

impl transport::Transport for MockTransport {}

#[derive(Debug)]
struct MockRequestMaker {
    request_tx: futures::unsync::oneshot::Sender<MockRequest>,
    response_rx: futures::unsync::oneshot::Receiver<MockResponse>,
}

impl MockRequestMaker {
    fn new(
        request_tx: futures::unsync::oneshot::Sender<MockRequest>,
        response_rx: futures::unsync::oneshot::Receiver<MockResponse>,
    ) -> Self {
        MockRequestMaker {
            request_tx: request_tx,
            response_rx: response_rx,
        }
    }
}

impl transport::RequestMaker for MockRequestMaker {
    type Request = MockRequest;
    type Future = futures::future::FutureResult<Self::Request, Error>;
    fn make_request(self, method: Method, url_path: &str) -> Self::Future {
        futures::future::ok(MockRequest::new(
            self.request_tx,
            self.response_rx,
            method,
            url_path,
        ))
    }
}

#[derive(Default, Debug)]
pub struct MockRequest {
    request_tx: Option<futures::unsync::oneshot::Sender<MockRequest>>,
    response_rx: Option<futures::unsync::oneshot::Receiver<MockResponse>>,
    method: Method,
    url_path: String,
    headers: reqwest::header::Headers,
}

impl MockRequest {
    fn new(
        request_tx: futures::unsync::oneshot::Sender<MockRequest>,
        response_rx: futures::unsync::oneshot::Receiver<MockResponse>,
        method: Method,
        url_path: &str,
    ) -> Self {
        MockRequest {
            request_tx: Some(request_tx),
            response_rx: Some(response_rx),
            method: method,
            url_path: String::from(url_path),
            headers: reqwest::header::Headers::new(),
        }
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub fn url_path(&self) -> &str {
        &self.url_path
    }

    pub fn is_accept_application_json(&self) -> bool {
        self.headers
            .get::<reqwest::header::Accept>()
            .map(|x| *x == reqwest::header::Accept::json())
            .unwrap_or(false)
    }
}

impl transport::Request for MockRequest {
    type Response = MockResponse;
    type Future = Box<Future<Item = Self::Response, Error = Error>>;

    fn set_accept_application_json(&mut self) {
        self.headers.set(reqwest::header::Accept::json());
    }

    fn send_without_body(mut self) -> Self::Future {
        let request_tx = self.request_tx.take().unwrap();
        let response_rx = self.response_rx.take().unwrap();
        request_tx.send(self).unwrap();
        Box::new(response_rx.map_err(must_receive))
    }
}

#[derive(Debug)]
pub struct MockResponse {
    status_code: StatusCode,
    headers: reqwest::header::Headers,
    body: Option<Body>,
}

impl MockResponse {
    pub fn set_json_body<T: Serialize>(&mut self, content: &T) {
        self.headers.set(reqwest::header::ContentType::json());
        self.body = Some(Body::Json(serde_json::to_vec(content).unwrap()));
    }
}

impl transport::Response for MockResponse {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn json_body<T>(&mut self) -> Box<Future<Item = T, Error = Error>>
    where
        for<'de> T: Deserialize<'de> + 'static,
    {
        Box::new(futures::future::result(
            if let Some(Body::Json(ref bytes)) = self.body {
                serde_json::from_slice(bytes).map_err(|e| {
                    Error::from((
                        format!(
                            "Could not decode mock JSON body (bytes: {:?})",
                            String::from_utf8_lossy(bytes)
                        ),
                        e,
                    ))
                })
            } else {
                Err(Error::from(
                    format!("Expected mock JSON body, got {:?}", self.body),
                ))
            },
        ))
    }
}

#[derive(Debug)]
enum Body {
    Json(Vec<u8>),
}
