use rogue_oxide::hosting::HostBuilder;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let host = HostBuilder::new().build();
    match host.execute().await {
        Ok(status) => {
            if status {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            error.log();
            ExitCode::FAILURE
        }
    }
}
