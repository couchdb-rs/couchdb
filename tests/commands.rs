extern crate couchdb;
extern crate serde_json;

use std::collections::HashSet;

macro_rules! expect_error_database_exists {
    ($result:expr) => {
        match $result {
            Ok(..) => {
                panic!("Got unexpected OK result");
            }
            Err(ref e) => {
                match *e {
                    couchdb::Error::DatabaseExists(..) => (),
                    _ => {
                        panic!("Got unexpected error: {}", e);
                    }
                }
            }
        }
    }
}

macro_rules! expect_error_document_conflict {
    ($result:expr) => {
        match $result {
            Ok(..) => {
                panic!("Got unexpected OK result");
            }
            Err(ref e) => {
                match *e {
                    couchdb::Error::DocumentConflict(..) => (),
                    _ => {
                        panic!("Got unexpected error: {}", e);
                    }
                }
            }
        }
    }
}

macro_rules! expect_error_not_found_none {
    ($result:expr) => {
        match $result {
            Ok(..) => {
                panic!("Got unexpected OK result");
            }
            Err(ref e) => {
                match *e {
                    couchdb::Error::NotFound(ref response) => {
                        match *response {
                            None => (),
                            Some(ref response) => {
                                panic!("Expected None error response, got Some: {}", response);
                            }
                        }
                    }
                    _ => {
                        panic!("Got unexpected error: {}", e);
                    }
                }
            }
        }
    }
}

macro_rules! expect_error_not_found_some {
    ($result:expr) => {
        match $result {
            Ok(..) => {
                panic!("Got unexpected OK result");
            }
            Err(ref e) => {
                match *e {
                    couchdb::Error::NotFound(ref response) => {
                        match *response {
                            Some(..) => (),
                            None => {
                                panic!("Expected Some error response, got None");
                            }
                        }
                    }
                    _ => {
                        panic!("Got unexpected error: {}", e);
                    }
                }
            }
        }
    }
}

fn make_server_and_client() -> (couchdb::Server, couchdb::Client) {
    let server = couchdb::Server::new().unwrap();
    let client = couchdb::Client::new(server.uri()).unwrap();
    (server, client)
}

#[test]
fn get_all_databases_ok() {
    let (_server, client) = make_server_and_client();
    let expected = vec!["_replicator", "_users"]
                       .into_iter()
                       .map(|x| String::from(x))
                       .collect::<HashSet<_>>();
    let got = client.get_all_databases()
                    .run()
                    .unwrap()
                    .into_iter()
                    .map(|x| String::from(x))
                    .collect::<HashSet<_>>();
    assert_eq!(expected, got);
}

#[test]
fn head_database_ok() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    client.head_database("/foo").run().unwrap();
}

#[test]
fn head_database_nok_database_does_not_exist() {
    let (_server, client) = make_server_and_client();
    let got = client.head_database("/foo").run();
    expect_error_not_found_none!(got);
}

#[test]
fn get_database_ok() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    let got = client.get_database("/foo").run().unwrap();
    assert_eq!(couchdb::DatabaseName::from("foo"), got.db_name);
    assert_eq!(0, got.update_seq);
    assert_eq!(0, got.committed_update_seq);
    assert_eq!(0, got.doc_count);
    assert_eq!(0, got.doc_del_count);
    assert_eq!(0, got.data_size);
    assert_eq!(0, got.purge_seq);
    assert_eq!(false, got.compact_running);
}

#[test]
fn get_database_nok_database_does_not_exist() {
    let (_server, client) = make_server_and_client();
    let got = client.get_database("/foo").run();
    expect_error_not_found_some!(got);
}

#[test]
fn put_database_ok() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    client.head_database("/foo").run().unwrap();
}

#[test]
fn put_database_nok_database_already_exists() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    let got = client.put_database("/foo").run();
    expect_error_database_exists!(got);
}

#[test]
fn delete_database_ok() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    client.delete_database("/foo").run().unwrap();
    let got = client.head_database("/foo").run();
    expect_error_not_found_none!(got);
}

#[test]
fn delete_database_nok_database_does_not_exist() {
    let (_server, client) = make_server_and_client();
    let got = client.delete_database("/foo").run();
    expect_error_not_found_some!(got);
}

#[test]
fn post_to_database_ok() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let doc = client.get_document::<_, serde_json::Value>(("/baseball", doc_id.clone()))
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc.id, doc_id);
    assert_eq!(doc.revision, rev);
    assert_eq!(doc.content, source_content);
}

#[test]
fn post_to_database_nok_database_does_not_exist() {
    let (_server, client) = make_server_and_client();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let got = client.post_to_database("/baseball", &source_content).run();
    expect_error_not_found_some!(got);
}

