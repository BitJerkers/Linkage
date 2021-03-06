use super::Executor;
use std::process::Command;
use crate::error::{FirewallError, FirewallResult};

/// The base command to use for iptables actions.
#[derive(Debug, PartialEq)]
pub enum IptablesBaseCommand {
    Iptables,
    Ip6tables,
}

/// The binary name for `iptables`.
pub(crate) const IPTABLES_BINARY_NAME: &str = "iptables";
/// The binary name for `ip6tables`.
pub(crate) const IP6TABLES_BINARY_NAME: &str = "ip6tables";

impl IptablesBaseCommand {
    /// Returns the command according to the selected base command.
    pub fn get_command(&self) -> String {
        match self {
            IptablesBaseCommand::Iptables => IPTABLES_BINARY_NAME,
            IptablesBaseCommand::Ip6tables => IP6TABLES_BINARY_NAME,
        }.into()
    }
}

/// Responsible for executing `iptables` using std::process::Command.
#[derive(Debug, PartialEq)]
pub struct IptablesCommandExecutor(IptablesBaseCommand);

impl IptablesCommandExecutor {
    /// Returns a new and configured instance of IptablesCommandExecutor.
    pub fn new(base_command: IptablesBaseCommand) -> IptablesCommandExecutor {
        IptablesCommandExecutor(base_command)
    }
}

impl Executor for IptablesCommandExecutor {
    /// Executes the `iptables` command with the given arguments.
    fn execute(&self, args: Vec<String>) -> FirewallResult<()> {
        // TODO: How to test this?
        let exit_status = Command::new(self.0.get_command())
            .args(args)
            .spawn()?
            .wait()?;
        if exit_status.success() {
            Ok(())
        } else {
            Err(FirewallError::IptablesError(exit_status.code()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iptables_base_command_new() {
        assert_eq!(IptablesCommandExecutor(IptablesBaseCommand::Iptables),
                   IptablesCommandExecutor::new(IptablesBaseCommand::Iptables));
        assert_eq!(String::from("ip6tables"), IptablesBaseCommand::Ip6tables.get_command());
    }

    #[test]
    fn test_iptables_base_command_get_command() {
        assert_eq!(String::from("iptables"), IptablesBaseCommand::Iptables.get_command());
        assert_eq!(String::from("ip6tables"), IptablesBaseCommand::Ip6tables.get_command());
    }
}