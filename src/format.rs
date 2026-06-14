use chrono::{DateTime, Local, Utc};
use tabled::{
    settings::{
        object::{Cell, Rows},
        Color, Modify, Style, Width,
    },
    Table, Tabled,
};

use crate::api::MatchInfo;

#[derive(Clone, Tabled)]
pub struct MatchRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Match")]
    match_: String,
    #[tabled(rename = "Venue")]
    venue: String,
}

fn format_countdown(hours: f64) -> String {
    if hours < 1.0 {
        "1h+".to_string()
    } else {
        format!("{}h+", (hours.ceil() as i64))
    }
}

fn time_status_text(match_info: &MatchInfo) -> (String, Option<Color>) {
    let now = Utc::now();
    let local_dt: DateTime<Local> = match_info.datetime.into();
    let local_time = local_dt.format("%-I:%M %p").to_string();
    let weekday = local_dt.format("%a").to_string();

    match match_info.status_state.as_str() {
        "post" => {
            let score = format!(
                "{}-{}",
                match_info.score_a.as_deref().unwrap_or("0"),
                match_info.score_b.as_deref().unwrap_or("0")
            );
            (format!("FT {}", score), Some(Color::FG_RED))
        }
        "in" => {
            let clock = match_info.display_clock.as_deref().unwrap_or("00:00");
            let score = format!(
                "{}-{}",
                match_info.score_a.as_deref().unwrap_or("0"),
                match_info.score_b.as_deref().unwrap_or("0")
            );
            (
                format!("LIVE {} {}", clock, score),
                Some(Color::FG_YELLOW),
            )
        }
        _ => {
            let duration = match_info.datetime.signed_duration_since(now);
            let hours = duration.num_seconds() as f64 / 3600.0;

            if hours <= 24.0 && hours > 0.0 {
                let countdown = format_countdown(hours);
                (
                    format!("⏱ {} {} ({})", weekday, local_time, countdown),
                    Some(Color::FG_CYAN),
                )
            } else {
                (format!("{} {}", weekday, local_time), None)
            }
        }
    }
}

pub struct RowStyle {
    time_color: Option<Color>,
}

pub fn build_table_rows(events: &[MatchInfo], is_next: bool) -> Vec<(MatchRow, RowStyle)> {
    let now = Utc::now();

    events
        .iter()
        .filter(|e| {
            if is_next {
                e.status_state == "pre" && e.datetime > now
            } else {
                true
            }
        })
        .map(|e| {
            let (time, time_color) = time_status_text(e);
            let match_ = format!("{} vs {}", e.team_a, e.team_b);
            let venue = e.venue.clone();

            (
                MatchRow { time, match_, venue },
                RowStyle {
                    time_color,
                },
            )
        })
        .collect()
}

pub fn render_table(rows: Vec<(MatchRow, RowStyle)>) -> String {
    let data: Vec<MatchRow> = rows.iter().map(|(r, _)| r.clone()).collect();
    let mut table = Table::new(data);

    let header_style = Color::BOLD | Color::FG_CYAN;
    let match_style = Color::BOLD | Color::FG_WHITE;
    let venue_style = Color::FG_BRIGHT_BLACK;

    table
        .with(Style::modern())
        .with(Width::list([26, 22]))
        .with(Modify::new(Rows::first()).with(header_style));

    for (i, (_, style)) in rows.iter().enumerate() {
        let row_idx = i + 1;
        if let Some(color) = &style.time_color {
            table.with(Modify::new(Cell::new(row_idx, 0)).with(color.clone()));
        }
        table.with(Modify::new(Cell::new(row_idx, 1)).with(match_style.clone()));
        table.with(Modify::new(Cell::new(row_idx, 2)).with(venue_style.clone()));
    }

    table.to_string()
}
