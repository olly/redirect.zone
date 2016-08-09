extern crate hyper;
extern crate env_logger;
extern crate regex;
extern crate url;

use hyper::header::Host;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;
use regex::Regex;
use std::collections::HashMap;
use url::Url;

macro_rules! bad_request(
    ($response:ident, $text:tt) => {{
        *$response.status_mut() = hyper::BadRequest;
        $response.send(format!("400 Bad Request: {}\n", $text).as_bytes()).unwrap();
        return;
    }}
);

#[derive(Debug)]
#[derive(PartialEq)]
enum RedirectConfigurationParseError {
    MissingVersion,
    InvalidVersion,
    UnsupportedVersion(u8),
    MissingTarget,
    InvalidTarget(url::ParseError),
}

#[derive(Debug)]
#[derive(PartialEq)]
struct RedirectConfiguration {
    target: Url,
    replace_path: bool,
}

impl RedirectConfiguration {
    fn parse(config: &str) -> Result<RedirectConfiguration, RedirectConfigurationParseError> {
        let delimiter = Regex::new(r";(\s*)").unwrap();

        let fields = delimiter.split(config).fold(HashMap::new(), |mut memo, field| {
            let x: Vec<&str> = field.split("=").collect();
            memo.insert(x[0], x[1]);
            memo
        });

        match fields.get("v").ok_or(RedirectConfigurationParseError::MissingVersion).and_then(|v| u8::from_str_radix(v, 10).map_err(|_| RedirectConfigurationParseError::InvalidVersion)) {
            Ok(1) => {},
            Ok(v) => return Err(RedirectConfigurationParseError::UnsupportedVersion(v)),
            Err(e) => return Err(e),
        };

        let target = try!(fields.get("target").ok_or(RedirectConfigurationParseError::MissingTarget).and_then(|v| Url::parse(v).map_err(RedirectConfigurationParseError::InvalidTarget)));
        let replace_path = fields.get("replace_path").and_then(|v| v.parse::<bool>().ok()).unwrap_or(false);

        return Ok(RedirectConfiguration {
            target: target,
            replace_path: replace_path,
        });
    }
}

#[cfg(test)]
mod tests {
    extern crate url;

    use RedirectConfiguration;
    use RedirectConfigurationParseError::*;
    use url::ParseError;

    #[test]
    fn it_handles_missing_version() {
        assert_eq!(Err(MissingVersion), RedirectConfiguration::parse(""));
    }

    #[test]
    fn it_handles_invalid_version() {
        assert_eq!(Err(InvalidVersion), RedirectConfiguration::parse("v=junk;"));

        // TODO: include value
        // assert_eq!(Err(InvalidVersion("junk")), RedirectConfiguration::parse("v=junk;"));
    }

    #[test]
    fn it_handles_unsupported_versions() {
        assert_eq!(Err(UnsupportedVersion(0)), RedirectConfiguration::parse("v=0"));
        assert_eq!(Err(UnsupportedVersion(2)), RedirectConfiguration::parse("v=2"));
    }

    #[test]
    fn it_handles_missing_target() {
        assert_eq!(Err(MissingTarget), RedirectConfiguration::parse("v=1;"));
    }

    #[test]
    fn it_handles_invalid_target() {
        assert_eq!(Err(InvalidTarget(ParseError::RelativeUrlWithoutBase)), RedirectConfiguration::parse("v=1; target=junk"));
    }

    #[test]
    fn it_parses_configuration() {
        let configuration = RedirectConfiguration::parse("v=1; target=https://google.com").unwrap();
        assert_eq!(url::Url::parse("https://google.com").unwrap(), configuration.target);
        assert_eq!(false, configuration.replace_path);

        let configuration = RedirectConfiguration::parse("v=1; target=https://google.com; replace_path=true").unwrap();
        assert_eq!(true, configuration.replace_path);
    }
}

fn handler(request: Request, mut response: Response) {
    match request.headers.get::<Host>() {
        None => {
            bad_request!(response, "No Hostname")
        },
        // TODO: what does ref do? it compile without. do I need it?
        Some(ref host) => println!("{}", host.hostname),
    }

    println!("path: {}", request.uri);

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
