# worldcup — CLI tool for FIFA World Cup 2026 live info

## Overview

A command-line tool that fetches and displays live FIFA World Cup 2026 data from ESPN's public API. Written in Rust. Single binary, no runtime dependencies.

## Data Source

ESPN public API — no authentication required.

```
GET https://site.api.espn.com/apis/site/v2/sports/soccer/fifa.world/scoreboard
GET https://site.api.espn.com/apis/site/v2/sports/soccer/fifa.world/scoreboard?dates=YYYYMMDD
```

## Commands

### `worldcup` (default)

Shows today's matches in a table.

| Time column | Condition |
|---|---|
| `FT 2-0` | Finished match (post) |
| `LIVE 45' 1-0` | In-progress match (in) |
| `⏱ Sun 2:00 AM (3h+)` | Upcoming today, kickoff ≤ 24h |
| `Sun 2:00 AM` | Upcoming today, kickoff > 24h |

### `worldcup next`

Shows only upcoming matches for the next 7 days.

| Time column | Condition |
|---|---|
| `⏱ Sun 2:00 AM (3h+)` | Kickoff ≤ 24h from now, with countdown |
| `Mon 12:00 PM` | Kickoff > 24h, weekday + time only |

Countdown format: rounded hours with `+` suffix (e.g. `3h+`, `12h+`, `1h+` for < 1h).

### Table layout

Three columns, rendered with a TUI table library:

```
┌────────────────────────────┬────────────────────────┬──────────────────────┐
│ Time                       │ Match                  │ Venue                │
├────────────────────────────┼────────────────────────┼──────────────────────┤
│ FT 2-0                     │ Australia vs Türkiye   │ BC Place             │
│ ⏱ Sun 2:00 AM (3h+)       │ Germany vs Curaçao     │ NRG Stadium          │
│ Mon 12:00 PM               │ Cape Verde vs Spain    │ Mercedes-Benz Stadium│
└────────────────────────────┴────────────────────────┴──────────────────────┘
```

- **Time:** left-aligned, 26-char width
- **Match:** left-aligned, 22-char width (Team A `vs` Team B)
- **Venue:** left-aligned, no truncation

## Tech Stack

| Dependency | Purpose |
|---|---|
| `clap` (derive) | CLI argument parsing |
| `reqwest` (json, native-tls) | HTTP client |
| `tokio` (full) | Async runtime |
| `serde` + `serde_json` | JSON deserialization |
| `chrono` (serde) | Date/time parsing and math |
| `tabled` | Table rendering with borders |
| `colored` | Terminal styling (cyan/white/red/dimmed) |

## Project Structure

```
worldcup/
├── Cargo.toml
└── src/
    ├── main.rs      # clap setup, command dispatch
    ├── api.rs       # reqwest client, serde types for ESPN response
    └── format.rs    # time math (countdown, local format), table builder
```

## Display Rules

### Team name display

Use `team.shortDisplayName` from the API for both teams.

### Time math

- Parse `event.date` (ISO 8601 UTC) → convert to local time via `chrono`
- Countdown = `event.date - now`
- ≤ 24h → show `(Xh+)` where X = `ceil(hours)`, minimum `1h+`
- > 24h → show only weekday + time (no countdown)

### Color scheme

- `⏱` marker: cyan
- Score / Status (FT, LIVE): red for finished, yellow for live
- Team names: white/bold
- Venue: dimmed
- Table borders: default
- Header row: bold/cyan
