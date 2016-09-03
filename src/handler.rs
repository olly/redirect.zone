use iron::headers;
use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;
use url::Host;
use url::Host::Domain;

use redirector::Redirector;
use redirector::RedirectorError;

macro_rules! bad_request(
    ($text:tt) => {{
        return Ok(Response::with((status::BadRequest, format!("400 Bad Request: {}\n", $text))))
    }}
);

pub struct RedirectorHandler {
    redirector: Redirector,
}

impl RedirectorHandler {
    pub fn new() -> RedirectorHandler {
        let redirector = Redirector::new();
        return RedirectorHandler{
            redirector: redirector,
        }
    }
}

impl Handler for RedirectorHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let host = match request.headers.get::<headers::Host>() {
            None => bad_request!("Unknown Host"),
            Some(header) => Host::parse(&header.hostname).ok().expect("invalid hostname"),
        };

        let domain: String = match host {
            Domain(domain) => domain,
            _ => bad_request!("Invalid Host")
        };

        let redirect = self.redirector.find(&domain);
        match redirect {
            Ok(redirect) => {
                let url = request.url.to_owned().into_generic_url();
                let path = url.path();
                let target = redirect.target_from(&path).into_string();

                let mut response = Response::with(status::MovedPermanently);
                response.headers.set(headers::Location(target));
                return Ok(response);
            },
            Err(RedirectorError::ResolverError) => bad_request!("Resolver Error"),
            Err(RedirectorError::NoValidRedirect) => bad_request!("No Valid Redirect"),
        }
    }
}
