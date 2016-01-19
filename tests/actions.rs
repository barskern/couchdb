extern crate couchdb;
extern crate serde_json;

use std::collections::HashSet;

macro_rules! expect_couchdb_error {
    ($result:expr, $expected_error_pattern:pat) => {
        match $result {
            Ok(..) => {
                panic!("Got unexpected OK result");
            }
            Err(ref e) => {
                match *e {
                    $expected_error_pattern => (),
                    _ => {
                        panic!("Got unexpected error: {}", e);
                    }
                }
            }
        }
    }
}

fn make_server_and_client() -> (couchdb::testing::FakeServer, couchdb::Client) {
    let server = couchdb::testing::FakeServer::new().unwrap();
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
    expect_couchdb_error!(got, couchdb::Error::NotFound(None));
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
    expect_couchdb_error!(got, couchdb::Error::NotFound(Some(..)));
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
    expect_couchdb_error!(got, couchdb::Error::DatabaseExists(Some(..)));
}

#[test]
fn delete_database_ok() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    client.delete_database("/foo").run().unwrap();
    let got = client.head_database("/foo").run();
    expect_couchdb_error!(got, couchdb::Error::NotFound(None));
}

#[test]
fn delete_database_nok_database_does_not_exist() {
    let (_server, client) = make_server_and_client();
    let got = client.delete_database("/foo").run();
    expect_couchdb_error!(got, couchdb::Error::NotFound(Some(..)));
}

#[test]
fn post_to_database_ok() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let expected_content = serde_json::builder::ObjectBuilder::new()
                               .insert("name", "Babe Ruth")
                               .insert("career_hr", 714)
                               .unwrap();
    let (doc_id, rev) = client.post_to_database("/baseball", &expected_content).run().unwrap();
    let doc = client.get_document(("/baseball", doc_id.clone()))
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc.id, doc_id);
    assert_eq!(doc.rev, rev);
    assert_eq!(expected_content, doc.into_content().unwrap());
}

#[test]
fn post_to_database_nok_database_does_not_exist() {
    let (_server, client) = make_server_and_client();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let got = client.post_to_database("/baseball", &source_content).run();
    expect_couchdb_error!(got, couchdb::Error::NotFound(Some(..)));
}

#[test]
fn get_changes_ok_no_changes() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let expected = couchdb::ChangesBuilder::new(0).unwrap();
    let got = client.get_changes("/baseball").run().unwrap();
    assert_eq!(expected, got);
}

#[test]
fn get_changes_ok_with_changes() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (babe_ruth_id, babe_ruth_rev) = client.post_to_database("/baseball", &source_content)
                                              .run()
                                              .unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Hank Aaron")
                             .insert("career_hr", 755)
                             .unwrap();
    let (hank_aaron_id, hank_aaron_rev) = client.post_to_database("/baseball", &source_content)
                                                .run()
                                                .unwrap();
    let expected = couchdb::ChangesBuilder::new(2)
                       .build_result(1, babe_ruth_id, |x| x.build_change(babe_ruth_rev, |x| x))
                       .build_result(2, hank_aaron_id, |x| x.build_change(hank_aaron_rev, |x| x))
                       .unwrap();
    let got = client.get_changes("/baseball").run().unwrap();
    assert_eq!(expected, got);
}

#[test]
fn head_document_ok_without_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (doc_id, _rev) = client.post_to_database("/baseball", &source_content).run().unwrap();
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
    let (doc_id, rev) = client.post_to_database("/baseball", &source_content).run().unwrap();
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
    let (doc_id, rev1) = client.post_to_database("/baseball", &source_content).run().unwrap();
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
    expect_couchdb_error!(got, couchdb::Error::NotFound(None));
}

#[test]
fn get_document_ok_without_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let expected_content = serde_json::builder::ObjectBuilder::new()
                               .insert("name", "Babe Ruth")
                               .insert("career_hr", 714)
                               .unwrap();
    let (doc_id, rev) = client.post_to_database("/baseball", &expected_content).run().unwrap();
    let got = client.get_document(("/baseball", doc_id.clone()))
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc_id, got.id);
    assert_eq!(rev, got.rev);
    assert_eq!(expected_content, got.into_content().unwrap());
}

#[test]
fn get_document_ok_if_none_match_fresh() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (doc_id, rev) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let got = client.get_document(("/baseball", doc_id.clone()))
                    .if_none_match(&rev)
                    .run()
                    .unwrap();
    assert!(got.is_none());
}

