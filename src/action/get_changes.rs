use hyper;
use std;

use ChangeResult;
use Changes;
use ChangesBuilder;
use Error;
use ErrorResponse;
use IntoDatabasePath;
use action::{self, Action, Request, Response};
use client::ClientState;
use dbtype::ChangeLine;

/// Handler that receives a single change result when using the action's
/// continuous feed.
pub trait ChangesEvent {

    /// Method that's called exactly once for each change result.
    fn change_event(&self, ChangeResult);
}

impl<T> ChangesEvent for T where T: Fn(ChangeResult)
{
    fn change_event(&self, result: ChangeResult) {
        self(result);
    }
}

enum Feed<'a> {
    Normal,
    Longpoll,
    Continuous(Box<ChangesEvent + 'a>),
}

impl<'a> std::fmt::Display for Feed<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Feed::Normal => write!(f, "normal"),
            Feed::Longpoll => write!(f, "longpoll"),
            Feed::Continuous(..) => write!(f, "continuous"),
        }
    }
}

enum QueryIterator<'a> {
    Feed(&'a QueryParams<'a>),
    Timeout(&'a QueryParams<'a>),
    Since(&'a QueryParams<'a>),
    Done,
}

impl<'a> Iterator for QueryIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self {
                &mut QueryIterator::Feed(params) => {
                    *self = QueryIterator::Timeout(params);
                    if let Some(ref feed) = params.feed {
                        return Some(("feed".to_string(), feed.to_string()));
                    }
                }
                &mut QueryIterator::Timeout(params) => {
                    *self = QueryIterator::Since(params);
                    if let Some(ref timeout) = params.timeout {
                        return Some(("timeout".to_string(), timeout.to_string()));
                    }
                }
                &mut QueryIterator::Since(params) => {
                    *self = QueryIterator::Done;
                    if let Some(ref seq) = params.since {
                        return Some(("since".to_string(), seq.to_string()));
                    }
                }
                &mut QueryIterator::Done => {
                    return None;
                }
            }
        }
    }
}

/// Type for the `since` query parameter when getting database changes.
///
/// # Examples
///
/// Applications may construct a `ChangesSince` directly from a number, or
/// convert a `ChangesSince` to a string.
///
/// ```
/// use couchdb::action::ChangesSince;
/// let x: ChangesSince = 42.into();
/// assert_eq!("42", x.to_string());
/// ```
///
#[derive(Debug, Eq, PartialEq)]
pub enum ChangesSince {
    /// A literal sequence number.
    SequenceNumber(u64),

    /// The `now` value.
    Now,
}

impl std::fmt::Display for ChangesSince {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &ChangesSince::SequenceNumber(x) => x.fmt(f),
            &ChangesSince::Now => write!(f, "now"),
        }
    }
}

impl From<u64> for ChangesSince {
    fn from(seq: u64) -> Self {
        ChangesSince::SequenceNumber(seq)
    }
}

#[derive(Default)]
struct QueryParams<'a> {
    feed: Option<Feed<'a>>,
    timeout: Option<u64>,
    since: Option<ChangesSince>,
}

impl<'a> QueryParams<'a> {
    fn is_default(&self) -> bool {
        self.feed.is_none() && self.timeout.is_none() && self.since.is_none()
    }

    fn iter(&self) -> QueryIterator {
        QueryIterator::Feed(self)
    }
}

/// Action to get changes made to documents in a database.
///
/// # Return
///
/// This action returns a list of changes to documents that have occurred within
/// the database. However, if using the continuous feed then the returned list
/// is empty and the changes are instead returned via an event handler. See the
/// [`continuous`](#method.continuous) method for more information.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
///
/// * `Error::BadRequest`: Bad request.
///
pub struct GetChanges<'a, P>
    where P: IntoDatabasePath
{
    client_state: &'a ClientState,
    path: P,
    query: QueryParams<'a>,
}

