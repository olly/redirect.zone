extern crate hyper;
extern crate env_logger;
extern crate regex;
extern crate resolve;
extern crate url;

use hyper::header;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;
use url::Host;
use url::Host::Domain;

mod redirector;
use redirector::Redirector;
use redirector::RedirectorError;

macro_rules! bad_request(
    ($response:ident, $text:tt) => {{
        *$response.status_mut() = hyper::BadRequest;
        $response.send(format!("400 Bad Request: {}\n", $text).as_bytes()).unwrap();
        return;
    }}
);

fn handler(request: Request, mut response: Response) {
    let host = match request.headers.get::<header::Host>() {
        None => {
            bad_request!(response, "Unknown Host")
        },
        // TODO: what does ref do? it compile without. do I need it?
        // TODO: unwrap
        Some(header) => Host::parse(&header.hostname).ok().unwrap()
    };

    let domain: String = match host {
        Domain(domain) => domain,
        _ => bad_request!(response, "Invalid Host")
    };

    let redirector = Redirector::new();
    let redirect = redirector.find(&domain);
    match redirect {
        Ok(redirect) => {
            match request.uri {
                AbsolutePath(path) => {
                    let target = redirect.target_from(&path).into_string();

                    *response.status_mut() = hyper::status::StatusCode::MovedPermanently;
                    response.headers_mut().set(hyper::header::Location(target));
                    return;
                },
                _ => {
                    return;
                }
            };
        },
        Err(RedirectorError::ResolverError) => bad_request!(response, "Resolver Error"),
        Err(RedirectorError::NoValidRedirect) => bad_request!(response, "No Valid Redirect"),
    }
}

fn main() {
    env_logger::init().unwrap();
    let server = Server::http("127.0.0.1:1337").unwrap();
    let _guard = server.handle(handler);
    println!("Listening on http://127.0.0.1:1337");
}
