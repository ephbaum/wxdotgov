# WXdotGOV

[![CI](https://github.com/ephbaum/wxdotgov/actions/workflows/ci.yml/badge.svg)](https://github.com/ephbaum/wxdotgov/actions/workflows/ci.yml)

A command-line weather application written in Rust that fetches weather forecasts from the National Weather Service API.

## Features

- Location search by:
  - ZIP code (5-digit or ZIP+4)
  - City name
  - City and state combination
- Two forecast types:
  - Detailed forecast (default)
  - Hourly forecast
- Pretty printing with colored output
- Error handling with informative messages
- Uses OpenStreetMap's Nominatim for geocoding
- Uses the National Weather Service API for weather data

## Installation

Requires Rust 1.88 or newer (see `rust-version` in `Cargo.toml`; CI verifies
it on every run). Then:

```bash
# Clone the repository
git clone https://github.com/ephbaum/wxdotgov.git
cd wxdotgov

# Build the project
cargo build --release

# The binary will be available in target/release/wxdotgov
```

## Usage

```bash
# Get help and see all available options
wxdotgov --help

# Get weather by ZIP code
wxdotgov --zip 98101

# ZIP+4 is accepted too
wxdotgov --zip 98101-1234

# Get weather by city and state
wxdotgov --city "Seattle" --state WA

# Get weather by city only (less precise)
wxdotgov --city "Seattle"

# Get hourly forecast with pretty printing
wxdotgov --city "Seattle" --state WA --forecast-type hourly --pretty

# Get detailed forecast with pretty printing
wxdotgov --city "New York" --state NY --pretty

# Show only the next 6 hourly periods
wxdotgov --zip 98101 --forecast-type hourly --limit 6

# Show every period the API returned
wxdotgov --zip 98101 --forecast-type hourly --limit 0
```

### Command-line Options

- `-z, --zip <ZIP>`: ZIP code in the U.S. (`12345` or `12345-6789`). Ignores `--state`.
- `-c, --city <CITY>`: City name
- `-s, --state <STATE>`: State abbreviation (e.g., CA)
- `--pretty`: Enable pretty output with colors and formatting
- `--forecast-type <TYPE>`: Type of forecast to display [possible values: detailed, hourly]
- `-n, --limit <N>`: Maximum number of forecast periods to print [default: 24].
  Use `0` for all of them. The NWS hourly endpoint returns a week-plus of
  entries, so the default keeps `--forecast-type hourly` readable; the detailed
  forecast returns roughly 14 periods, so the default is a no-op there.
- `-h, --help`: Print help
- `-V, --version`: Print version

### Output Streams

The forecast is written to stdout. Progress lines (`Location found: ...`,
`Fetching forecast from: ...`) and warnings go to stderr, so redirecting stdout
gives you just the forecast:

```bash
wxdotgov --zip 98101 > today.txt   # today.txt holds only the forecast
```

## APIs Used

- **Nominatim (OpenStreetMap)**
  - Used for geocoding (converting location names to coordinates)
  - [API Documentation](https://nominatim.org/release-docs/develop/api/Search/)

- **National Weather Service API**
  - Used for weather forecasts
  - [API Documentation](https://www.weather.gov/documentation/services-web-api)

### Environment Variables

- `WXDOTGOV_USER_AGENT`: overrides the `User-Agent` sent to both APIs. The
  default identifies the crate, its version, and this repository. Both the NWS
  API and Nominatim require an identifying User-Agent with usable contact
  details, so set this to your own contact if you run a fork.

## Error Handling

The application handles:

- **Invalid location input** — ZIP codes are validated before any network call,
  so `--zip abcde` fails immediately rather than being sent upstream.
- **Network failures** — every request carries a 10s timeout (5s to connect),
  so an unresponsive upstream fails fast instead of hanging.
- **Upstream errors** — HTTP status is checked before parsing, and the status
  code is reported. Error bodies are truncated, so an HTML error page from
  either service cannot flood the terminal. Nominatim rate limiting (HTTP 429)
  is called out specifically rather than surfacing as a JSON parse error.
- **Missing forecast data** — a location without an hourly forecast reports
  that, rather than panicking.

## Development

The test suite is fully offline — it mocks both upstream APIs, so it runs
without network access and never calls the live services.

```bash
cargo test                  # 30 tests, no network required
cargo clippy --all-targets  # warnings are denied
cargo fmt --all -- --check  # formatting is enforced
cargo audit                 # RUSTSEC advisories
```

Lint levels live in the `[lints]` table in `Cargo.toml` rather than in CI, so
these commands behave identically on your machine and in CI. `cargo audit`
runs against an unfiltered advisory database -- there is no ignore list, so a
new advisory turns CI red rather than being suppressed.

TLS comes from `rustls`, which is `reqwest` 0.13's default. The build needs no
system OpenSSL (no `libssl-dev`, no `pkg-config`); the crypto provider is
`aws-lc-rs`, which vendors its own C sources, so a working C compiler is the
only non-Rust build requirement.

## Contributing

Contributions are welcome — please open a Pull Request. CI runs the four checks
above on every PR and must be green to merge.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Why?

This project serves as both a useful weather tool and a learning exercise in Rust, demonstrating:
- API integration
- Error handling
- Command-line argument parsing
- Pretty printing and user interface
- Modular code organization
