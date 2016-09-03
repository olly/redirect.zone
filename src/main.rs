extern crate iron;
extern crate regex;
extern crate resolve;
extern crate url;

mod handler;
mod redirector;

use handler::RedirectorHandler;
use iron::Iron;

fn main() {
    let redirector = RedirectorHandler::new();
    Iron::new(redirector).http("127.0.0.1:1337").unwrap();
    println!("Listening on http://127.0.0.1:1337");
}
