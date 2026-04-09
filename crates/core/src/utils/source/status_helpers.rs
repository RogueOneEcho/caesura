use reqwest::StatusCode;

/// Format an HTTP status code as `{code} {reason}`, falling back to just the code if unknown.
#[allow(dead_code)]
pub(crate) fn status_code_and_reason(code: u16) -> String {
    StatusCode::from_u16(code)
        .ok()
        .and_then(|code| code.canonical_reason())
        .map(|reason| format!("{code} {reason}"))
        .unwrap_or(code.to_string())
}
