# worldcup

Single-binary Rust CLI that fetches live FIFA World Cup 2026 data from ESPN's public API (no auth).

## Commands

```sh
cargo build                # debug build
cargo run                  # today's matches
cargo run -- next          # upcoming matches in next 7 days
```

No tests, no lint/format config, no CI.

## Project structure

```
src/
  main.rs     # clap setup, command dispatch
  api.rs      # reqwest client, serde types for ESPN scoreboard API
  format.rs   # time math, table builder, flag emoji mapping
```

- `api.rs`: `Team` struct deserializes `shortDisplayName` + `abbreviation` from ESPN; `MatchInfo` carries name + code for each team.
- `format.rs`: `flag()` maps ESPN 3-letter abbreviation codes to flag emoji strings (~130 countries).
- Display: 3-column table (Time, Match, Venue) via `tabled` with `ansi` feature enabled.

## ESPN API

```
GET https://site.api.espn.com/apis/site/v2/sports/soccer/fifa.world/scoreboard
GET https://site.api.espn.com/apis/site/v2/sports/soccer/fifa.world/scoreboard?dates=YYYYMMDD
```

## Style notes

- Table uses ANSI escape codes embedded in cell content (e.g. `\x1b[90m` for gray "vs" text). `tabled` must have `ansi` feature enabled so width calculation strips these.
- Column widths: Time=26, Match=30 (22 + room for flag emojis).
- `req.md` is the spec doc (not a README).
- No external formatter config; follows default `rustfmt`.
