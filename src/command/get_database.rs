use hyper;
use serde_json;
use std;

use client::{self, ClientState};
use database::{self, Database};
use error::{self, Error};
use transport::{self, Command, Request};

#[doc(hidden)]
pub fn new_get_database<'a>(
    client_state: &'a ClientState,
    db_name: &'a str)
    -> GetDatabase<'a>
{
    GetDatabase {
        client_state: client_state,
        db_name: db_name,
    }
}

/// Command to get a database.
pub struct GetDatabase<'a> {
    client_state: &'a ClientState,
    db_name: &'a str,
}

impl<'a> GetDatabase<'a> {

    /// Send the command request and wait for the response.
    ///
    /// # Errors
    ///
    /// Note: Other errors may occur.
    ///
    /// * `Error::NotFound`: The database does not exist.
    ///
    pub fn run(self) -> Result<Database, Error> {
        transport::run_command(self)
    }
}

impl<'a> Command for GetDatabase<'a> {

    type Output = Database;

    fn make_request(self) -> Result<Request, Error> {
        let uri = database::new_uri(&self.client_state.uri, self.db_name);
        let req = try!(Request::new(hyper::Get, uri))
            .accept_application_json();
        Ok(req)
    }

    fn take_response(mut resp: hyper::client::Response)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                let s = try!(client::read_json_response(&mut resp));
                let mut resp_body = try!(client::decode_json::<serde_json::Value>(&s));
                (|| {
                    let dot = match resp_body.as_object_mut() {
                        None => { return None; },
                        Some(x) => x,
                    };
                    let doc_count = match dot.get("doc_count") {
                        None => { return None; },
                        Some(x) => match x.as_u64() {
                            None => { return None; },
                            Some(x) => x,
                        },
                    };
                    let doc_del_count = match dot.get("doc_del_count") {
                        None => { return None; },
                        Some(x) => match x.as_u64() {
                            None => { return None; },
                            Some(x) => x,
                        },
                    };
                    let db_name = match dot.get_mut("db_name") {
                        None => { return None; },
                        Some(x) => match *x {
                            serde_json::Value::String(ref mut x) =>
                                std::mem::replace(x, String::new()),
                            _ => { return None; },
                        },
                    };
                    Some(Database {
                        doc_count: doc_count,
                        doc_del_count: doc_del_count,
                        db_name: db_name,
                    })
                })()
                .ok_or(Error::UnexpectedContent { got: s } )
            },
            hyper::status::StatusCode::NotFound =>
                Err(error::new_because_not_found(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}