impl<'a, P: IntoDatabasePath> GetChanges<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        GetChanges {
            client_state: client_state,
            path: path,
            query: Default::default(),
        }
    }

    /// Sets the `feed` query parameter to do long-polling.
    pub fn longpoll(mut self) -> Self {
        self.query.feed = Some(Feed::Longpoll);
        self
    }

    /// Sets the `feed` query parameter to receive a continuous feed.
    ///
    /// The continuous feed behaves differently from other feeds. When using the
    /// continuous feed, the action returns an empty list of change results and
    /// the change results are instead returned via the `handler` argument,
    /// which is called exactly once for each change result.
    ///
    pub fn continuous<H: 'a + ChangesEvent>(mut self, handler: H) -> Self {
        self.query.feed = Some(Feed::Continuous(Box::new(handler)));
        self
    }

    /// Sets the `timeout` query parameter.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        let milliseconds = 1000 * timeout.as_secs() + timeout.subsec_nanos() as u64 / 1_000_000;
        self.query.timeout = Some(milliseconds);
        self
    }

    /// Sets the `since` query parameter.
    ///
    /// The `since` query parameter causes the action to return change results
    /// starting after the given sequence number.
    ///
    pub fn since<S: Into<ChangesSince>>(mut self, seq: S) -> Self {
        self.query.since = Some(seq.into());
        self
    }

    impl_action_public_methods!(Changes);
}