#[test]
fn get_document_ok_if_none_match_stale() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let expected_content = serde_json::builder::ObjectBuilder::new()
                               .insert("name", "Babe Ruth")
                               .insert("career_hr", 714)
                               .unwrap();
    let (doc_id, rev1) = client.post_to_database("/baseball", &expected_content).run().unwrap();
    let rev2 = client.put_document(("/baseball", doc_id.clone()), &expected_content)
                     .if_match(&rev1)
                     .run()
                     .unwrap();
    let got = client.get_document(("/baseball", doc_id.clone()))
                    .if_none_match(&rev1)
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc_id, got.id);
    assert_eq!(rev2, got.rev);
    assert_eq!(expected_content, got.into_content().unwrap());
}

#[test]
fn get_document_ok_by_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let expected_content = serde_json::builder::ObjectBuilder::new()
                               .insert("name", "Babe Ruth")
                               .insert("career_hr", 714)
                               .unwrap();
    let (doc_id, rev1) = client.post_to_database("/baseball", &expected_content).run().unwrap();
    let fresh_content = serde_json::builder::ObjectBuilder::new()
                            .insert("name", "Babe Ruth")
                            .insert("career_hr", 714)
                            .insert("career_hits", 2873)
                            .unwrap();
    client.put_document(("/baseball", doc_id.clone()), &fresh_content)
          .if_match(&rev1)
          .run()
          .unwrap();
    let got = client.get_document(("/baseball", doc_id.clone()))
                    .rev(&rev1)
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc_id, got.id);
    assert_eq!(rev1, got.rev);
    assert_eq!(expected_content, got.into_content().unwrap());
}

#[test]
fn get_document_ok_deleted() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (doc_id, rev1) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let rev2 = client.delete_document(("/baseball", doc_id.clone()), &rev1).run().unwrap();
    let expected_content = serde_json::builder::ObjectBuilder::new().unwrap();
    let got = client.get_document(("/baseball", doc_id.clone()))
                    .rev(&rev2)
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc_id, got.id);
    assert_eq!(rev2, got.rev);
    assert!(got.deleted);
    assert_eq!(expected_content, got.into_content().unwrap());
}

#[test]
fn get_document_nok_document_does_not_exist() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    let got = client.get_document("/foo/bar").run();
    expect_couchdb_error!(got, couchdb::Error::NotFound(Some(..)));
}

#[test]
fn put_document_ok_new_document() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let expected_content = serde_json::builder::ObjectBuilder::new()
                               .insert("name", "Babe Ruth")
                               .insert("career_hr", 714)
                               .unwrap();
    let rev = client.put_document("/baseball/babe_ruth", &expected_content).run().unwrap();
    let got = client.get_document("/baseball/babe_ruth")
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(couchdb::DocumentId::from("babe_ruth"), got.id);
    assert_eq!(rev, got.rev);
    assert_eq!(expected_content, got.into_content().unwrap());
}

#[test]
fn put_document_ok_update_document() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (doc_id, rev1) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let expected_content = serde_json::builder::ObjectBuilder::new()
                               .insert("name", "Babe Ruth")
                               .insert("career_hr", 714)
                               .insert("career_hits", 2873)
                               .unwrap();
    let rev2 = client.put_document(("/baseball", doc_id.clone()), &expected_content)
                     .if_match(&rev1)
                     .run()
                     .unwrap();
    let got = client.get_document(("/baseball", doc_id.clone()))
                    .run()
                    .unwrap()
                    .unwrap();
    assert_eq!(doc_id, got.id);
    assert_eq!(rev2, got.rev);
    assert_eq!(expected_content, got.into_content().unwrap());
}

#[test]
fn put_document_nok_stale_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (doc_id, rev1) = client.post_to_database("/baseball", &source_content).run().unwrap();
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
    expect_couchdb_error!(got, couchdb::Error::DocumentConflict(Some(..)));
}

#[test]
fn put_document_nok_database_does_not_exist() {
    let (_server, client) = make_server_and_client();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let got = client.put_document("/baseball/babe_ruth", &source_content).run();
    expect_couchdb_error!(got, couchdb::Error::NotFound(Some(..)));
}

#[test]
fn delete_document_ok() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (doc_id, rev1) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let rev2 = client.delete_document(("/baseball", doc_id.clone()), &rev1).run().unwrap();
    assert_eq!(rev1.update_number() + 1, rev2.update_number());
    let got = client.head_document(("/baseball", doc_id)).run();
    expect_couchdb_error!(got, couchdb::Error::NotFound(None));
}

