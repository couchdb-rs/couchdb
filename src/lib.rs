//! The couchdb crate provides low-level access to individual HTTP actions—e.g.,
//! PUT database, GET document, etc. It handles the menial task of sending
//! requests and receiving responses, thereby allowing application programmers
//! to focus on their business logic.
//!
//! This documentation has been written assuming the application programmer is
//! familiar with the CouchDB API. Descriptions of types, methods, etc. in the
//! couchdb crate should provide just enough information for the programmer to
//! map the crate's concepts onto the CouchDB API and then use the CouchDB
//! documentation as needed to fill in remaining details. Most names in the
//! crate API are identical to the names used in the CouchDB API so as to make
//! this mapping straightforward. There's also a table, below, that [shows the
//! mapping at a high level](#couchdb-api-coverage).
//!
//! One key difference between the couchdb crate's API and the CouchDB API is
//! the crate provides stronger type-safety beyond working with raw strings. For
//! example, applications get and put documents using structured types, and
//! other types, such as revisions and views, are strongly typed as well.
//!
//! To deal with JSON, the couchdb crate relies on Serde and its traits
//! `serde::Serialize` and `serde::Deserialize`. These traits are fundamental to
//! the crate's API—they are not mere implementation details. As such,
//! applications must use these traits when working with documents, views, or
//! any type that is JSON-encoded in the CouchDB API.
//!
//! ## Example: Create a document, read a document
//!
//! This example shows how the couchdb crates thinly wraps the CouchDB API.
//!
//! The following program (1) constructs a `Client` with which to connect to the
//! CouchDB server, (2) creates a database (via the `put_database` method), (3)
//! creates a document within that database (via the `post_to_database` method),
//! and (4) reads the new document (via the `get_document` method).
//!
//! ```no_run
//! extern crate couchdb;
//! extern crate serde_json;
//!
//! // The `Client` type is the entry point for sending all HTTP requests to the
//! // CouchDB server.
//! let client = couchdb::Client::new("http://couchdb-server:5984/").unwrap();
//!
//! // PUT http://couchdb-server:5984/baseball
//! client.put_database("/baseball").run().unwrap();
//!
//! // POST http://couchdb-server:5984/baseball
//! let content = serde_json::builder::ObjectBuilder::new()
//!                   .insert("name", "Babe Ruth")
//!                   .insert("career_hr", 714)
//!                   .unwrap();
//! let (id, rev) = client.post_to_database("/baseball", &content)
//!                       .run()
//!                       .unwrap();
//!
//! // GET http://couchdb-server:5984/baseball/<doc_id>
//! let doc = client.get_document(("/baseball", id.clone()))
//!                 .run()
//!                 .unwrap()
//!                 .unwrap();
//! assert_eq!(id, doc.id);
//! assert_eq!(rev, doc.rev);
//! assert_eq!(content, doc.into_content().unwrap());
//! ```
//!
//! ## CouchDB API coverage
//!
//! In the couchdb crate, the `Client` type is the principal type for
//! communicating with a CouchDB server. All HTTP requests to the CouchDB server
//! go through a `Client` instance.
//!
//! This table maps each CouchDB API resource to the `Client` method that accesses
//! that resource.
//!
//! <table>
//!
//!  <thead>
//!   <tr>
//!    <th>CouchDB URI</th>
//!    <th>HTTP method</th>
//!    <th><code>Client</code> method</th>
//!    <th>Description</th>
//!   </tr>
//!  </thead>
//!
//!  <tbody>
//!   <tr>
//!    <td>
//!     <ul><li><code>/_all_dbs</code></li></ul>
//!    </td>
//!    <td>GET</td>
//!    <td><code>get_all_databases</code></td>
//!    <td>Get list of all databases.</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="5">
//!     <ul><li><code>/db</code></li></ul>
//!    </td>
//!    <td>HEAD</td>
//!    <td><code>head_database</code></td>
//!    <td>Test whether a database exists.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>GET</td>
//!    <td><code>get_database</code></td>
//!    <td>Get meta-information about a database.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td><code>put_database</code></td>
//!    <td>Create a database.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>DELETE</td>
//!    <td><code>delete_database</code></td>
//!    <td>Delete a database.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>POST</td>
//!    <td><code>post_to_database</code></td>
//!    <td>Create a document.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>
//!     <ul>
//!      <li><code>/db/_changes</code></li>
//!     </ul>
//!    </td>
//!    <td>GET</td>
//!    <td><code>get_changes</code></td>
//!    <td>Get changes made to documents in a database.</td>
//!   </tr>
//!
//!   <tr>
//!    <td rowspan="4">
//!     <ul>
//!      <li><code>/db/doc</code><br></li>
//!      <li><code>/db/_design/design-doc</code></li>
//!      <li><code>/db/_local/id</code></li>
//!     </ul>
//!    </td>
//!    <td>HEAD</td>
//!    <td><code>head_document</code></td>
//!    <td>Test whether a document exists.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>GET</td>
//!    <td><code>get_document</code></td>
//!    <td>Get meta-information and application-defined content for a
//!    document.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>PUT</td>
//!    <td><code>put_document</code></td>
//!    <td>Create or update a document.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>DELETE</td>
//!    <td><code>delete_document</code></td>
//!    <td>Delete a document.</td>
//!   </tr>
//!
//!   <tr>
//!    <td>
//!     <ul><li><code>/db/_design/design-doc/_view/view</code></li></ul>
//!    </td>
//!    <td>GET</td>
//!    <td><code>get_view</code></td>
//!    <td>Execute a view.</td>
//!   </tr>
//!
//!  </tbody>
//! </table>

extern crate hyper;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tempdir;
extern crate url;

pub mod action;
pub mod testing;

mod client;
mod dbtype;
mod error;
mod path;

pub use client::{Client, IntoUrl};
pub use dbtype::{Database, ChangeItem, ChangeItemBuilder, ChangeResult, ChangeResultBuilder,
                 Changes, ChangesBuilder, DatabaseName, Design, DesignBuilder, DesignDocumentName,
                 Document, DocumentId, DocumentName, ErrorResponse, ViewName, Revision,
                 ViewFunction, ViewFunctionBuilder, ViewFunctionMap, ViewResult, ViewRow};
pub use path::{IntoDatabasePath, IntoDesignDocumentPath, IntoDocumentPath, IntoViewPath,
               DatabasePath, DesignDocumentPath, DocumentPath, ViewPath};
pub use error::Error;
