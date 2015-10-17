use hyper;

use document::Revision;
use error::{Error, TransportCause};

pub struct Request {
	request: hyper::client::Request<hyper::net::Fresh>,
	body: Vec<u8>,
}

impl Request {

    pub fn new(
        method: hyper::method::Method,
        uri: hyper::Url)
        -> Result<Self, Error>
    {
        let r = try!(
            hyper::client::Request::new(method, uri)
                .map_err(|e| {
                    Error::Transport { cause: TransportCause::Hyper(e) }
                })
        );

        Ok(Request {
            request: r,
            body: Vec::new(),
        })
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn accept_application_json(mut self) -> Self {
        let qitems = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            vec![
                hyper::header::qitem(
                    Mime(TopLevel::Application, SubLevel::Json, vec![])
                )
            ]
        };
        self.request.headers_mut().set(hyper::header::Accept(qitems));
        self
    }

    pub fn content_type_application_json(mut self) -> Self {
        let qitems = {
            use hyper::mime::{Mime, TopLevel, SubLevel};
            Mime(TopLevel::Application, SubLevel::Json, vec![])
        };
        self.request.headers_mut().set(hyper::header::ContentType(qitems));
        self
    }

    pub fn if_match_revision(mut self, rev: Option<&Revision>) -> Self {
        match rev {
            None => self,
            Some(rev) => {
                let etags = new_revision_etags(rev);
                self.request.headers_mut().set(hyper::header::IfMatch::Items(etags));
                self
            }
        }
    }

    pub fn if_none_match_revision(mut self, rev: Option<&Revision>) -> Self {
        match rev {
            None => self,
            Some(rev) => {
                let etags = new_revision_etags(rev);
                self.request.headers_mut()
                    .set(hyper::header::IfNoneMatch::Items(etags));
                self
            }
        }
    }
}

fn new_revision_etags(rev: &Revision) -> Vec<hyper::header::EntityTag> {
    vec![
        hyper::header::EntityTag::new(
            false,
            rev.to_string()
        )
    ]
}

/// CouchDB command trait
///
/// The Command trait abstracts the machinery for executing CouchDB commands.
/// Types implementing the Command trait define only how they construct requests
/// and process responses.
pub trait Command: Sized {
    type Output;
    fn make_request(self) -> Result<Request, Error>;
    fn take_response(resp: hyper::client::Response)
        -> Result<Self::Output, Error>;
}

pub fn run_command<C>(cmd: C) -> Result<C::Output, Error> where C: Command
{
    let resp = {
        use std::io::Write;
        let req = try!(cmd.make_request());
        let mut stream = try!(
            req.request.start().map_err(|e| {
                Error::Transport { cause: TransportCause::Hyper(e) }
            })
        );
        try!(
            stream.write_all(&req.body)
                .map_err(|e| {
                    Error::Transport {
                        cause: TransportCause::Io(e),
                    }
                })
        );
        try!(
            stream.send()
                .map_err(|e| {
                    Error::Transport {
                        cause: TransportCause::Hyper(e),
                    }
                })
        )
    };
    C::take_response(resp)
}