use reqwest::StatusCode;

#[allow(dead_code)]
pub(crate) fn status_code_and_reason(code: u16) -> String {
    StatusCode::from_u16(code)
        .ok()
        .and_then(|code| code.canonical_reason())
        .map(|reason| format!("{code} {reason}"))
        .unwrap_or(code.to_string())
}
