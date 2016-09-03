extern crate env_logger;
extern crate iron;
extern crate regex;
extern crate resolve;
extern crate url;

use iron::headers;
use iron::middleware::Handler;
use iron::prelude::*;
use iron::status;
use url::Host::Domain;

mod redirector;
use redirector::Redirector;
use redirector::RedirectorError;

macro_rules! bad_request(
    ($text:tt) => {{
        return Ok(Response::with((status::BadRequest, format!("400 Bad Request: {}\n", $text))))
    }}
);

struct RedirectorHandler {
    redirector: Redirector,
}

impl RedirectorHandler {
    fn new() -> RedirectorHandler {
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
            Some(header) => url::Host::parse(&header.hostname).ok().expect("invalid hostname"),
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

fn main() {
    let redirector = RedirectorHandler::new();
    Iron::new(redirector).http("127.0.0.1:1337").unwrap();
    println!("Listening on http://127.0.0.1:1337");
}
