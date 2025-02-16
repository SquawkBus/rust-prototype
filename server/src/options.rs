// Command line parameters.

use std::path::PathBuf;
use std::str::FromStr;
use std::{collections::HashSet, io};

use regex::Regex;

use crate::authorization::{AuthorizationSpec, Role};

const DEFAULT_SOCKET_ENDPOINT: &str = "0.0.0.0:8558";
const DEFAULT_WEB_SOCKET_ENDPOINT: &str = "0.0.0.0:8559";

/// Parses the string <user-pattern>:<topic-pattern>:<entitlements>:<roles>
impl FromStr for AuthorizationSpec {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let args: Vec<&str> = s.split(':').collect();
        if args.len() != 4 {
            return Err(format!("expected 4 parts, found {}", args.len()));
        }

        let topic_pattern = args[0];
        let user_pattern = args[1];
        let entitlements = args[2];
        let roles = args[3];

        let topic_pattern =
            Regex::new(topic_pattern).map_err(|e| format!("invalid regex: {}", e))?;
        let user_pattern = Regex::new(user_pattern).map_err(|e| format!("invalid regex: {}", e))?;
        let entitlements = entitlements
            .split(',')
            .map(|x| x.parse().map_err(|e| format!("invalid entitlement {}", e)))
            .collect::<std::result::Result<HashSet<i32>, String>>()?;
        let roles: Role =
            bitflags::parser::from_str(roles).map_err(|e| format!("invalid roles: {}", e))?;
        Ok(AuthorizationSpec {
            user_pattern,
            topic_pattern,
            entitlements,
            roles,
        })
    }
}

pub struct TLSOption {
    pub keyfile: PathBuf,
    pub certfile: PathBuf,
}

pub enum AuthenticationOption {
    None,
    Basic(PathBuf),
    Ldap(String),
}

pub struct Options {
    pub socket_endpoint: String,
    pub web_socket_endpoint: String,
    pub authorizations: Vec<AuthorizationSpec>,
    pub authorizations_file: Option<PathBuf>,
    pub tls: Option<TLSOption>,
    pub authentication: AuthenticationOption,
}

fn fetch_arg(arg_name: &str, args: &[String], arg_index: &mut usize) -> io::Result<String> {
    *arg_index = *arg_index + 1;
    if *arg_index >= args.len() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("insufficient arguments for {}", arg_name),
        ));
    }
    let arg = args.get(*arg_index).unwrap();

    Ok(arg.clone())
}

fn check_fetch_arg<T>(
    arg_name: &str,
    current_value: &Option<T>,
    args: &[String],
    arg_index: &mut usize,
) -> io::Result<String> {
    if current_value.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("argument {} requires a parameter", arg_name),
        ));
    }

    fetch_arg(arg_name, args, arg_index)
}

fn check_fetch_two_args<T>(
    arg_name: &str,
    current_value: &Option<T>,
    args: &[String],
    arg_index: &mut usize,
) -> io::Result<(String, String)> {
    if current_value.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("argument {} requires a parameter", arg_name),
        ));
    }

    let arg1 = fetch_arg(arg_name, args, arg_index)?;
    let arg2 = fetch_arg(arg_name, args, arg_index)?;

    Ok((arg1, arg2))
}

