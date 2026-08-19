//! Forecast rendering.
//!
//! Everything that decides *what* the forecast output says lives here and
//! returns a `String`; `main` only fetches and prints. Keeping that seam means
//! the output can be tested without spawning the binary or mocking the network
//! at process level.
//!
//! `--pretty` controls two independent things: colour and layout. Both output
//! blocks were previously written twice, once per branch, with different
//! arrangements of the same fields and nothing holding the two in parity --
//! adding a field to one branch left the other silently printing the old set.
//! Rendering both styles through one function makes that parity directly
//! assertable (see `tests/render_tests.rs`).
//!
//! Note the tests deliberately assert on field *values*, never on ANSI escape
//! codes. Asserting that `.cyan()` emits `\x1b[36m` tests the `colored` crate
//! rather than this project. It would also only ever pass in a configuration
//! real users never encounter: `colored` suppresses escapes when stdout is not
//! a TTY, so `wxdotgov --pretty > out.txt` correctly produces the pretty
//! *layout* with no colour at all.

use colored::*;

use crate::weatherdotgov::{HourlyPeriod, Period};

/// How a forecast is laid out and coloured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Plain,
    Pretty,
}

impl Style {
    /// `--pretty` is a flag, so the CLI only ever offers these two.
    pub fn from_pretty_flag(pretty: bool) -> Self {
        if pretty {
            Style::Pretty
        } else {
            Style::Plain
        }
    }
}

const SEPARATOR: &str = "-------------------------------------";

/// The number of periods to print when `--limit` is not given.
///
/// The NWS hourly endpoint returns a week-plus of hourly entries, and each one
/// prints two lines plus a separator, so an unbounded hourly run scrolls the
/// useful near-term forecast off the top of the terminal. The detailed endpoint
/// returns roughly 14 periods, so the same default is a no-op there in practice.
pub const DEFAULT_LIMIT: usize = 24;

/// Take at most `limit` periods, where `None` means "all of them".
fn limited<T>(periods: &[T], limit: Option<usize>) -> &[T] {
    match limit {
        Some(n) => &periods[..n.min(periods.len())],
        None => periods,
    }
}

fn header(text: &str, style: Style) -> String {
    match style {
        Style::Pretty => format!("\n{}\n\n", text.bold().underline().bright_white()),
        Style::Plain => format!("\n{text}\n\n"),
    }
}

fn separator(style: Style) -> String {
    match style {
        Style::Pretty => format!("{}\n", SEPARATOR.dimmed()),
        Style::Plain => format!("{SEPARATOR}\n"),
    }
}

/// Tell the reader that output was cut short, and how to see the rest.
///
/// Silently dropping periods would be indistinguishable from the API having
/// returned fewer of them.
fn truncation_note(total: usize, shown: usize, style: Style) -> String {
    if shown >= total {
        return String::new();
    }
    let text = format!(
        "... {} more period(s) not shown (use --limit 0 to show all)",
        total - shown
    );
    match style {
        Style::Pretty => format!("{}\n", text.dimmed()),
        Style::Plain => format!("{text}\n"),
    }
}

/// Render the daily/detailed forecast.
pub fn render_detailed(periods: &[Period], style: Style, limit: Option<usize>) -> String {
    let shown = limited(periods, limit);
    let mut out = header("Weather Forecast:", style);

    for period in shown {
        match style {
            Style::Pretty => {
                out.push_str(&format!("{}\n", period.name.bold().blue()));
                out.push_str(&format!("{}\n", period.detailed_forecast.green()));
            }
            Style::Plain => {
                out.push_str(&format!("{}: {}\n", period.name, period.detailed_forecast));
            }
        }
        out.push_str(&separator(style));
    }

    out.push_str(&truncation_note(periods.len(), shown.len(), style));
    out
}

/// Render the hourly forecast.
pub fn render_hourly(periods: &[HourlyPeriod], style: Style, limit: Option<usize>) -> String {
    let shown = limited(periods, limit);
    let mut out = header("Hourly Weather Forecast:", style);

    for period in shown {
        match style {
            Style::Pretty => {
                out.push_str(&format!("{}\n", period.start_time.bold().blue()));
                out.push_str(&format!(
                    "{}°{} | {} | Wind: {} {}\n",
                    period.temperature.to_string().yellow(),
                    period.temperature_unit.yellow(),
                    period.short_forecast.cyan(),
                    period.wind_speed.cyan(),
                    period.wind_direction.cyan()
                ));
            }
            Style::Plain => {
                out.push_str(&format!(
                    "{}: {}°{} | {} | Wind: {} {}\n",
                    period.start_time,
                    period.temperature,
                    period.temperature_unit,
                    period.short_forecast,
                    period.wind_speed,
                    period.wind_direction,
                ));
            }
        }
        out.push_str(&separator(style));
    }

    out.push_str(&truncation_note(periods.len(), shown.len(), style));
    out
}
