# WXdotGOV

**Note: This is WIP and doesn't really do anything yet. You've been warned.

## Plan

This is meant to be a command application build in [Rust](https://www.rust-lang.org/)

Should accept a location and return the weather forecast from a zip code, zip plus 4, city, or city & state from `api.weather.gov` using a geoencoded latitude and longitude data using `nominatim.openstreetmap.org`.

## Resources

- `nominatim.openstreetmap.org`
    - [Search](https://nominatim.org/release-docs/develop/api/Search/)
        - GET https://nominatim.openstreetmap.org/search?city=ketchikan&format=json
        - GET https://nominatim.openstreetmap.org/search?postalcode=99901&format=json
- `api.weather.gov`
    - [Docs](https://www.weather.gov/documentation/services-web-api) 
    - Basic Worflow
        - GET https://api.weather.gov/points/47.5619,-122.625
        - GET https://api.weather.gov/gridpoints/SEW/115,68/forecast

## Why?

I've been enjoying playing with Rust Lang and I figured: "why not make a terminal application from an open service like so many before me?"

## How

I'm just noodling on this as I go to learn, most of this was written with Co-pilot and poking around `cargo docs` with a bit of prior experience toying with Rust.

I'm a Rust novice and I'm open to [feedback](https://github.com/ephbaum/wxdotgov/issues). 

## Running Tests

To run the tests, use the following command:

```sh
cargo test
```

This will execute all the tests, including the integration tests that mock external API calls.
