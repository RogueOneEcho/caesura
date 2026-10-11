use crate::prelude::*;

/// Information required to create a [`TokioCommand`].
pub(crate) struct CommandInfo {
    /// Program to run
    pub program: String,
    /// Arguments to pass to the program
    pub args: Vec<String>,
}

impl CommandInfo {
    /// Create a [`TokioCommand`] from the program and its arguments.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_command(self) -> TokioCommand {
        let mut cmd = TokioCommand::new(self.program);
        cmd.args(self.args);
        set_own_process_group(&mut cmd);
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

/// Place the child in its own process group so it does not receive the terminal's SIGINT.
///
/// - Uses a [process group ID of 0](https://docs.rs/tokio/latest/tokio/process/struct.Command.html#method.process_group) to create a new group
#[cfg(unix)]
fn set_own_process_group(cmd: &mut TokioCommand) {
    const NEW_PROCESS_GROUP: i32 = 0;
    cmd.process_group(NEW_PROCESS_GROUP);
}

/// Place the child in its own process group so it does not receive Ctrl-C.
///
/// - Uses [`CREATE_NEW_PROCESS_GROUP`](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags#CREATE_NEW_PROCESS_GROUP)
#[cfg(windows)]
fn set_own_process_group(cmd: &mut TokioCommand) {
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
}