#[test]
fn head_document_ok_without_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (_rev, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let got = client.head_document(("/baseball", doc_id)).run().unwrap();
    assert!(got.is_some());
}

#[test]
fn head_document_ok_fresh_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let got = client.head_document(("/baseball", doc_id)).if_none_match(&rev).run().unwrap();
    assert!(got.is_none());
}

#[test]
fn head_document_ok_stale_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev1, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let _rev2 = client.put_document(("/baseball", doc_id.clone()), &source_content)
                      .if_match(&rev1)
                      .run()
                      .unwrap();
    let got = client.head_document(("/baseball", doc_id)).if_none_match(&rev1).run().unwrap();
    assert!(got.is_some());
}

#[test]
fn head_document_nok_document_does_not_exist() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    let got = client.head_document("/foo/bar").run();
    expect_error_not_found_none!(got);
}

#[test]
fn get_document_ok_without_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let got = client.get_document::<_, serde_json::Value>(("/baseball", doc_id.clone()))
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc_id, got.id);
    assert_eq!(rev, got.revision);
    assert_eq!(source_content, got.content);
}

#[test]
fn get_document_ok_fresh_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let got = client.get_document::<_, serde_json::Value>(("/baseball", doc_id.clone()))
                    .if_none_match(&rev)
                    .run()
                    .unwrap();
    assert!(got.is_none());
}

#[test]
fn get_document_ok_stale_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev1, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let rev2 = client.put_document(("/baseball", doc_id.clone()), &source_content)
                     .if_match(&rev1)
                     .run()
                     .unwrap();
    let got = client.get_document::<_, serde_json::Value>(("/baseball", doc_id.clone()))
                    .if_none_match(&rev1)
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc_id, got.id);
    assert_eq!(rev2, got.revision);
    assert_eq!(source_content, got.content);
}

#[test]
fn get_document_nok_document_does_not_exist() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    let got = client.get_document::<_, serde_json::Value>("/foo/bar").run();
    expect_error_not_found_some!(got);
}

#[test]
fn put_document_ok_new_document() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let rev = client.put_document("/baseball/babe_ruth", &source_content).run().unwrap();
    let got = client.get_document::<_, serde_json::Value>("/baseball/babe_ruth")
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(couchdb::DocumentId::from("babe_ruth"), got.id);
    assert_eq!(rev, got.revision);
    assert_eq!(source_content, got.content);
}

#[test]
fn put_document_ok_update_document() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev1, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .insert("career_hits", 2873)
                             .unwrap();
    let rev2 = client.put_document(("/baseball", doc_id.clone()), &source_content)
                     .if_match(&rev1)
                     .run()
                     .unwrap();
    let got = client.get_document::<_, serde_json::Value>(("/baseball", doc_id.clone()))
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc_id, got.id);
    assert_eq!(rev2, got.revision);
    assert_eq!(source_content, got.content);
}

#[test]
fn put_document_nok_stale_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev1, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .insert("career_hits", 2873)
                             .unwrap();
    let _rev2 = client.put_document(("/baseball", doc_id.clone()), &source_content)
                      .if_match(&rev1)
                      .run()
                      .unwrap();
    let got = client.put_document(("/baseball", doc_id.clone()), &source_content)
                    .if_match(&rev1)
                    .run();
    expect_error_document_conflict!(got);
}

#[test]
fn put_document_nok_database_does_not_exist() {
    let (_server, client) = make_server_and_client();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let got = client.put_document("/baseball/babe_ruth", &source_content).run();
    expect_error_not_found_some!(got);
}

#[test]
fn delete_document_ok() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    client.delete_document(("/baseball", doc_id.clone()), &rev).run().unwrap();
    let got = client.head_document(("/baseball", doc_id)).run();
    expect_error_not_found_none!(got);
}

#[test]
fn delete_document_nok_stale_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (rev1, doc_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let _rev2 = client.put_document(("/baseball", doc_id.clone()), &source_content)
                      .if_match(&rev1)
                      .run()
                      .unwrap();
    let got = client.delete_document(("/baseball", doc_id.clone()), &rev1).run();
    expect_error_document_conflict!(got);
}

#[test]
fn delete_document_nok_document_does_not_exist() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    let rev = couchdb::Revision::parse("1-12345678123456781234567812345678").unwrap();
    let got = client.delete_document("/foo/bar", &rev).run();
    expect_error_not_found_some!(got);
}

