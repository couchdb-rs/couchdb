use hyper;
use serde;

use Error;
use IntoDatabasePath;
use IntoDocumentPath;
use IntoViewPath;
use Revision;
use command;

/// Trait for converting a type into a URI.
pub trait IntoUrl: hyper::client::IntoUrl {}
impl<T: hyper::client::IntoUrl> IntoUrl for T {}

// Encapsulates state borrowed by command instances. The reason this struct is
// separated from Client is so that the struct fields aren't exposed in this
// crate's public API.
pub struct ClientState {
    pub http_client: hyper::Client,
    pub uri: hyper::Url,
}

/// Entry point for communicating with a CouchDB server.
///
/// The `Client` is the principal type for communicating with a CouchDB server.
/// All CouchDB commands (e.g., PUT database, GET document, etc.) go through a
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
    pub fn new<U: IntoUrl>(uri: U) -> Result<Client, Error> {
        Ok(Client {
            state: ClientState {
                http_client: hyper::Client::new(),
                uri: try!(uri.into_url()
                             .or_else(|e| Err(Error::UriParse { cause: e }))),
            },
        })
    }

    /// Gets the server URI the client connects to.
    pub fn uri(&self) -> &hyper::Url {
        &self.state.uri
    }

    /// Builds a command to GET all database names.
    pub fn get_all_databases(&'a self) -> command::GetAllDatabases<'a> {
        command::GetAllDatabases::new(&self.state)
    }

    /// Builds a command to HEAD a database.
    pub fn head_database<P: IntoDatabasePath>(&'a self, path: P) -> command::HeadDatabase<'a, P> {
        command::HeadDatabase::new(&self.state, path)
    }

    /// Builds a command to GET a database.
    pub fn get_database<P: IntoDatabasePath>(&'a self, path: P) -> command::GetDatabase<'a, P> {
        command::GetDatabase::new(&self.state, path)
    }

    /// Builds a command to PUT a database.
    pub fn put_database<P: IntoDatabasePath>(&'a self, path: P) -> command::PutDatabase<'a, P> {
        command::PutDatabase::new(&self.state, path)
    }

    /// Builds a command to DELETE a database.
    pub fn delete_database<P: IntoDatabasePath>(&'a self,
                                                path: P)
                                                -> command::DeleteDatabase<'a, P> {
        command::DeleteDatabase::new(&self.state, path)
    }

    /// Builds a command to POST to a database.
    pub fn post_to_database<P: IntoDatabasePath, T: serde::Serialize>
        (&'a self,
         path: P,
         doc_content: &'a T)
         -> command::PostToDatabase<'a, P, T> {
        command::PostToDatabase::new(&self.state, path, doc_content)
    }

    /// Builds a command to HEAD a document.
    pub fn head_document<P: IntoDocumentPath>(&'a self, path: P) -> command::HeadDocument<'a, P> {
        command::HeadDocument::new(&self.state, path)
    }

    /// Builds a command to GET a document.
    pub fn get_document<P: IntoDocumentPath, T: serde::Deserialize>
        (&'a self,
         path: P)
         -> command::GetDocument<'a, P, T> {
        command::GetDocument::new(&self.state, path)
    }

    /// Builds a command to PUT a document.
    pub fn put_document<P: IntoDocumentPath, T: serde::Serialize>
        (&'a self,
         path: P,
         doc_content: &'a T)
         -> command::PutDocument<'a, P, T> {
        command::PutDocument::new(&self.state, path, doc_content)
    }

    /// Builds a command to DELETE a document.
    pub fn delete_document<P: IntoDocumentPath>(&'a self,
                                                path: P,
                                                rev: &'a Revision)
                                                -> command::DeleteDocument<'a, P> {
        command::DeleteDocument::new(&self.state, path, rev)
    }

    /// Builds a command to GET a view.
    pub fn get_view<P: IntoViewPath,
                    K: serde::Deserialize + serde::Serialize,
                    V: serde::Deserialize>
        (&'a self,
         path: P)
         -> command::GetView<'a, P, K, V> {
        command::GetView::new(&self.state, path)
    }
}
