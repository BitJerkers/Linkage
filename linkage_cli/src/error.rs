use thiserror::Error;
use std::io;
use std::net::AddrParseError;
use linkage_firewall::FirewallError;
use linkage_firewall::FirewallExceptionProtocolError;
use std::num::ParseIntError;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("io error occurred: {0}")]
    Io(#[from] io::Error),

    #[error("cannot parse openvpn configuration file")]
    OvpnFile,

    #[error("can't parse address: {0}")]
    AddrParse(#[from] AddrParseError),

    #[error("firewall error occurred: {0}")]
    Firewall(#[from] FirewallError),

    #[error("can't parse protocol: {0}")]
    FirewallExceptionProtocol(#[from] FirewallExceptionProtocolError),

    #[error("can't parse int: {0}")]
    ParseInt(#[from] ParseIntError),

    #[error("the program has to be run as administrator")]
    RootRequired,
}

pub(crate) type CliResult<T> = Result<T, CliError>;

impl CliError {
    /// Returns an exit code for the specific errors.
    pub fn get_exit_code(&self) -> i32 {
        match self {
            // Status code 2 is reserved
            CliError::RootRequired => 3,
            _ => 1,
        }
    }
}