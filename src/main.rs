use std::process::ExitCode;

use red_oxide::hosting::HostBuilder;

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
