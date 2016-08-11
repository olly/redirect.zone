mod configuration;

extern crate hyper;
extern crate resolve;
extern crate url;

use regex::Regex;
use resolve::config::default_config;
use resolve::resolver::DnsResolver;
use resolve::record::Txt;
use std;
use std::collections::HashMap;
use std::str;
use url::Url;

#[derive(Debug)]
#[derive(PartialEq)]
struct RedirectConfiguration {
    pub target: Url,
    pub replace_path: bool,
}

#[derive(Debug)]
#[derive(PartialEq)]
enum RedirectConfigurationParseError {
    MissingVersion,
    InvalidVersion,
    UnsupportedVersion(u8),
    MissingTarget,
    InvalidTarget(url::ParseError),
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

pub struct Redirector {
    resolver: DnsResolver
}

pub enum RedirectorError {
    ResolverError,
}

type ResolverError = std::io::Error;

pub struct Redirect {

}

impl Redirect {
    pub fn target_from(&self, source: hyper::uri::RequestUri) -> String {
        return "http://google.com".to_string();
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

        return Redirector { resolver: resolver }
    }

    pub fn lookup(&self, hostname: &str) -> Result<Redirect, RedirectorError> {
        let record = format!("_redirect.{}", hostname);
        println!("lookup: {}", record);

        let records = self.resolve(record.as_str());
        match records {
            Err(_) => return Err(RedirectorError::ResolverError),
            Ok(records) => {
                for record in records {
                    let x = RedirectConfiguration::parse(record.as_str());
                    match x {
                        Ok(x) => println!("{}", x.target),
                        Err(e) => println!("{:?}", e),
                    }
                }
            }
        }

        return Ok(Redirect{});
    }

    fn resolve(&self, record: &str) -> Result<Vec<String>, ResolverError> {
        // TODO: can I hold onto enough here, to use &strs rather than String?
        // TODO: unwrap
        match self.resolver.resolve_record::<Txt>(record) {
            Ok(records) => {
                Ok(records.iter().map(|record| str::from_utf8(&record.data).unwrap().to_string()).collect())
            }
            Err(e) => return Err(e),
        }
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
