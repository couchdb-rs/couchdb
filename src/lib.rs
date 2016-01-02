// FIXME: Rewrite the crate-level doc comment.

// The couchdb crate is a thin wrapper around the CouchDB API, providing
// low-level access to individual CouchDB commands—e.g., PUT database, GET
// document, etc. The goal is for the crate to deal with the menial task of
// sending HTTP requests and receiving HTTP responses and allow application
// writers to focus on their business logic.
//
// This documentation has been written with the assumption that the reader is
// familiar with the CouchDB API. Descriptions of types, methods, etc. in the
// couchdb crate should provide just enough information for the reader to map
// the crate's concepts onto the CouchDB API and then use the CouchDB
// documentation to fill in the remaining details. Most names in the crate are
// identical to the names used in the CouchDB API so as to make this mapping
// straightforward.
//
// The couchdb crate provides applications with type-safety beyond working with
// raw strings. Applications get and put documents as structured types. Other
// types, such as revisions and views, are strongly typed as well.
//
// To deal with JSON, the couchdb crate relies on Serde and its traits
// `serde::Serialize` and `serde::Deserialize`. These traits are fundamental to
// the crate's API—they are not mere implementation details. As such,
// applications must use these traits when working with documents, views, or
// any type that is JSON-encoded in the CouchDB API.
//
// # Examples
//
// Here are some examples of how to use the couchdb crate.
//
// ## Example #1: Put a document
//
// This example creates a database named “baseball” and puts a single document
// into it.
//
// ```ignore
// extern crate couchdb;
//
// #[derive(Serialize)]
// struct Player {
//     name: String,
//     hits: u32,
//     home_runs: u32,
// }
//
// let client = couchdb::Client::new("http://your_database/").unwrap();
//
// client.put_database("baseball").run().unwrap();
//
// let doc = Player {
//     name: "Babe Ruth".to_string(),
//     hits: 2873,
//     home_runs: 714,
// };
//
// client.put_document("baseball/babe_ruth", &doc).run()
//                                                .unwrap();
// ```
//
// ## Example #2: Get a document
//
// This example gets the document created in the previous example.
//
// ```ignore
// extern crate couchdb;
//
// #[derive(Deserialize)]
// struct Player {
//   name: String,
//   hits: u32,
//   home_runs: u32,
// }
//
// let client = couchdb::Client::new("http://your_database/").unwrap();
//
// let doc = client.get_document::<_, Player>("baseball/babe_ruth").run()
//                                                                 .unwrap()
//                                                                 .unwrap();
//
// assert_eq!(doc.path, "baseball/babe_ruth".into());
// assert_eq!(doc.content.name, "Babe Ruth".to_string());
// assert_eq!(doc.hits, 2873);
// assert_eq!(doc.home_runs, 714);
// ```
//
// # CouchDB API coverage
//
// In the couchdb crate, the `Client` type is the principal type for
// communicating with a CouchDB server. All HTTP requests to the CouchDB server
// go through a `Client` instance.
//
// This table maps each CouchDB API resource to the `Client` method that accesses
// that resource.
//
// <table>
//  <thead>
//   <tr>
//    <th>URI</td>
//    <th>HTTP method</td>
//    <th><span style="font-family:monospace;">Client</span> method</td>
//    <th>Description</th>
//   </tr>
//  </thead>
//  <tbody>
//   <tr>
//    <td style="font-family:monospace;">
//     <ul><li>/_all_dbs</li></ul>
//    </td>
//    <td>GET</td>
//    <td style="font-family:monospace;">get_all_databases</td>
//    <td>Get list of all databases</td>
//   </tr>
//   <tr>
//    <td style="font-family:monospace;" rowspan="5">
//     <ul><li>/db</li></ul>
//    </td>
//    <td>HEAD</td>
//    <td style="font-family:monospace;">head_database</td>
//    <td>Test whether a database exists</td>
//   </tr>
//   <tr>
//    <td>GET</td>
//    <td style="font-family:monospace;">get_database</td>
//    <td>Get meta-information about a database</td>
//   </tr>
//   <tr>
//    <td>PUT</td>
//    <td style="font-family:monospace;">put_database</td>
//    <td>Create a database</td>
//   </tr>
//   <tr>
//    <td>DELETE</td>
//    <td style="font-family:monospace;">delete_database</td>
//    <td>Delete a database</td>
//   </tr>
//   <tr>
//    <td>POST</td>
//    <td style="font-family:monospace;">post_to_database</td>
//    <td>Create a document</td>
//   </tr>
//   <tr>
//    <td style="font-family:monospace;" rowspan="4">
//     <ul>
//      <li>/db/doc<br></li>
//      <li>/db/_design/design-doc</li>
//      <li>/db/_local/id</li>
//     </ul>
//    </td>
//    <td>HEAD</td>
//    <td style="font-family:monospace;">head_document</td>
//    <td>Test whether a document exists</td>
//   </tr>
//   <tr>
//    <td>GET</td>
//    <td style="font-family:monospace;">get_document</td>
//    <td>Get meta-information and application-defined content for a
//    document</td>
//   </tr>
//   <tr>
//    <td>PUT</td>
//    <td style="font-family:monospace;">put_document</td>
//    <td>Create or update a document</td>
//   </tr>
//   <tr>
//    <td>DELETE</td>
//    <td style="font-family:monospace;">delete_document</td>
//    <td>Delete a document</td>
//   </tr>
//   <tr>
//    <td style="font-family:monospace;">
//     <ul><li>/db/_design/design-doc/_view/view</li></ul>
//    </td>
//    <td>GET</td>
//    <td style="font-family:monospace;">get_view</td>
//    <td>Execute a view</td>
//   </tr>
//  </tbody>
// </table>

extern crate hyper;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tempdir;
extern crate url;

pub mod command;

mod client;
mod dbtype;
mod error;
mod json;
mod path;
mod server;

pub use client::{Client, IntoUrl};
pub use dbtype::{Database, DatabaseName, Design, DesignBuilder, DesignDocumentName, Document,
                 DocumentId, DocumentName, ErrorResponse, ViewName, Revision, ViewFunction,
                 ViewFunctionMap, ViewResult, ViewRow};
pub use path::{IntoDatabasePath, IntoDesignDocumentPath, IntoDocumentPath, IntoViewPath,
               DatabasePath, DesignDocumentPath, DocumentPath, ViewPath};
pub use error::Error;
pub use server::Server;