#[test]
fn get_view_empty_result() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    let design = couchdb::DesignBuilder::new()
                     .insert_view("qux",
                                  couchdb::ViewFunction {
                                      map: "function(doc) {}".to_string(),
                                      reduce: None,
                                  })
                     .unwrap();
    client.put_document("/foo/_design/bar", &design).run().unwrap();
    let got = client.get_view::<_, (), ()>("/foo/_design/bar/_view/qux").run().unwrap();
    assert_eq!(Some(0), got.total_rows);
    assert_eq!(Some(0), got.offset);
    assert!(got.rows.is_empty());
}

#[test]
fn get_view_nonempty_result() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (_, babe_ruth_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Hank Aaron")
                             .insert("career_hr", 755)
                             .unwrap();
    let (_, hank_aaron_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let design = couchdb::DesignBuilder::new()
                     .insert_view("by_career_hr",
                                  couchdb::ViewFunction {
                                      map: "function(doc) { emit(doc.name, doc.career_hr); }"
                                               .to_string(),
                                      reduce: None,
                                  })
                     .unwrap();
    client.put_document("/baseball/_design/stat", &design).run().unwrap();
    let got = client.get_view::<_, String, i32>("/baseball/_design/stat/_view/by_career_hr")
                    .run()
                    .unwrap();
    assert_eq!(Some(2), got.total_rows);
    assert_eq!(Some(0), got.offset);
    assert_eq!(vec![couchdb::ViewRow {
                        id: Some(babe_ruth_id),
                        key: Some("Babe Ruth".to_string()),
                        value: 714,
                    },
                    couchdb::ViewRow {
                        id: Some(hank_aaron_id),
                        key: Some("Hank Aaron".to_string()),
                        value: 755,
                    }],
               got.rows);
}

#[test]
fn get_view_with_endkey() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (_, babe_ruth_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Hank Aaron")
                             .insert("career_hr", 755)
                             .unwrap();
    let (_, _) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let design = couchdb::DesignBuilder::new()
                     .insert_view("by_career_hr",
                                  couchdb::ViewFunction {
                                      map: "function(doc) { emit(doc.name, doc.career_hr); }"
                                               .to_string(),
                                      reduce: None,
                                  })
                     .unwrap();
    client.put_document("/baseball/_design/stat", &design).run().unwrap();
    let got = client.get_view::<_, String, i32>("/baseball/_design/stat/_view/by_career_hr")
                    .endkey("Babe Ruth".to_string())
                    .run()
                    .unwrap();
    assert_eq!(Some(2), got.total_rows);
    assert_eq!(Some(0), got.offset);
    assert_eq!(vec![couchdb::ViewRow {
                        id: Some(babe_ruth_id),
                        key: Some("Babe Ruth".to_string()),
                        value: 714,
                    }],
               got.rows);
}

#[test]
fn get_view_with_startkey() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (_, _) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Hank Aaron")
                             .insert("career_hr", 755)
                             .unwrap();
    let (_, hank_aaron_id) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let design = couchdb::DesignBuilder::new()
                     .insert_view("by_career_hr",
                                  couchdb::ViewFunction {
                                      map: "function(doc) { emit(doc.name, doc.career_hr); }"
                                               .to_string(),
                                      reduce: None,
                                  })
                     .unwrap();
    client.put_document("/baseball/_design/stat", &design).run().unwrap();
    let got = client.get_view::<_, String, i32>("/baseball/_design/stat/_view/by_career_hr")
                    .startkey("Hank Aaron".to_string())
                    .run()
                    .unwrap();
    assert_eq!(Some(2), got.total_rows);
    assert_eq!(Some(1), got.offset);
    assert_eq!(vec![couchdb::ViewRow {
                        id: Some(hank_aaron_id),
                        key: Some("Hank Aaron".to_string()),
                        value: 755,
                    }],
               got.rows);
}

#[test]
fn get_view_reduced() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    client.post_to_database("/baseball", &source_content).run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Hank Aaron")
                             .insert("career_hr", 755)
                             .unwrap();
    client.post_to_database("/baseball", &source_content).run().unwrap();
    let design = couchdb::DesignBuilder::new()
                     .insert_view("by_career_hr",
                                  couchdb::ViewFunction {
                                      map: "function(doc) { emit(doc.name, doc.career_hr); }"
                                               .to_string(),
                                      reduce: Some("_sum".to_string()),
                                  })
                     .unwrap();
    client.put_document("/baseball/_design/stat", &design).run().unwrap();
    let got = client.get_view::<_, String, i32>("/baseball/_design/stat/_view/by_career_hr")
                    .reduce(true)
                    .run()
                    .unwrap();
    assert_eq!(None, got.total_rows);
    assert_eq!(None, got.offset);
    assert_eq!(vec![couchdb::ViewRow {
                        id: None,
                        key: None,
                        value: 714 + 755,
                    }],
               got.rows);
}
