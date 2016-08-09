extern crate hyper;
extern crate env_logger;
extern crate regex;
extern crate url;

use hyper::header::Host;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

pub mod redirector;
use redirector::RedirectConfiguration;

macro_rules! bad_request(
    ($response:ident, $text:tt) => {{
        *$response.status_mut() = hyper::BadRequest;
        $response.send(format!("400 Bad Request: {}\n", $text).as_bytes()).unwrap();
        return;
    }}
);

fn handler(request: Request, mut response: Response) {
    match request.headers.get::<Host>() {
        None => {
            bad_request!(response, "No Hostname")
        },
        // TODO: what does ref do? it compile without. do I need it?
        Some(ref host) => println!("{}", host.hostname),
    }

    println!("path: {}", request.uri);

    // let redirector = Redirector::new();
    // redirector::lookup(host.hostname);

    let redirect_configuration = RedirectConfiguration::parse("v=1; target=http://google.com; replace_path=true");
    match redirect_configuration {
        Err(_) => {
            // TODO: bad request
            return
        },
        Ok(redirect_configuration) => {
            match request.uri {
                AbsolutePath(_) => {
                    *response.status_mut() = hyper::status::StatusCode::MovedPermanently;
                    response.headers_mut().set(hyper::header::Location(redirect_configuration.target.into_string()));
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
