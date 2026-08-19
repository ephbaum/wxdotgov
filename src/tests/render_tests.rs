#[cfg(test)]
mod tests {
    use crate::render::{render_detailed, render_hourly, Style, DEFAULT_LIMIT};
    use crate::weatherdotgov::{HourlyPeriod, Period};

    /// Count the separator rules, which is how many periods actually printed.
    /// Matching on a dash substring would over-count inside the rule itself.
    fn separator_count(out: &str) -> usize {
        out.lines()
            .filter(|line| !line.is_empty() && line.chars().all(|c| c == '-'))
            .count()
    }

    fn detailed_periods(count: usize) -> Vec<Period> {
        (0..count)
            .map(|i| Period {
                name: format!("Period{i}"),
                detailed_forecast: format!("Forecast text {i}"),
            })
            .collect()
    }

    fn hourly_periods(count: usize) -> Vec<HourlyPeriod> {
        (0..count)
            .map(|i| HourlyPeriod {
                start_time: format!("2024-01-28T{i:02}:00:00-08:00"),
                temperature: 50 + i as i32,
                temperature_unit: "F".to_string(),
                wind_speed: format!("{i} mph"),
                wind_direction: "SSW".to_string(),
                short_forecast: format!("Short forecast {i}"),
            })
            .collect()
    }

    /// The parity property #40 exists to protect: every field value printed in
    /// one style is printed in the other. Adding a field to only one branch
    /// fails here. Deliberately says nothing about ANSI escapes, so it does not
    /// break when `colored` releases a new major.
    #[test]
    fn detailed_prints_the_same_fields_in_both_styles() {
        let periods = detailed_periods(3);
        let plain = render_detailed(&periods, Style::Plain, None);
        let pretty = render_detailed(&periods, Style::Pretty, None);

        for period in &periods {
            for field in [&period.name, &period.detailed_forecast] {
                assert!(plain.contains(field.as_str()), "plain missing {field}");
                assert!(pretty.contains(field.as_str()), "pretty missing {field}");
            }
        }
    }

    #[test]
    fn hourly_prints_the_same_fields_in_both_styles() {
        let periods = hourly_periods(3);
        let plain = render_hourly(&periods, Style::Plain, None);
        let pretty = render_hourly(&periods, Style::Pretty, None);

        for period in &periods {
            let fields = [
                period.start_time.clone(),
                period.temperature.to_string(),
                period.temperature_unit.clone(),
                period.wind_speed.clone(),
                period.wind_direction.clone(),
                period.short_forecast.clone(),
            ];
            for field in fields {
                assert!(plain.contains(&field), "plain missing {field}");
                assert!(pretty.contains(&field), "pretty missing {field}");
            }
        }
    }

    #[test]
    fn both_styles_carry_the_header_and_one_separator_per_period() {
        for style in [Style::Plain, Style::Pretty] {
            let detailed = render_detailed(&detailed_periods(3), style, None);
            assert!(detailed.contains("Weather Forecast:"));
            assert_eq!(separator_count(&detailed), 3, "{style:?}");

            let hourly = render_hourly(&hourly_periods(2), style, None);
            assert!(hourly.contains("Hourly Weather Forecast:"));
            assert_eq!(separator_count(&hourly), 2, "{style:?}");
        }
    }

    #[test]
    fn no_limit_prints_every_period() {
        let periods = hourly_periods(50);
        let out = render_hourly(&periods, Style::Plain, None);
        assert_eq!(separator_count(&out), 50);
        assert!(!out.contains("not shown"));
    }

    #[test]
    fn limit_truncates_and_says_how_many_were_dropped() {
        let out = render_hourly(&hourly_periods(50), Style::Plain, Some(2));

        assert!(out.contains("2024-01-28T00:00:00-08:00"));
        assert!(out.contains("2024-01-28T01:00:00-08:00"));
        assert!(!out.contains("2024-01-28T02:00:00-08:00"));
        assert!(out.contains("48 more period(s) not shown"));
        assert!(out.contains("--limit 0"));
    }

    #[test]
    fn limit_applies_to_detailed_too() {
        let out = render_detailed(&detailed_periods(5), Style::Plain, Some(1));
        assert!(out.contains("Period0"));
        assert!(!out.contains("Period1"));
        assert!(out.contains("4 more period(s) not shown"));
    }

    /// The default is chosen so it is a no-op for the ~14-period detailed
    /// forecast while bounding the week-plus hourly one.
    #[test]
    fn default_limit_does_not_truncate_a_detailed_forecast() {
        let out = render_detailed(&detailed_periods(14), Style::Plain, Some(DEFAULT_LIMIT));
        assert_eq!(separator_count(&out), 14);
        assert!(!out.contains("not shown"));
    }

    #[test]
    fn limit_larger_than_the_period_list_is_not_an_error() {
        let out = render_detailed(&detailed_periods(2), Style::Plain, Some(99));
        assert_eq!(separator_count(&out), 2);
        assert!(!out.contains("not shown"));
    }

    #[test]
    fn empty_period_lists_render_just_the_header() {
        for style in [Style::Plain, Style::Pretty] {
            let out = render_detailed(&[], style, Some(DEFAULT_LIMIT));
            assert!(out.contains("Weather Forecast:"));
            assert_eq!(separator_count(&out), 0);
            assert!(!out.contains("not shown"));
        }
    }
}
