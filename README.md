# WXdotGOV

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

Make sure you have Rust and Cargo installed. Then:

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
```

### Command-line Options

- `-z, --zip <ZIP>`: ZIP code in the U.S. (`12345` or `12345-6789`). Ignores `--state`.
- `-c, --city <CITY>`: City name
- `-s, --state <STATE>`: State abbreviation (e.g., CA)
- `--pretty`: Enable pretty output with colors and formatting
- `--forecast-type <TYPE>`: Type of forecast to display [possible values: detailed, hourly]
- `-h, --help`: Print help
- `-V, --version`: Print version

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

The application includes robust error handling for:
- Invalid location inputs
- Network request failures
- API response parsing
- Missing forecast data

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Why?

This project serves as both a useful weather tool and a learning exercise in Rust, demonstrating:
- API integration
- Error handling
- Command-line argument parsing
- Pretty printing and user interface
- Modular code organization
