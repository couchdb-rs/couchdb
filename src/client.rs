use hyper;
use serde;

use Error;
use IntoDatabasePath;
use IntoDocumentPath;
use IntoViewPath;
use Revision;
use action;

/// Trait for converting a type into a URI.
pub trait IntoUrl: hyper::client::IntoUrl {}
impl<T: hyper::client::IntoUrl> IntoUrl for T {}

// Encapsulates state borrowed by action instances. The reason this struct is
// separated from Client is so that the struct fields aren't exposed in this
// crate's public API.
pub struct ClientState {
    pub http_client: hyper::Client,
    pub uri: hyper::Url,
}

impl ClientState {
    pub fn new<U: IntoUrl>(uri: U) -> Result<Self, Error> {
        let client_state = ClientState {
            http_client: hyper::Client::new(),
            uri: try!(uri.into_url()
                         .or_else(|e| Err(Error::UriParse { cause: e }))),
        };
        Ok(client_state)
    }
}

/// Entry point for communicating with a CouchDB server.
///
/// The `Client` is the principal type for communicating with a CouchDB server.
/// All CouchDB actions (e.g., PUT database, GET document, etc.) go through a
/// `Client`.
///
/// A `Client` communicates with exactly one CouchDB server, as specified by the
/// URI set when the `Client` is constructed.
///
pub struct Client {
    state: ClientState,
}

impl<'a> Client {
    /// Constructs a CouchDB client.
    pub fn new<U: IntoUrl>(uri: U) -> Result<Self, Error> {
        let client = Client { state: try!(ClientState::new(uri)) };
        Ok(client)
    }

    /// Gets the server URI the client connects to.
    pub fn uri(&self) -> &hyper::Url {
        &self.state.uri
    }

    /// Builds an action to GET all database names.
    pub fn get_all_databases(&'a self) -> action::GetAllDatabases<'a> {
        action::GetAllDatabases::new(&self.state)
    }

    /// Builds an action to HEAD a database.
    pub fn head_database<P: IntoDatabasePath>(&'a self, path: P) -> action::HeadDatabase<'a, P> {
        action::HeadDatabase::new(&self.state, path)
    }

    /// Builds an action to GET a database.
    pub fn get_database<P: IntoDatabasePath>(&'a self, path: P) -> action::GetDatabase<'a, P> {
        action::GetDatabase::new(&self.state, path)
    }

    /// Builds an action to PUT a database.
    pub fn put_database<P: IntoDatabasePath>(&'a self, path: P) -> action::PutDatabase<'a, P> {
        action::PutDatabase::new(&self.state, path)
    }

    /// Builds an action to DELETE a database.
    pub fn delete_database<P: IntoDatabasePath>(&'a self,
                                                path: P)
                                                -> action::DeleteDatabase<'a, P> {
        action::DeleteDatabase::new(&self.state, path)
    }

    /// Builds an action to POST to a database.
    pub fn post_database<P: IntoDatabasePath, T: serde::Serialize>
        (&'a self,
         path: P,
         doc_content: &'a T)
         -> action::PostDatabase<'a, P, T> {
        action::PostDatabase::new(&self.state, path, doc_content)
    }

    #[doc(hidden)]
    pub fn post_to_database<P: IntoDatabasePath, T: serde::Serialize>
        (&'a self,
         path: P,
         doc_content: &'a T)
         -> action::PostDatabase<'a, P, T> {
        action::PostDatabase::new(&self.state, path, doc_content)
    }

    /// Builds an action to GET changes made to a database.
    pub fn get_changes<P>(&'a self, path: P) -> action::GetChanges<'a, P>
        where P: IntoDatabasePath
    {
        action::GetChanges::new(&self.state, path)
    }

    /// Builds an action to HEAD a document.
    pub fn head_document<P: IntoDocumentPath>(&'a self, path: P) -> action::HeadDocument<'a, P> {
        action::HeadDocument::new(&self.state, path)
    }

    /// Builds an action to GET a document.
    pub fn get_document<P: IntoDocumentPath>(&'a self, path: P) -> action::GetDocument<'a, P> {
        action::GetDocument::new(&self.state, path)
    }

    /// Builds an action to PUT a document.
    pub fn put_document<P: IntoDocumentPath, T: serde::Serialize>
        (&'a self,
         path: P,
         doc_content: &'a T)
         -> action::PutDocument<'a, P, T> {
        action::PutDocument::new(&self.state, path, doc_content)
    }

    /// Builds an action to DELETE a document.
    pub fn delete_document<P: IntoDocumentPath>(&'a self,
                                                path: P,
                                                rev: &'a Revision)
                                                -> action::DeleteDocument<'a, P> {
        action::DeleteDocument::new(&self.state, path, rev)
    }

    /// Builds an action to GET a view.
    pub fn get_view<P: IntoViewPath,
                    K: serde::Deserialize + serde::Serialize,
                    V: serde::Deserialize>
        (&'a self,
         path: P)
         -> action::GetView<'a, P, K, V> {
        action::GetView::new(&self.state, path)
    }
}
