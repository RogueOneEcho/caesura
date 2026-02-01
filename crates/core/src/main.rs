use caesura::HostBuilder;
use log::error;
use rogue_logging::LoggerBuilder;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let host = match HostBuilder::new().build() {
        Ok(host) => host,
        Err(error) => {
            let _ = LoggerBuilder::new().create();
            error!("{error}");
            return ExitCode::FAILURE;
        }
    };
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
