//! Automatic tracker report generation for reportable source issues.

pub(crate) use report_action::*;
pub(crate) use report_renderer::*;
pub(crate) use source_issues_renderer::*;
pub(crate) use source_reporter::*;

mod report_action;
mod report_renderer;
mod source_issues_renderer;
mod source_reporter;
#[cfg(test)]
mod tests;