impl Options {
    pub fn parse(args: &[String]) -> io::Result<Self> {
        let mut socket_endpoint: Option<String> = None;
        let mut websocket_endpoint: Option<String> = None;
        let mut authorizations: Vec<AuthorizationSpec> = Vec::new();
        let mut authorizations_file: Option<PathBuf> = None;
        let mut tls: Option<TLSOption> = None;
        let mut authentication: Option<AuthenticationOption> = None;

        let mut arg_index = 1;
        while arg_index < args.len() {
            let arg_name = args.get(arg_index).unwrap().as_str();
            match arg_name {
                "--socket-endpoint" => {
                    let endpoint =
                        check_fetch_arg(arg_name, &socket_endpoint, &args, &mut arg_index)?;
                    socket_endpoint = Some(endpoint);
                }
                "--web-socket-endpoint" => {
                    let endpoint =
                        check_fetch_arg(arg_name, &websocket_endpoint, &args, &mut arg_index)?;
                    websocket_endpoint = Some(endpoint);
                }
                "--authorization" => {
                    let authorization = fetch_arg(arg_name, &args, &mut arg_index)?;
                    let authorization = authorization
                        .parse()
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    authorizations.push(authorization);
                }
                "--authorizations-file" => {
                    let filename =
                        check_fetch_arg(arg_name, &authorizations_file, &args, &mut arg_index)?;
                    authorizations_file = Some(filename.into());
                }
                "--tls" => {
                    let (certfile, keyfile) =
                        check_fetch_two_args(arg_name, &tls, &args, &mut arg_index)?;
                    tls = Some(TLSOption {
                        certfile: certfile.into(),
                        keyfile: keyfile.into(),
                    });
                }
                "--authentication" => {
                    let method = check_fetch_arg(arg_name, &authentication, &args, &mut arg_index)?;
                    authentication = Some(match method.as_str() {
                        "none" => AuthenticationOption::None,
                        "basic" => {
                            let filename =
                                check_fetch_arg(arg_name, &authentication, &args, &mut arg_index)?;
                            AuthenticationOption::Basic(filename.into())
                        }
                        "ldap" => {
                            let url =
                                check_fetch_arg(arg_name, &authentication, &args, &mut arg_index)?;
                            AuthenticationOption::Ldap(url)
                        }
                        _ => Err(io::Error::new(
                            io::ErrorKind::Other,
                            "invalid authentication option",
                        ))?,
                    });
                }
                "--help" => Err(io::Error::new(
                    io::ErrorKind::Other,
                    Self::usage(args.get(0).unwrap()),
                ))?,
                _ => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("invalid argument {}", arg_name),
                ))?,
            }

            arg_index += 1
        }

        // Default socket endpoint
        let socket_endpoint = socket_endpoint
            .or(Some(DEFAULT_SOCKET_ENDPOINT.into()))
            .unwrap();
        // Default websocket endpoint
        let websocket_endpoint = websocket_endpoint
            .or(Some(DEFAULT_WEB_SOCKET_ENDPOINT.into()))
            .unwrap();
        // Default authentication to none
        let authentication = authentication.or(Some(AuthenticationOption::None)).unwrap();

        return Ok(Self {
            socket_endpoint,
            web_socket_endpoint: websocket_endpoint,
            authorizations,
            authorizations_file,
            tls,
            authentication,
        });
    }

    pub fn usage(prog_name: &str) -> String {
        format!(
            "usage:
            \t{prog_name} [<options>]
            
            options:
            \t--socket-endpoint <ip-address>:<port> # defaults to {DEFAULT_SOCKET_ENDPOINT}
            \t--web-socket-endpoint <ip-address>:<port> # defaults to {DEFAULT_WEB_SOCKET_ENDPOINT}
            \t--tls <certfile> <keyfile>
            \t--authorization none # the default
            \t--authorization basic <passwd-file>
            \t--authorization ldap <url>
            \t--authorizations-file <filename>
            \t--authorization <user:topic:entitlements:roles>
            "
        )
    }

    pub fn load() -> io::Result<Self> {
        let args: Vec<String> = std::env::args().collect();
        match Self::parse(&args) {
            Ok(args) => Ok(args),
            Err(error) => {
                let prog_name = args.get(0).unwrap();
                let s = Self::usage(&prog_name);
                println!("error: {error}\n{s}");
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::authorization::AuthorizationManager;

    use super::*;

    #[test]
    fn parse() {
        let spec: AuthorizationSpec =
            AuthorizationSpec::from_str(".*:PUB\\.*:1,2:Subscriber|Publisher").unwrap();
        let user_entitlements_spec = vec![spec];
        let entitlements_manager = AuthorizationManager::new(user_entitlements_spec);

        let actual = entitlements_manager.entitlements("nobody", "PUB.foo", Role::Subscriber);
        let expected: HashSet<i32> = HashSet::from([1, 2]);
        assert_eq!(actual, expected);
    }
}
