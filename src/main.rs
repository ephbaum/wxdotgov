//! wxdotgov
//!
//! Takes a US postal code, or a city name with an optional state code, and
//! prints that location's weather forecast.
//!
//! The lookup runs in three steps:
//!
//! 1. Geocode the location to a latitude/longitude via nominatim.openstreetmap.org
//! 2. Resolve those coordinates to a forecast office and grid point via api.weather.gov
//! 3. Fetch and print the forecast for that grid point
//!
//! Examples:
//!
//! ```text
//! $ wxdotgov --zip 12345
//! $ wxdotgov --city "New York"
//! $ wxdotgov --city Seattle --state WA
//! $ wxdotgov --city Seattle --state WA --forecast-type hourly --pretty
//! $ wxdotgov --zip 12345 --forecast-type hourly --limit 6
//! ```
//!
//! Fetching lives here; deciding what the output says lives in [`render`].

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};

mod http;
mod nominatim;
mod render;
mod weatherdotgov;

use crate::nominatim::get_lat_lon;
use crate::render::{render_detailed, render_hourly, Style, DEFAULT_LIMIT};
use crate::weatherdotgov::{get_detailed_forecast, get_hourly_forecast, get_weather_point};

#[derive(Debug, PartialEq)]
pub enum LocationInput {
    PostalCode(String),
    PostalCodePlusFour(String, String),
    City(String),
    CityWithState(String, String),
}

/// Parse a US ZIP code, accepting both 5-digit and ZIP+4 forms.
///
/// The ZIP+4 form has always been documented but was never parsed, leaving
/// `PostalCodePlusFour` unconstructible outside tests.
fn parse_zip(zip: &str) -> Result<LocationInput> {
    let zip = zip.trim();

    let is_digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());

    if let Some((base, plus_four)) = zip.split_once('-') {
        if base.len() == 5 && is_digits(base) && plus_four.len() == 4 && is_digits(plus_four) {
            return Ok(LocationInput::PostalCodePlusFour(
                base.to_string(),
                plus_four.to_string(),
            ));
        }
    } else if zip.len() == 5 && is_digits(zip) {
        return Ok(LocationInput::PostalCode(zip.to_string()));
    }

    bail!(
        "'{zip}' is not a valid US ZIP code. Expected 5 digits (12345) \
         or ZIP+4 (12345-6789)."
    )
}

/// Turn the parsed CLI arguments into a single location query.
fn build_location_input(
    zip: Option<String>,
    city: Option<String>,
    state: Option<String>,
) -> Result<LocationInput> {
    // clap's required ArgGroup guarantees exactly one of zip/city is present.
    match (zip, city) {
        (Some(zip), _) => {
            if state.is_some() {
                // Previously ignored in silence, which looked like the state
                // had been applied to the lookup.
                eprintln!("warning: --state is ignored when --zip is given");
            }
            parse_zip(&zip)
        }
        (None, Some(city)) => Ok(match state {
            Some(state) => LocationInput::CityWithState(city, state),
            None => LocationInput::City(city),
        }),
        (None, None) => unreachable!("clap's required ArgGroup guarantees zip or city"),
    }
}

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    arg_required_else_help = true,
    group = clap::ArgGroup::new("location")
        .required(true)
        .args(["zip", "city"]),
)]
struct Args {
    /// ZIP code in the U.S.
    #[arg(short, long, group = "location")]
    zip: Option<String>,

    /// City name (when using city/state search)
    #[arg(short, long, group = "location")]
    city: Option<String>,

    /// State abbreviation (e.g., CA)
    #[arg(short, long)]
    state: Option<String>,

    /// Enable pretty output with colors and formatting.
    #[arg(long)]
    pretty: bool,

    /// Forecast type to display. Options: detailed or hourly.
    #[arg(long, value_enum, default_value_t = ForecastType::Detailed)]
    forecast_type: ForecastType,

    /// Maximum number of forecast periods to print. Use 0 for all of them.
    #[arg(short = 'n', long, default_value_t = DEFAULT_LIMIT)]
    limit: usize,
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
enum ForecastType {
    Detailed,
    Hourly,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments.
    let args = Args::parse();

    // Build the location input.
    let location_input = build_location_input(args.zip, args.city, args.state)?;

    // Step 1: Geocode with Nominatim.
    //
    // Progress lines go to stderr so stdout carries only the forecast: piping
    // this to a file previously interleaved a resolved place name and a raw
    // API URL above the output. The `--state` warning below already used
    // stderr, so the two streams were being mixed inconsistently.
    let location = get_lat_lon(location_input, None).await?;
    eprintln!("Location found: {}", location.display_name);

    // Step 2: Get points data from Weather.gov.
    let points_resp = get_weather_point(&location.lat, &location.lon, None).await?;

    // Select the forecast URL based on the chosen forecast type.
    let forecast_url = match args.forecast_type {
        ForecastType::Hourly => points_resp
            .properties
            .forecast_hourly
            .as_ref()
            .context("Hourly forecast not available for this location")?,
        ForecastType::Detailed => &points_resp.properties.forecast,
    };

    eprintln!("Fetching forecast from: {forecast_url}");

    let style = Style::from_pretty_flag(args.pretty);
    // 0 is the "no limit" spelling; every other value is taken literally.
    let limit = (args.limit != 0).then_some(args.limit);

    // Step 3: Fetch and display the forecast.
    let output = match args.forecast_type {
        ForecastType::Detailed => {
            let forecast_resp = get_detailed_forecast(forecast_url).await?;
            render_detailed(&forecast_resp.properties.periods, style, limit)
        }
        ForecastType::Hourly => {
            let hourly_forecast_resp = get_hourly_forecast(forecast_url).await?;
            render_hourly(&hourly_forecast_resp.properties.periods, style, limit)
        }
    };
    print!("{output}");

    Ok(())
}

#[cfg(test)]
mod tests {
    mod api_tests;
    mod app_tests;
    mod integration_tests;
    mod location_tests;
    mod render_tests;
}
