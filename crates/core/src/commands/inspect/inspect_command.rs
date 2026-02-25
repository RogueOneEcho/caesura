use crate::prelude::*;

/// Inspect audio file metadata in a directory.
#[injectable]
pub(crate) struct InspectCommand {
    arg: Ref<InspectArg>,
}

impl InspectCommand {
    /// Execute [`InspectCommand`] from the CLI.
    pub(crate) fn execute_cli(&self) -> Result<bool, Failure<InspectAction>> {
        let factory = InspectFactory::new(true);
        let output = factory.create(&self.arg.inspect_path)?;
        println!("{output}");
        Ok(true)
    }
}
