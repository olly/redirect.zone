extern crate hyper;
extern crate env_logger;
extern crate regex;
extern crate resolve;
extern crate url;

use hyper::header::Host;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

pub mod redirector;
use redirector::Redirector;

macro_rules! bad_request(
    ($response:ident, $text:tt) => {{
        *$response.status_mut() = hyper::BadRequest;
        $response.send(format!("400 Bad Request: {}\n", $text).as_bytes()).unwrap();
        return;
    }}
);

fn handler(request: Request, mut response: Response) {
    let hostname = match request.headers.get::<Host>() {
        None => {
            bad_request!(response, "No Hostname")
        },
        // TODO: what does ref do? it compile without. do I need it?
        Some(ref host) => host.hostname.as_str()
    };

    println!("path: {}", request.uri);

    let redirector = Redirector::new();
    let redirect = redirector.lookup(hostname);

    match redirect {
        Err(_) => {
            // TODO: bad request
            return
        },
        Ok(redirect) => {
            match request.uri {
                AbsolutePath(_) => {
                    let target = redirect.target_from(request.uri);

                    *response.status_mut() = hyper::status::StatusCode::MovedPermanently;
                    response.headers_mut().set(hyper::header::Location(target));
                    return;
                },
                _ => {
                    return;
                }
            };
        },
    }
}

fn main() {
    env_logger::init().unwrap();
    let server = Server::http("127.0.0.1:1337").unwrap();
    let _guard = server.handle(handler);
    println!("Listening on http://127.0.0.1:1337");
}