impl<'a, P: IntoDatabasePath> Action for GetChanges<'a, P> {
    type Output = Changes;
    type State = Feed<'a>;

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let db_path = try!(self.path.into_database_path());
        let uri = {
            let mut uri = db_path.into_uri(self.client_state.uri.clone());
            uri.path_mut().unwrap().push("_changes".to_string());
            if !self.query.is_default() {
                uri.set_query_from_pairs(self.query.iter());
            }
            uri
        };
        let request = Request::new(hyper::Get, uri).set_accept_application_json();
        let feed = self.query.feed.unwrap_or(Feed::Normal);
        Ok((request, feed))
    }

    fn take_response<R>(mut response: R, feed: Self::State) -> Result<Self::Output, Error>
        where R: Response
    {
        match response.status() {
            hyper::Ok => {
                try!(response.content_type_must_be_application_json());
                if let Feed::Continuous(handler) = feed {
                    loop {
                        match try!(response.decode_json_line::<ChangeLine>()) {
                            ChangeLine::Event(result) => handler.change_event(result),
                            ChangeLine::End { last_seq } => {
                                try!(response.no_more_content());
                                return Ok(ChangesBuilder::new(last_seq).unwrap());
                            }
                        }
                    }
                } else {
                    response.decode_json_all::<Changes>()
                }
            }
            hyper::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            status => Err(Error::UnexpectedHttpStatus { got: status }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;
    use serde_json;

    use ChangeResultBuilder;
    use ChangesBuilder;
    use DatabasePath;
    use action::{Action, JsonResponse};
    use client::ClientState;
    use super::{ChangesSince, Feed, GetChanges, QueryParams};

    #[test]
    fn feed_display() {
        assert_eq!("normal", format!("{}", Feed::Normal));
        assert_eq!("longpoll", format!("{}", Feed::Longpoll));
        assert_eq!("continuous",
                   format!("{}", Feed::Continuous(Box::new(|_| {}))));
    }

    #[test]
    fn changes_since_display() {
        assert_eq!("42", format!("{}", ChangesSince::SequenceNumber(42)));
        assert_eq!("now", format!("{}", ChangesSince::Now));
    }

    #[test]
    fn changes_since_eq() {
        let a = ChangesSince::SequenceNumber(42);
        let b = ChangesSince::SequenceNumber(42);
        assert!(a == b);

        let a = ChangesSince::SequenceNumber(17);
        let b = ChangesSince::SequenceNumber(42);
        assert!(a != b);

        let a = ChangesSince::Now;
        let b = ChangesSince::SequenceNumber(42);
        assert!(a != b);

        let a = ChangesSince::Now;
        let b = ChangesSince::Now;
        assert!(a == b);
    }

    #[test]
    fn changes_since_from_number() {
        let expected = ChangesSince::SequenceNumber(42);
        let got = ChangesSince::from(42);
        assert_eq!(expected, got);
    }

    #[test]
    fn query_iterator() {
        let query = QueryParams {
            feed: Some(Feed::Longpoll),
            timeout: Some(42),
            since: Some(17.into()),
        };
        let expected = vec![("feed".to_string(), "longpoll".to_string()),
                            ("timeout".to_string(), "42".to_string()),
                            ("since".to_string(), "17".to_string())];
        let got = query.iter().collect::<Vec<_>>();
        assert_eq!(expected, got);
    }

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetChanges::new(&client_state, "/db");
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/db/_changes");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn make_request_longpoll() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetChanges::new(&client_state, "/db").longpoll();
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/db/_changes?feed=longpoll");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn make_request_continuous() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetChanges::new(&client_state, "/db").continuous(|_| {});
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request,
                            "http://example.com:1234/db/_changes?feed=continuous");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn make_request_timeout() {
        use std::time::Duration;
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetChanges::new(&client_state, "/db").timeout(Duration::new(12, 34_000_000));
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/db/_changes?timeout=12034");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn make_request_since() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetChanges::new(&client_state, "/db").since(42);
        let (request, _) = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/db/_changes?since=42");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn take_response_ok() {
        let expected = ChangesBuilder::new(11)
                           .build_result(6, "6478c2ae800dfc387396d14e1fc39626", |x| {
                               x.build_change_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337",
                                                           |x| x)
                           })
                           .unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("last_seq", 11)
                         .insert_array("results", |x| {
                             x.push_object(|x| {
                                 x.insert_array("changes", |x| {
                                      x.push_object(|x| {
                                          x.insert("rev", "2-7051cbe5c8faecd085a3fa619e6e6337")
                                      })
                                  })
                                  .insert("id", "6478c2ae800dfc387396d14e1fc39626")
                                  .insert("seq", 6)
                             })
                         })
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let got = GetChanges::<DatabasePath>::take_response(response, Feed::Normal).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn take_response_ok_longpoll() {
        let expected = ChangesBuilder::new(11)
                           .build_result(6, "6478c2ae800dfc387396d14e1fc39626", |x| {
                               x.build_change_from_rev_str("2-7051cbe5c8faecd085a3fa619e6e6337",
                                                           |x| x)
                           })
                           .unwrap();
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("last_seq", 11)
                         .insert_array("results", |x| {
                             x.push_object(|x| {
                                 x.insert_array("changes", |x| {
                                      x.push_object(|x| {
                                          x.insert("rev", "2-7051cbe5c8faecd085a3fa619e6e6337")
                                      })
                                  })
                                  .insert("id", "6478c2ae800dfc387396d14e1fc39626")
                                  .insert("seq", 6)
                             })
                         })
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let got = GetChanges::<DatabasePath>::take_response(response, Feed::Longpoll).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn take_response_ok_continuous() {
        use std::sync::Mutex;

        let body = "{\"seq\":6,\"id\":\"6478c2ae800dfc387396d14e1fc39626\",\"changes\":[{\"rev\":\
                    \"2-7051cbe5c8faecd085a3fa619e6e6337\"}]}\n{\"last_seq\":11}\n";
        let response = JsonResponse::new_from_string(hyper::Ok, body);

        let change_results = Mutex::new(Vec::new());
        {
            let handler = |result| {
                change_results.lock().unwrap().push(result);
            };

            let expected = ChangesBuilder::new(11).unwrap();
            let got =
                GetChanges::<DatabasePath>::take_response(response,
                                                          Feed::Continuous(Box::new(handler)))
                    .unwrap();
            assert_eq!(expected, got);
        }

        let expected = vec![ChangeResultBuilder::new(6, "6478c2ae800dfc387396d14e1fc39626")
                                .build_change("2-7051cbe5c8faecd085a3fa619e6e6337"
                                                  .parse()
                                                  .unwrap(),
                                              |x| x)
                                .unwrap()];
        assert_eq!(expected, change_results.into_inner().unwrap());
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "bad_request")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = GetChanges::<DatabasePath>::take_response(response, Feed::Normal);
        expect_couchdb_error!(got, BadRequest);
    }
}
