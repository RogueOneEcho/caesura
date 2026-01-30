use crate::prelude::*;
use tokio::process::Command;

/// Information required to create a [`Command`].
pub(crate) struct CommandInfo {
    /// Program to run
    pub program: String,
    /// Arguments to pass to the program
    pub args: Vec<String>,
}

impl CommandInfo {
    /// Create a [`Command`] from the program and its arguments
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_command(self) -> Command {
        let mut cmd = Command::new(self.program);
        cmd.args(self.args);
        cmd
    }

    /// Get a string representation of the CLI command.
    ///
    /// If an arg contains spaces it will be wrapped in double quotes, but no other escaping is
    /// applied so this method is not safe for execution.
    #[must_use]
    pub(crate) fn display(&self) -> String {
        self.args.iter().fold(self.program.clone(), |mut acc, arg| {
            acc.push(' ');
            if arg.contains(' ') {
                acc.push('"');
                acc.push_str(arg);
                acc.push('"');
            } else {
                acc.push_str(arg);
            }
            acc
        })
    }
}

impl Display for CommandInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.display())
    }
}
