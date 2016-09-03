extern crate resolve;
extern crate url;

use regex::Regex;
use resolve::config::default_config;
use resolve::resolver::DnsResolver;
use resolve::record::Txt;
use std;
use std::collections::HashMap;
use std::str;
use std::sync::{Arc, Mutex};
use url::Url;

trait Resolver: Send + Sync {
    fn resolve(&self, record: &str) -> Result<Vec<String>, ResolverError>;
}

impl Resolver for Arc<Mutex<DnsResolver>> {
    fn resolve(&self, record: &str) -> Result<Vec<String>, ResolverError> {
        // TODO: unwrap
        let resolver = self.lock().unwrap();

        // TODO: can I hold onto enough here, to use &strs rather than String?
        // TODO: unwrap
        match resolver.resolve_record::<Txt>(record) {
            Ok(records) => {
                Ok(records.iter().map(|record| str::from_utf8(&record.data).unwrap().to_string()).collect())
            }
            Err(e) => return Err(e),
        }
    }
}

pub struct Redirector {
    // TODO: I think this could probably be a reference, but I couldn't figure
    // out the ownership.
    resolver: Box<Resolver>,
}

pub enum RedirectorError {
    ResolverError,
    NoValidRedirect,
}

type ResolverError = std::io::Error;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum RedirectParseError {
    MissingVersion,
    InvalidVersion,
    UnsupportedVersion(u8),
    MissingTarget,
    InvalidTarget(url::ParseError),
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Redirect {
    pub target: Url,
    pub replace_path: bool,
}

impl Redirect {
    fn parse(config: &str) -> Result<Redirect, RedirectParseError> {
        let delimiter = Regex::new(r";(\s*)").unwrap();

        let fields = delimiter.split(config).fold(HashMap::new(), |mut memo, field| {
            let x: Vec<&str> = field.split("=").collect();
            memo.insert(x[0], x[1]);
            memo
        });

        match fields.get("v").ok_or(RedirectParseError::MissingVersion).and_then(|v| u8::from_str_radix(v, 10).map_err(|_| RedirectParseError::InvalidVersion)) {
            Ok(1) => {},
            Ok(v) => return Err(RedirectParseError::UnsupportedVersion(v)),
            Err(e) => return Err(e),
        };

        let target = try!(fields.get("target").ok_or(RedirectParseError::MissingTarget).and_then(|v| Url::parse(v).map_err(RedirectParseError::InvalidTarget)));
        let replace_path = fields.get("replace_path").and_then(|v| v.parse::<bool>().ok()).unwrap_or(false);

        return Ok(Redirect {
            target: target,
            replace_path: replace_path,
        });
    }

    pub fn target_from(&self, source: &str) -> Url {
        if self.replace_path {
            let mut target = self.target.clone();
            target.set_path(source);
            return target;
        } else {
            return self.target.clone();
        }
    }
}

impl Redirector {
    pub fn new() -> Redirector {
        let config = match default_config() {
            Ok(config) => config,
            Err(e) => {
                // TODO: this fails with no network.
                panic!("Failed to load system configuration: {}", e);
            }
        };

        let resolver = match DnsResolver::new(config) {
            Ok(resolver) => resolver,
            Err(e) => {
                panic!("Failed to create DNS resolver: {}", e);
            }
        };

        return Redirector { resolver: Box::new(Arc::new(Mutex::new(resolver))) }
    }

    pub fn lookup(&self, hostname: &str) -> Result<Vec<Result<Redirect, RedirectParseError>>, RedirectorError> {
        let record = format!("_redirect.{}", hostname);
        println!("lookup: {}", record);

        let records = self.resolver.resolve(&record);
        match records {
            Err(_) => return Err(RedirectorError::ResolverError),
            Ok(records) => return Ok(records.iter().map(|record| Redirect::parse(record)).collect()),
        }
    }

    pub fn find(&self, hostname: &str) -> Result<Redirect, RedirectorError> {
        let redirects = try!(self.lookup(hostname));
        let mut valid_redirects: Vec<_> = redirects.into_iter().filter_map(|redirect| redirect.ok()).collect();

        match valid_redirects.len() {
            0 => return Err(RedirectorError::NoValidRedirect), // TODO
            _ => return Ok(valid_redirects.remove(0)), // TODO: unwrap
        };
    }
}

#[cfg(test)]
mod tests {
    extern crate url;

    use redirector::Redirect;
    use redirector::RedirectParseError::*;
    use url::ParseError;
    use url::Url;

    #[test]
    fn it_handles_missing_version() {
        assert_eq!(Err(MissingVersion), Redirect::parse(""));
    }

    #[test]
    fn it_handles_invalid_version() {
        assert_eq!(Err(InvalidVersion), Redirect::parse("v=junk;"));

        // TODO: include value
        // assert_eq!(Err(InvalidVersion("junk")), Redirect::parse("v=junk;"));
    }

    #[test]
    fn it_handles_unsupported_versions() {
        assert_eq!(Err(UnsupportedVersion(0)), Redirect::parse("v=0"));
        assert_eq!(Err(UnsupportedVersion(2)), Redirect::parse("v=2"));
    }

    #[test]
    fn it_handles_missing_target() {
        assert_eq!(Err(MissingTarget), Redirect::parse("v=1;"));
    }

    #[test]
    fn it_handles_invalid_target() {
        assert_eq!(Err(InvalidTarget(ParseError::RelativeUrlWithoutBase)), Redirect::parse("v=1; target=junk"));
    }

    #[test]
    fn it_parses_configuration() {
        let configuration = Redirect::parse("v=1; target=https://google.com").unwrap();
        assert_eq!(url::Url::parse("https://google.com").unwrap(), configuration.target);
        assert_eq!(false, configuration.replace_path);

        let configuration = Redirect::parse("v=1; target=https://google.com; replace_path=true").unwrap();
        assert_eq!(true, configuration.replace_path);
    }

    #[test]
    fn it_returns_target_if_replace_path_is_false() {
        let source = "/";
        let redirect = Redirect{target: Url::parse("https://example.com/test/").unwrap(), replace_path: false};

        let expected = Url::parse("https://example.com/test/").unwrap();
        assert_eq!(expected, redirect.target_from(source))
    }

    #[test]
    fn it_returns_target_with_replaced_path_if_replace_path_is_true() {
        let source = "/source-path";
        let redirect = Redirect{target: Url::parse("https://example.com/test/").unwrap(), replace_path: true};

        let expected = Url::parse("https://example.com/source-path").unwrap();
        assert_eq!(expected, redirect.target_from(source))
    }
}
