use caesura::{HostBuilder, init_logger};
use log::error;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    #[cfg_attr(not(feature = "demo"), allow(unused_mut))]
    let mut builder = HostBuilder::new();
    #[cfg(feature = "demo")]
    builder.with_demo().await;
    let host = match builder.build() {
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
