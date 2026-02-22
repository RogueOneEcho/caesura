//! Platform and environment detection.

use std::env;

const TRUTHY: &[&str] = &["1", "true", "yes", "on", "y"];

/// Environment variable set in the Docker image to use container paths.
const DOCKER_ENV_VAR: &str = "CAESURA_DOCKER";

/// Environment variable that enables exact deterministic snapshot matching.
#[cfg(test)]
const DETERMINISTIC_ENV_VAR: &str = "CAESURA_DETERMINISTIC_TESTS";

/// Check if running in a Docker container.
pub(crate) fn is_docker() -> bool {
    is_env_var_truthy(DOCKER_ENV_VAR)
}

/// Check if deterministic snapshot tests are enabled.
#[cfg(test)]
pub(crate) fn is_deterministic() -> bool {
    is_env_var_truthy(DETERMINISTIC_ENV_VAR)
}

fn is_env_var_truthy(var: &str) -> bool {
    let Ok(value) = env::var(var) else {
        return false;
    };
    TRUTHY.iter().any(|t| value.eq_ignore_ascii_case(t))
}
