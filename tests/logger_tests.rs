use log::*;
use rogue_oxide::logging::{Logger, Verbosity};

#[test]
fn test() {
    // Arrange
    Logger::init_new(Verbosity::Trace);

    // Act
    error!("This is an error message");
    warn!("This is a warning message");
    info!("This is an info message");
    debug!("This is a debug message");
    trace!("This is a trace message");
}
