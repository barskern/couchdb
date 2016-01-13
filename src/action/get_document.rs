use hyper;

use Document;
use Error;
use ErrorResponse;
use IntoDocumentPath;
use Revision;
use client::ClientState;
use action::{self, Action, Request, Response};

/// Action to get document meta-information and application-defined content.
///
/// # Return
///
/// This action returns an `Option` type. The return value is `None` if the
/// action specifies a revision and the document hasn't been modified since
/// that revision. Otherwise, the return value is `Some` and contains the
/// document meta-information and application-defined content.
///
/// # Errors
///
/// The following are some of the errors that may occur as a result of executing
/// this action:
///
///
/// * `Error::NotFound`: The document does not exist.
/// * `Error::Unauthorized`: The client is unauthorized.
///
pub struct GetDocument<'a, P>
    where P: IntoDocumentPath
{
    client_state: &'a ClientState,
    path: P,
    if_none_match: Option<&'a Revision>,
}

impl<'a, P: IntoDocumentPath> GetDocument<'a, P> {
    #[doc(hidden)]
    pub fn new(client_state: &'a ClientState, path: P) -> Self {
        GetDocument {
            client_state: client_state,
            path: path,
            if_none_match: None,
        }
    }

    /// Sets the If-None-Match header.
    pub fn if_none_match(mut self, rev: &'a Revision) -> Self {
        self.if_none_match = Some(rev);
        self
    }

    impl_action_public_methods!(Option<Document>);
}

impl<'a, P: IntoDocumentPath> Action for GetDocument<'a, P> {
    type Output = Option<Document>;

    fn make_request(self) -> Result<Request, Error> {
        let doc_path = try!(self.path.into_document_path());
        let uri = doc_path.into_uri(self.client_state.uri.clone());
        let request = Request::new(hyper::Get, uri)
                          .set_accept_application_json()
                          .set_if_none_match_revision(self.if_none_match);
        Ok(request)
    }

    fn take_response<R: Response>(mut response: R) -> Result<Self::Output, Error> {
        match response.status() {
            hyper::status::StatusCode::Ok => {
                try!(response.content_type_must_be_application_json());
                let doc = try!(response.decode_json::<Document>());
                Ok(Some(doc))
            }
            hyper::status::StatusCode::NotModified => Ok(None),
            hyper::status::StatusCode::BadRequest => Err(make_couchdb_error!(BadRequest, response)),
            hyper::status::StatusCode::Unauthorized => {
                Err(make_couchdb_error!(Unauthorized, response))
            }
            hyper::status::StatusCode::NotFound => Err(make_couchdb_error!(NotFound, response)),
            _ => Err(Error::UnexpectedHttpStatus { got: response.status() }),
        }
    }
}

#[cfg(test)]
mod tests {

    use hyper;
    use serde_json;

    use DocumentPath;
    use Revision;
    use client::ClientState;
    use action::{Action, JsonResponse, NoContentResponse};
    use super::GetDocument;

    #[test]
    fn make_request_default() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let action = GetDocument::new(&client_state, "/foo/bar");
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/foo/bar");
        expect_request_accept_application_json!(request);
    }

    #[test]
    fn make_request_if_none_match() {
        let client_state = ClientState::new("http://example.com:1234/").unwrap();
        let rev = Revision::parse("42-1234567890abcdef1234567890abcdef").unwrap();
        let action = GetDocument::new(&client_state, "/foo/bar").if_none_match(&rev);
        let request = action.make_request().unwrap();
        expect_request_method!(request, hyper::Get);
        expect_request_uri!(request, "http://example.com:1234/foo/bar");
        expect_request_accept_application_json!(request);
        expect_request_if_none_match_revision!(request, "42-1234567890abcdef1234567890abcdef");
    }

    #[test]
    fn take_response_ok() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("_id", "foo")
                         .insert("_rev", "42-1234567890abcdef1234567890abcdef")
                         .insert("bar", 17)
                         .unwrap();
        let response = JsonResponse::new(hyper::Ok, &source);
        let got = GetDocument::<DocumentPath>::take_response(response).unwrap();
        let got = got.unwrap();
        assert_eq!(got.id, "foo".into());
        assert_eq!(got.rev,
                   "42-1234567890abcdef1234567890abcdef".parse().unwrap());
        let expected = serde_json::builder::ObjectBuilder::new()
                           .insert("bar", 17)
                           .unwrap();
        let got = got.into_content::<serde_json::Value>().unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn take_response_not_modified() {
        let response = NoContentResponse::new(hyper::status::StatusCode::NotModified);
        let got = GetDocument::<DocumentPath>::take_response(response).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn take_response_bad_request() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "bad_request")
                         .insert("reason", "Invalid rev format")
                         .unwrap();
        let response = JsonResponse::new(hyper::BadRequest, &source);
        let got = GetDocument::<DocumentPath>::take_response(response);
        expect_couchdb_error!(got, BadRequest);
    }

    #[test]
    fn take_response_not_found() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "not_found")
                         .insert("reason", "missing")
                         .unwrap();
        let response = JsonResponse::new(hyper::NotFound, &source);
        let got = GetDocument::<DocumentPath>::take_response(response);
        expect_couchdb_error!(got, NotFound);
    }

    #[test]
    fn take_response_unauthorized() {
        let source = serde_json::builder::ObjectBuilder::new()
                         .insert("error", "unauthorized")
                         .insert("reason", "blah blah blah")
                         .unwrap();
        let response = JsonResponse::new(hyper::status::StatusCode::Unauthorized, &source);
        let got = GetDocument::<DocumentPath>::take_response(response);
        expect_couchdb_error!(got, Unauthorized);
    }
}
