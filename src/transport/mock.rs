use {Error, futures, reqwest, serde_json, std, transport};
use futures::{BoxFuture, Future};
use serde::Deserialize;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use transport::{Method, StatusCode};
use url::Url;

#[derive(Debug)]
pub struct Transport {
    mock_url: Url,
    mock_engine: Arc<Mutex<MockEngine>>,
}

#[derive(Debug)]
pub struct Request {
    mock_engine: Arc<Mutex<MockEngine>>,
    response_tx: Option<futures::sync::oneshot::Sender<Response>>,
    response_rx: Option<futures::sync::oneshot::Receiver<Response>>,
    method: Method,
    url: Url,
    headers: reqwest::header::Headers,
}

#[derive(Debug)]
pub struct Response {
    status_code: StatusCode,
    headers: reqwest::header::Headers,
    body: Vec<u8>,
}

#[derive(Debug)]
struct MockEngine {
    incoming_requests: VecDeque<Request>,
}

impl Transport {
    pub fn new() -> Self {
        Transport {
            mock_url: "http://mock-transport".parse().unwrap(),
            mock_engine: Arc::new(Mutex::new(
                MockEngine { incoming_requests: VecDeque::new() },
            )),
        }
    }

    pub fn mock_url(&self) -> &Url {
        &self.mock_url
    }

    pub fn new_response(&self, status_code: StatusCode) -> Response {
        Response {
            status_code: status_code,
            headers: reqwest::header::Headers::new(),
            body: Vec::new(),
        }
    }

    pub fn handle_request<H>(&self, handler: H)
    where
        H: FnOnce(Request) -> Response,
    {
        let mut mock_engine = self.mock_engine.lock().unwrap();
        let mut request = mock_engine.incoming_requests.pop_front().unwrap();
        let response_tx = std::mem::replace(&mut request.response_tx, None).unwrap();
        let response = handler(request);
        response_tx.send(response).unwrap();
    }
}

impl transport::Transport for Transport {
    type Request = Request;
    fn request(&self, method: Method, url: Url) -> Self::Request {
        Request::new(self.mock_engine.clone(), method, url)
    }
}

impl Request {
    fn new(mock_engine: Arc<Mutex<MockEngine>>, method: Method, url: Url) -> Self {

        let (tx, rx) = futures::sync::oneshot::channel();

        Request {
            mock_engine: mock_engine,
            method: method,
            url: url,
            headers: reqwest::header::Headers::new(),
            response_tx: Some(tx),
            response_rx: Some(rx),
        }
    }

    pub fn is_accept_application_json(&self) -> bool {
        self.headers
            .get::<reqwest::header::Accept>()
            .map(|x| *x == reqwest::header::Accept::json())
            .unwrap_or(false)
    }
}

impl transport::Request for Request {
    fn accept_application_json(mut self) -> Self {
        self.headers.set(reqwest::header::Accept::json());
        self
    }

    fn send<H: transport::ResponseHandler + 'static>(mut self, handler: H) -> BoxFuture<H::Item, Error> {

        let response_rx = std::mem::replace(&mut self.response_rx, None).unwrap();

        let mock_engine = self.mock_engine.clone();

        {
            let mut mock_engine = mock_engine.lock().unwrap();
            mock_engine.incoming_requests.push_back(self);
        }

        response_rx
            .map_err(|_canceled| Error::from("Mock request handler canceled"))
            .and_then(|response| handler.handle_response(response))
            .boxed()
    }
}

impl Response {}

impl transport::Response for Response {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn decode_json_body<T>(&mut self) -> Result<T, Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        let body = std::mem::replace(&mut self.body, Vec::new());

        serde_json::from_slice(&body).map_err(|e| {
            Error::from(("Failed to decode mock response body as JSON", e))
        })
    }
}