#[test]
fn delete_document_nok_stale_revision() {
    let (_server, client) = make_server_and_client();
    client.put_database("/baseball").run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Babe Ruth")
                             .insert("career_hr", 714)
                             .unwrap();
    let (doc_id, rev1) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let _rev2 = client.put_document(("/baseball", doc_id.clone()), &source_content)
                      .if_match(&rev1)
                      .run()
                      .unwrap();
    let got = client.delete_document(("/baseball", doc_id.clone()), &rev1).run();
    expect_couchdb_error!(got, couchdb::Error::DocumentConflict(Some(..)));
}

#[test]
fn delete_document_nok_document_does_not_exist() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    let rev = couchdb::Revision::parse("1-12345678123456781234567812345678").unwrap();
    let got = client.delete_document("/foo/bar", &rev).run();
    expect_couchdb_error!(got, couchdb::Error::NotFound(Some(..)));
}

#[test]
fn get_view_empty_result() {
    let (_server, client) = make_server_and_client();
    client.put_database("/foo").run().unwrap();
    let design = couchdb::DesignBuilder::new()
                     .build_view("qux", "function(doc) {}", |x| x)
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
    let (babe_ruth_id, _) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Hank Aaron")
                             .insert("career_hr", 755)
                             .unwrap();
    let (hank_aaron_id, _) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let design = couchdb::DesignBuilder::new()
                     .build_view("by_career_hr",
                                 "function(doc) { emit(doc.name, doc.career_hr); }",
                                 |x| x)
                     .unwrap();
    client.put_document("/baseball/_design/stat", &design).run().unwrap();
    let got = client.get_view::<_, String, i32>("/baseball/_design/stat/_view/by_career_hr")
                    .run()
                    .unwrap();
    assert_eq!(Some(2), got.total_rows);
    assert_eq!(Some(0), got.offset);
    assert_eq!(vec![{
                        let mut v = couchdb::ViewRow::new(714);
                        v.id = Some(babe_ruth_id);
                        v.key = Some("Babe Ruth".to_string());
                        v
                    },
                    {
                        let mut v = couchdb::ViewRow::new(755);
                        v.id = Some(hank_aaron_id);
                        v.key = Some("Hank Aaron".to_string());
                        v
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
    let (babe_ruth_id, _) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let source_content = serde_json::builder::ObjectBuilder::new()
                             .insert("name", "Hank Aaron")
                             .insert("career_hr", 755)
                             .unwrap();
    let (_, _) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let design = couchdb::DesignBuilder::new()
                     .build_view("by_career_hr",
                                 "function(doc) { emit(doc.name, doc.career_hr); }",
                                 |x| x)
                     .unwrap();
    client.put_document("/baseball/_design/stat", &design).run().unwrap();
    let got = client.get_view::<_, String, i32>("/baseball/_design/stat/_view/by_career_hr")
                    .endkey("Babe Ruth".to_string())
                    .run()
                    .unwrap();
    assert_eq!(Some(2), got.total_rows);
    assert_eq!(Some(0), got.offset);
    assert_eq!(vec![{
                        let mut v = couchdb::ViewRow::new(714);
                        v.id = Some(babe_ruth_id);
                        v.key = Some("Babe Ruth".to_string());
                        v
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
    let (hank_aaron_id, _) = client.post_to_database("/baseball", &source_content).run().unwrap();
    let design = couchdb::DesignBuilder::new()
                     .build_view("by_career_hr",
                                 "function(doc) { emit(doc.name, doc.career_hr); }",
                                 |x| x)
                     .unwrap();
    client.put_document("/baseball/_design/stat", &design).run().unwrap();
    let got = client.get_view::<_, String, i32>("/baseball/_design/stat/_view/by_career_hr")
                    .startkey("Hank Aaron".to_string())
                    .run()
                    .unwrap();
    assert_eq!(Some(2), got.total_rows);
    assert_eq!(Some(1), got.offset);
    assert_eq!(vec![{
                        let mut v = couchdb::ViewRow::new(755);
                        v.id = Some(hank_aaron_id);
                        v.key = Some("Hank Aaron".to_string());
                        v
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
                     .build_view("by_career_hr",
                                 "function(doc) { emit(doc.name, doc.career_hr); }",
                                 |x| x.set_reduce("_sum"))
                     .unwrap();
    client.put_document("/baseball/_design/stat", &design).run().unwrap();
    let got = client.get_view::<_, String, i32>("/baseball/_design/stat/_view/by_career_hr")
                    .reduce(true)
                    .run()
                    .unwrap();
    assert_eq!(None, got.total_rows);
    assert_eq!(None, got.offset);
    assert_eq!(vec![couchdb::ViewRow::new(714 + 755)], got.rows);
}
