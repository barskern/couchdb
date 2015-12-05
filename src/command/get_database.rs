use hyper;

use client::{self, ClientState};
use database::Database;
use dbpath::DatabasePath;
use dbtype;
use error::{self, Error};
use transport::{self, Command, Request};

/// Command to get a database.
pub struct GetDatabase<'a> {
    client_state: &'a ClientState,
    path: DatabasePath,
}

impl<'a> GetDatabase<'a> {

    #[doc(hidden)]
    pub fn new_get_database(client_state: &'a ClientState, path: DatabasePath)
        -> Self
    {
        GetDatabase {
            client_state: client_state,
            path: path,
        }
    }

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
    type State = ();

    fn make_request(self) -> Result<(Request, Self::State), Error> {
        let uri = self.path.into_uri(self.client_state.uri.clone());
        let req = try!(Request::new(hyper::Get, uri))
            .accept_application_json();
        Ok((req, ()))
    }

    fn take_response(mut resp: hyper::client::Response, _state: Self::State)
        -> Result<Self::Output, Error>
    {
        match resp.status {
            hyper::status::StatusCode::Ok => {
                let s = try!(client::read_json_response(&mut resp));
                let db = try!(client::decode_json::<dbtype::Database>(&s));
                Database::from_db_database(db)
            },
            hyper::status::StatusCode::NotFound =>
                Err(error::new_because_not_found(&mut resp)),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}
