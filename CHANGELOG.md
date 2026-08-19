# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The repository carried no tags or releases before 0.1.0, so the entries below
are reconstructed from the issue and pull request history rather than from
version boundaries that never existed.

## [Unreleased]

Slated to become 0.1.0, the first release.

### Added

- Package metadata in `Cargo.toml` — `description`, `license`, `repository`,
  `readme`, `keywords`, and `categories`. The description is what `--help` now
  prints as its summary line; before this it printed none.
- A declared MSRV (`rust-version = "1.88"`), checked by a dedicated CI job so a
  dependency bump that raises the floor fails the build.
- `--limit` / `-n` to cap how many forecast periods are printed, defaulting to
  24, with a note reporting how many were withheld (#33).
- A weekly scheduled CI run, so a newly published RUSTSEC advisory is caught
  during quiet weeks rather than waiting for the next commit.
- Dependabot coverage for cargo and github-actions (#28).

### Changed

- Progress output (`Location found: ...`, `Fetching forecast from: ...`) moved
  from stdout to stderr, so redirecting stdout yields only the forecast.
- Weather.gov error bodies are truncated before being reported, matching the
  handling Nominatim responses already had, and now carry the HTTP status.
- All three Weather.gov requests send `Accept: application/geo+json`; it was
  previously sent on `/points` only, leaving the forecast calls dependent on
  the server default.
- Forecast rendering extracted from `main` so output is testable without
  spawning the binary or mocking at process level (#40).
- Migrated reqwest 0.11 to 0.13 and switched TLS to rustls, dropping the
  system OpenSSL build requirement (#31).
- Lint denials declared in the `[lints]` table rather than via `RUSTFLAGS`, so
  a local build reproduces CI's denials and third-party crates are unaffected
  (#32).
- Format arguments inlined throughout, removing exposure to
  `clippy::uninlined_format_args` moving back into the `style` group.
- `nomatim` module renamed to `nominatim` (#36).

### Fixed

- `--forecast-type hourly` failed against the live API because `HourlyPeriod`
  was missing its camelCase serde renames. The default detailed forecast was
  unaffected (#19).
- Requests sent a placeholder User-Agent, violating both the NWS and Nominatim
  usage policies; it is now derived from the crate name and version and is
  overridable via `WXDOTGOV_USER_AGENT` (#23).
- No HTTP timeouts, so a hung connection blocked the CLI indefinitely; requests
  now time out after 10s, with a 5s connect timeout (#24).
- Nominatim response status went unchecked, so a blocked or rate-limited
  request surfaced as a misleading JSON parse error (#25).
- ZIP+4 input was documented but never parsed (#29).
- `--state` was silently ignored when `--zip` was given; it now warns (#29).
- Cleared 7 RustSec advisories in transitive dependencies (#22).
- CI had never actually run: the workflow never triggered on push and used
  archived actions (#27).
- Two tests described as mocked were calling the live api.weather.gov (#21),
  and the hourly test mocked snake_case JSON that masked the bug in #19 (#20).

[Unreleased]: https://github.com/ephbaum/wxdotgov/compare/main...HEAD
