pub(crate) mod consts;
pub mod error;

use crate::error::{CliError, CliResult};
use clap::{App as ClapApp, Arg as ClapArg, ArgMatches as ClapArgMatches};
#[cfg(windows)]
use is_elevated::is_elevated;
#[cfg(unix)]
use libc;
use linkage_firewall::get_backends;
use linkage_firewall::FirewallException;
use ovpnfile::{self, ConfigDirective as OvpnConfigDirective};
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The entry point of the cli application.
pub fn entry() -> CliResult<()> {
    // Administrator privileges are required
    //root_check()?;

    let matches = get_config_matches();

    // This should not be None
    let config_file_path = matches.value_of("config").unwrap();
    println!("Using configuration file {}", config_file_path);
    let config_file = File::open(config_file_path)?;

    // Get the exceptions from the configuration file
    let exceptions = parse_configuration_file(config_file)?;

    // The first backend is currently iptables, will be made more modular in the next versions
    let firewall_backend = get_backends().first().unwrap();
    if !firewall_backend.is_available()? {
        return Err(error::CliError::FirewallBackendNotAvailable);
    }

    firewall_backend.on_pre_connect(&exceptions)?;

    let mut c = Command::new("openvpn")
        .arg(config_file_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = c.stdout.unwrap();
    let mut buffer = [0; 2048];

    let regex = Regex::new(r"net_iface_up: set (tun[0-9]+) up").unwrap();

    // TODO: Error handling
    let interface_name = loop {
        stdout.read(&mut buffer);
        let i = String::from_utf8_lossy(&buffer);
        let matches = regex.captures(&i);
        if let Some(matches) = matches {
            let m = matches.get(1).unwrap();
            break m.as_str().to_string();
        }
    };

    // After connect
    firewall_backend.on_post_connect(&interface_name);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst)).unwrap();

    println!("Waiting...");
    while running.load(Ordering::SeqCst) {}
    println!("Exiting...");
    //c.kill().unwrap();

    // When disconnecting
    firewall_backend.on_disconnect()?;

    Ok(())
}

/// Checks if the program is running as root.
fn root_check() -> CliResult<()> {
    if cfg!(windows) {
        #[cfg(windows)]
        // TODO: Ask for root permission. Windows support will be introduced in a later release though, so not high priority
        if !is_elevated() {
            panic!("The Program is not running as an administrator, please run it as admin");
        }
    }
    // We're assuming all other Platforms are Unix-based
    else {
        #[cfg(unix)]
        unsafe {
            let uid = libc::getuid();
            if uid != 0 {
                // TODO: Ask for root permission
                return Err(CliError::RootRequired);
            }
        }
    }

    Ok(())
}

/// Parses the supplied configuration file using ovpnfile.
fn parse_configuration_file(f: File) -> CliResult<Vec<FirewallException>> {
    // TODO: Make this more modular to support other VPN applications, not only OpenVPN.
    let parsed_file = ovpnfile::parse(f).map_err(|_| error::CliError::OvpnFile)?;

    // Get the default settings
    let mut default_protocol: Option<String> = None;
    for d in parsed_file.directives() {
        match d {
            OvpnConfigDirective::Proto { p } => {
                default_protocol = Some(p);
            }
            _ => (),
        }
    }

    // Create the firewall exceptions
    let mut exceptions: Vec<FirewallException> = Vec::new();
    for d in parsed_file.directives() {
        match d {
            OvpnConfigDirective::Remote { host, port, proto } => {
                // TODO: Handle the unwrap() calls here
                let default_protocol_clone = default_protocol.clone();
                exceptions.push(FirewallException::new(
                    host.parse()?,
                    port.unwrap().parse()?,
                    proto
                        .unwrap_or_else(|| default_protocol_clone.unwrap())
                        .parse()?,
                ));
            }
            _ => (),
        }
    }

    Ok(exceptions)
}

/// Returns the options that were supplied to the application.
fn get_config_matches<'a>() -> ClapArgMatches<'a> {
    ClapApp::new(consts::APP_NAME)
        .version(consts::APP_VERSION)
        .author(consts::APP_AUTHOR)
        .about(consts::APP_ABOUT)
        .arg(
            ClapArg::with_name("config")
                .required(true)
                .short("c")
                .long("config")
                .value_name("FILE"),
        )
        .get_matches()
}
