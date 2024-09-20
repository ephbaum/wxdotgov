# WXdotGOV

**Note: This application is under development. Some functionalities are implemented, while others are planned.

## Implemented Functionalities

- Accepts a location and returns the weather forecast from a zip code, zip plus 4, city, or city & state.
- Fetches latitude and longitude from `nominatim.openstreetmap.org`.
- Fetches weather data from `api.weather.gov`.

## Planned Functionalities

- Additional input validation and error handling.
- Improved user interface and experience.
- Support for more location input formats.

## Examples

- `wxdotgov 12345`
- `wxdotgov 12345-6789`
- `wxdotgov "New York"`
- `wxdotgov "Seattle, WA"`

## Resources

- `nominatim.openstreetmap.org`
    - [Search](https://nominatim.org/release-docs/develop/api/Search/)
        - GET https://nominatim.openstreetmap.org/search?city=ketchikan&format=json
        - GET https://nominatim.openstreetmap.org/search?postalcode=99901&format=json
- `api.weather.gov`
    - [Docs](https://www.weather.gov/documentation/services-web-api) 
    - Basic Workflow
        - GET https://api.weather.gov/points/47.5619,-122.625
        - GET https://api.weather.gov/gridpoints/SEW/115,68/forecast

## Why?

I've been enjoying playing with Rust Lang and I figured: "why not make a terminal application from an open service like so many before me?"

## How

I'm just noodling on this as I go to learn, most of this was written with Co-pilot and poking around `cargo docs` with a bit of prior experience toying with Rust.

I'm a Rust novice and I'm open to [feedback](https://github.com/ephbaum/wxdotgov/issues). 
