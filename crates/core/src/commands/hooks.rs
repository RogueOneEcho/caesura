use crate::prelude::*;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, process};
use tokio::fs::{remove_file, write};
use tokio::process::Command;

/// Execute a bash hook script with a YAML payload file argument.
pub(crate) async fn execute_yaml_hook<TAction: Debug + Display + Copy, TPayload: Serialize>(
    hook_path: &Path,
    payload: &TPayload,
    serialize_action: TAction,
    write_action: TAction,
    execute_action: TAction,
) -> Result<(), Failure<TAction>> {
    let payload_yaml = serde_yaml::to_string(payload).map_err(Failure::wrap(serialize_action))?;
    let payload_path = get_hook_payload_path();
    write(&payload_path, payload_yaml)
        .await
        .map_err(Failure::wrap_with_path(write_action, &payload_path))?;
    let mut command = Command::new("bash");
    command.arg(hook_path).arg(&payload_path);
    let result = command
        .run()
        .await
        .map_err(Failure::wrap_with_path(execute_action, hook_path));
    if let Err(error) = remove_file(&payload_path).await {
        warn!(
            "{} to remove hook payload {}: {error}",
            "Failed".bold(),
            payload_path.display()
        );
    }
    result?;
    trace!("{} hook {}", "Executed".bold(), hook_path.display());
    Ok(())
}

fn get_hook_payload_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_nanos());
    env::temp_dir().join(format!("caesura-hook-{}-{timestamp}.yml", process::id()))
}
