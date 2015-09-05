use hyper;

use client;
use error::{self, Error};

/// Command to create a database.
pub struct HeadDatabase<'a, 'b> {
    client_state: &'a client::ClientState,
    db_name: &'b str,
}

impl<'a, 'b> HeadDatabase<'a, 'b> {

    pub fn new(client_state: &'a client::ClientState, db_name: &'b str) -> HeadDatabase<'a, 'b> {
        HeadDatabase {
            client_state: client_state,
            db_name: db_name,
        }
    }

    /// Send the command request and wait for the response.
    // TODO: Document error variants.
    pub fn run(self) -> Result<(), Error> {
        let resp = {
            let mut u = self.client_state.uri.clone();
            u.path_mut().unwrap()[0] = self.db_name.to_string();
            try!(
                self.client_state.http_client.head(u)
                .send()
                .or_else(|e| {
                    Err(Error::Transport { cause: error::TransportCause::Hyper(e) } )
                })
            )
        };
        match resp.status {
            hyper::status::StatusCode::Ok => Ok(()),
            hyper::status::StatusCode::NotFound =>
                Err(Error::NotFound { response: None } ),
            _ => Err(Error::UnexpectedHttpStatus { got: resp.status } ),
        }
    }
}


