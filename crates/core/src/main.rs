use caesura::{HostBuilder, init_logger};
use log::error;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let host = match HostBuilder::new().build() {
        Ok(host) => host,
        Err(error) => {
            init_logger();
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
        Err(report) => {
            eprintln!("{report:?}");
            ExitCode::FAILURE
        }
    }
}
