//! Extension trait for rendering diagnostics with miette's graphical output.

use miette::{Diagnostic, GraphicalReportHandler};

/// Extension trait for rendering [`Diagnostic`] types with fancy output.
pub(crate) trait DiagnosticExt {
    /// Render the diagnostic using miette's graphical handler.
    fn render(&self) -> String;
}

impl<T: Diagnostic> DiagnosticExt for T {
    fn render(&self) -> String {
        let mut output = String::new();
        GraphicalReportHandler::new()
            .render_report(&mut output, self)
            .expect("diagnostic should render");
        output
    }
}
