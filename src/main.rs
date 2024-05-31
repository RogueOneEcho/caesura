use colored::Colorize;
use log::error;
use std::process::ExitCode;

use red_oxide::hosting::HostBuilder;
use red_oxide::logging::{Logger, Trace};

#[tokio::main]
async fn main() -> ExitCode {
    match HostBuilder::new().build() {
        Ok(host) => {
            if host.execute().await {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            Logger::init_new(Trace);
            error!(
                "{} to build the application: {}",
                "Failed".red().bold(),
                error
            );
            ExitCode::FAILURE
        }
    }
}
