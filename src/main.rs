extern crate hyper;
extern crate env_logger;
extern crate rand;
extern crate regex;
extern crate resolve;
extern crate url;

use hyper::header;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;
use rand::{thread_rng, Rng};
use url::Host;
use url::Host::Domain;

mod redirector;
use redirector::Redirector;

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
    let redirects = redirector.lookup(&domain).ok().unwrap(); // TODO

    let valid_redirects: Vec<_> = redirects.into_iter().filter_map(|redirect| redirect.ok()).collect();

    let redirect = match valid_redirects.len() {
        0 => bad_request!(response, "No Valid Redirect"), // TODO
        1 => valid_redirects.get(0).unwrap(), // TODO: unwrap
        _ => {
            let mut random = thread_rng();
            random.choose(&valid_redirects).unwrap()
        },
    };

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
}

fn main() {
    env_logger::init().unwrap();
    let server = Server::http("127.0.0.1:1337").unwrap();
    let _guard = server.handle(handler);
    println!("Listening on http://127.0.0.1:1337");
}
