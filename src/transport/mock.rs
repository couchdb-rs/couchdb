use {Error, futures, reqwest, serde_json, transport};
use futures::{Future, Sink, Stream};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use transport::{Method, StatusCode};

// There are two agents operating in a MockTransport: (1) the *action* being
// tested and (2) the *mocker* that checks the action's requests and constructs
// mock responses to be consumed by the action.
//
// The agents communicate with each other via a pair of channels. One channel is
// for the requests, the other is for the responses.
//
// Everything is carried out via futures.

#[derive(Clone, Debug)]
pub struct MockTransport {
    // The Rc wrapper allows us to derive the Clone trait, which is required by
    // the Transport trait.
    //
    // The RefCell wrapper is needed because the Transport trait requires
    // (external) immutability all over.
    inner: Rc<RefCell<Inner>>,
}

#[derive(Debug)]
struct Inner {
    mocker_channels: Option<MockerChannelPair>,
    action_channels: Option<ActionChannelPair>,
}

#[derive(Debug)]
struct MockerChannelPair(futures::unsync::mpsc::Receiver<MockRequest>, futures::unsync::mpsc::Sender<MockResponse>);

#[derive(Debug)]
struct ActionChannelPair(futures::unsync::mpsc::Sender<MockRequest>, futures::unsync::mpsc::Receiver<MockResponse>);

pub type MockFuture = Box<Future<Item = Option<MockRequest>, Error = Error>>;

impl MockTransport {
    pub fn new() -> Self {

        let (request_tx, request_rx) = futures::unsync::mpsc::channel(1);
        let (response_tx, response_rx) = futures::unsync::mpsc::channel(1);

        let mocker_channels = MockerChannelPair(request_rx, response_tx);
        let action_channels = ActionChannelPair(request_tx, response_rx);

        MockTransport {
            inner: Rc::new(RefCell::new(Inner {
                mocker_channels: Some(mocker_channels),
                action_channels: Some(action_channels),
            })),
        }
    }

    pub fn mock<A, F, M, T>(self, action: A, mocker: M) -> Result<T, Error>
    where
        A: Future<Item = T, Error = Error>,
        F: Future<Item = (), Error = Error>,
        M: FnOnce(MockFuture) -> F,
    {
        let MockerChannelPair(request_rx, response_tx) = self.inner
            .try_borrow_mut()
            .unwrap()
            .mocker_channels
            .take()
            .unwrap();

        let maybe_request = {
            let inner = self.inner.clone();
            Box::new(
                request_rx
                    .into_future()
                    .map_err(|_| {
                        Error::from("MockTransport failed to receive on request channel")
                    })
                    .map(move |(maybe_request, request_rx)| {

                        // Must move the channel back into the Inner state
                        // *before* the mocker begins handling the request so
                        // that the mocker can construct a response.
                        inner.try_borrow_mut().unwrap().mocker_channels =
                            Some(MockerChannelPair(request_rx, response_tx));

                        maybe_request
                    }),
            )
        };

        // Unless an error occurs, the action future will finish first because
        // the mocker only finishes *after* the request sender is closed, which
        // we must do ourselves.

        let action_result = match action.select2(mocker(maybe_request)).wait() {
            Ok(futures::future::Either::A((action_item, _mocker_future))) => Ok(action_item),
            Ok(futures::future::Either::B(_)) => unreachable!(),
            Err(futures::future::Either::A((action_error, _mocker_future))) => Err(action_error),
            Err(x) => panic!("MockTransport yielded an error: {}", x.split().0),
        };

        let ActionChannelPair(request_tx, _) = self.inner
            .try_borrow_mut()
            .unwrap() // cannot be called concurrently
            .action_channels
            .take()
            .unwrap();

        drop(request_tx);

        action_result
    }

    pub fn done() -> futures::future::FutureResult<(), Error> {
        futures::future::ok(())
    }
}

impl transport::Transport for MockTransport {
    type Request = MockRequest;
    type RequestFuture = futures::future::FutureResult<Self::Request, Error>;
    fn request<P: AsRef<str>>(&self, method: Method, url_path: Result<P, Error>) -> Self::RequestFuture {
        futures::future::result(match url_path {
            Err(e) => Err(e),
            Ok(p) => Ok(MockRequest::new(
                self.inner.clone(),
                method,
                p.as_ref().to_string(),
            )),
        })
    }
}

#[derive(Debug)]
pub struct MockRequest {
    inner: Rc<RefCell<Inner>>,
    method: Method,
    url_path: String,
    headers: reqwest::header::Headers,
}

impl MockRequest {
    fn new(inner: Rc<RefCell<Inner>>, method: Method, url_path: String) -> Self {
        MockRequest {
            inner: inner,
            method: method,
            url_path: url_path,
            headers: reqwest::header::Headers::new(),
        }
    }

    pub fn response(&self, status_code: StatusCode) -> MockResponse {
        MockResponse {
            inner: self.inner.clone(),
            status_code: status_code,
            headers: reqwest::header::Headers::new(),
            body: None,
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

    fn accept_application_json(&mut self) {
        self.headers.set(reqwest::header::Accept::json());
    }

    fn send_without_body(self) -> Self::Future {

        let ActionChannelPair(request_tx, response_rx) = self.inner
            .try_borrow_mut()
            .unwrap() // cannot be called concurrently
            .action_channels
            .take()
            .expect(
                "MockTransport cannot begin a request until the action receives the previous response",
            );

        let inner = self.inner.clone();

        Box::new(
            request_tx
                .send(self)
                .map_err(|_send_error| {
                    Error::from("MockTransport failed to send on request channel")
                })
                .and_then(move |request_tx| {
                    response_rx
                        .into_future()
                        .map_err(|_| {
                            Error::from("MockTransport failed to receive on response channel")
                        })
                        .map(move |(response, response_rx)| {

                            // Must move the channel back into the Inner state
                            // *before* the action begins handling the response
                            // so that the mocker can construct another request.
                            inner.try_borrow_mut().unwrap().action_channels =
                                Some(ActionChannelPair(request_tx, response_rx));

                            response.expect("MockTransport exhausted the mock responses")
                        })
                }),
        )
    }
}

#[derive(Debug)]
pub struct MockResponse {
    inner: Rc<RefCell<Inner>>,
    status_code: StatusCode,
    headers: reqwest::header::Headers,
    body: Option<Body>,
}

impl MockResponse {
    pub fn set_json_body<T: Serialize>(&mut self, content: &T) {
        self.headers.set(reqwest::header::ContentType::json());
        self.body = Some(Body::Json(serde_json::to_vec(content).unwrap()));
    }

    pub fn finish(self) -> MockFuture {

        let MockerChannelPair(request_rx, response_tx) = self.inner
            .try_borrow_mut()
            .unwrap()
            .mocker_channels
            .take()
            .unwrap();

        let inner = self.inner.clone();

        Box::new(
            response_tx
                .send(self)
                .map_err(|_send_error| {
                    Error::from("MockTransport failed to send on response channel")
                })
                .and_then(|response_tx| {
                    request_rx
                        .into_future()
                        .map_err(|_| {
                            Error::from("MockTransport failed to receive on request channel")
                        })
                        .map(move |(maybe_request, request_rx)| {

                            // Must move the channel back into the Inner state
                            // *before* the mocker maybe handles another request.
                            inner.try_borrow_mut().unwrap().mocker_channels =
                                Some(MockerChannelPair(request_rx, response_tx));

                            maybe_request
                        })
                }),
        )
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
