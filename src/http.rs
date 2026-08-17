//! Shared HTTP client construction.
//!
//! Both upstream services this tool talks to — the National Weather Service API
//! and OpenStreetMap's Nominatim — require a User-Agent that identifies the
//! application and provides a way to make contact. Nominatim's usage policy is
//! explicit that requests without one may be blocked. Building the client in one
//! place keeps that header from drifting between call sites.
//!
//! `reqwest::Client` owns a connection pool and is designed to be built once and
//! reused, so it is cached here rather than constructed per request.

use anyhow::{Context, Result};
use std::sync::OnceLock;
use std::time::Duration;

/// Contact point advertised to upstream APIs. Both services accept a project URL
/// in place of an email address.
const CONTACT: &str = "https://github.com/ephbaum/wxdotgov";

/// Ceiling on a single request. `reqwest` applies no timeout by default, which
/// previously let an unresponsive upstream hang the CLI indefinitely.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Environment variable allowing operators to override the User-Agent, e.g. to
/// substitute their own contact address when running a fork.
const USER_AGENT_ENV: &str = "WXDOTGOV_USER_AGENT";

/// The User-Agent sent to both upstream APIs.
///
/// Derived from the crate name and version so it cannot fall out of step with
/// `Cargo.toml` the way the previous hardcoded `RustWeatherCLI/0.1` string did.
pub fn user_agent() -> String {
    match std::env::var(USER_AGENT_ENV) {
        Ok(custom) if !custom.trim().is_empty() => custom,
        _ => format!(
            "{}/{} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            CONTACT
        ),
    }
}

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// A process-wide HTTP client carrying the shared User-Agent and timeouts.
pub fn client() -> Result<&'static reqwest::Client> {
    if let Some(client) = CLIENT.get() {
        return Ok(client);
    }

    let client = reqwest::Client::builder()
        .user_agent(user_agent())
        .timeout(REQUEST_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
        .context("Error building HTTP client")?;

    // A concurrent caller may have won the race; its client is equivalent.
    Ok(CLIENT.get_or_init(|| client))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_reports_crate_name_and_version() {
        // Guards against the placeholder-contact regression: the UA must carry
        // the real crate version and a reachable contact, never a stub address.
        let ua = user_agent();
        assert!(ua.starts_with(concat!(env!("CARGO_PKG_NAME"), "/")), "got: {}", ua);
        assert!(ua.contains(env!("CARGO_PKG_VERSION")), "got: {}", ua);
        assert!(ua.contains(CONTACT), "got: {}", ua);
        assert!(!ua.contains("example.com"), "placeholder contact in UA: {}", ua);
    }

    #[test]
    fn client_is_reused_across_calls() {
        let first = client().expect("client should build");
        let second = client().expect("client should build");
        assert!(std::ptr::eq(first, second), "client should be cached, not rebuilt");
    }
}
