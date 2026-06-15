use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ScoreboardResponse {
    pub events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    pub id: String,
    pub date: String,
    pub competitions: Vec<Competition>,
}

#[derive(Debug, Deserialize)]
pub struct Competition {
    pub competitors: Vec<Competitor>,
    pub status: Status,
    pub venue: Option<Venue>,
}

#[derive(Debug, Deserialize)]
pub struct Competitor {
    pub team: Team,
    pub score: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Team {
    #[serde(rename = "shortDisplayName")]
    pub short_display_name: String,
    pub abbreviation: String,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    #[serde(rename = "type")]
    pub type_: StatusType,
    pub displayclock: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusType {
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct Venue {
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GoalDetail {
    pub minute: String,
    pub scorer: String,
    pub team_code: String,
}

pub struct MatchInfo {
    pub datetime: DateTime<Utc>,
    pub team_a: String,
    pub team_b: String,
    pub code_a: String,
    pub code_b: String,
    pub score_a: Option<String>,
    pub score_b: Option<String>,
    pub status_state: String,
    pub display_clock: Option<String>,
    pub venue: String,
    pub event_id: String,
    pub goals: Vec<GoalDetail>,
}

#[derive(Debug, Deserialize)]
struct SummaryResponse {
    header: SummaryHeader,
    #[serde(rename = "keyEvents")]
    key_events: Vec<KeyEvent>,
}

#[derive(Debug, Deserialize)]
struct SummaryHeader {
    competitions: Vec<SummaryCompetition>,
}

#[derive(Debug, Deserialize)]
struct SummaryCompetition {
    competitors: Vec<SummaryCompetitor>,
}

#[derive(Debug, Deserialize)]
struct SummaryCompetitor {
    id: String,
    team: SummaryTeam,
}

#[derive(Debug, Deserialize)]
struct SummaryTeam {
    abbreviation: String,
}

#[derive(Debug, Deserialize)]
struct KeyEvent {
    clock: KeyEventClock,
    #[serde(rename = "type")]
    type_: KeyEventType,
    #[serde(rename = "shortText", default)]
    short_text: Option<String>,
    team: Option<KeyEventTeam>,
}

#[derive(Debug, Deserialize)]
struct KeyEventType {
    text: String,
}

#[derive(Debug, Deserialize)]
struct KeyEventClock {
    #[serde(rename = "displayValue")]
    display_value: String,
}

#[derive(Debug, Deserialize)]
struct KeyEventTeam {
    id: String,
}

pub async fn fetch_summary(event_id: &str) -> Result<Vec<GoalDetail>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://site.api.espn.com/apis/site/v2/sports/soccer/fifa.world/summary?event={}",
        event_id
    );
    let resp: SummaryResponse = client.get(&url).send().await?.json().await?;

    let team_map: std::collections::HashMap<String, String> = resp
        .header
        .competitions
        .first()
        .map(|c| {
            c.competitors
                .iter()
                .map(|c| (c.id.clone(), c.team.abbreviation.clone()))
                .collect()
        })
        .unwrap_or_default();

    let goals: Vec<GoalDetail> = resp
        .key_events
        .iter()
        .filter(|e| e.type_.text.starts_with("Goal"))
        .map(|e| {
            let raw = e.short_text.as_deref().unwrap_or("Goal");
            let scorer = match raw.find(" Goal") {
                Some(pos) => raw[..pos].to_string(),
                None => raw.to_string(),
            };
            let team_code = e
                .team
                .as_ref()
                .and_then(|t| team_map.get(&t.id))
                .cloned()
                .unwrap_or_default();
            GoalDetail {
                minute: e.clock.display_value.clone(),
                scorer,
                team_code,
            }
        })
        .collect();

    Ok(goals)
}

pub async fn fetch_events(days: Option<i64>) -> Result<Vec<MatchInfo>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut all_events = Vec::new();

    let now = Utc::now();
    let start_date = now.date_naive();

    let start_offset: i64 = if days.is_none() { -1 } else { 0 };
    let num_days = days.unwrap_or(2);

    for i in 0..num_days {
        let target_date = start_date + chrono::Duration::days(i + start_offset);
        let date_str = target_date.format("%Y%m%d").to_string();

        let url = format!(
            "https://site.api.espn.com/apis/site/v2/sports/soccer/fifa.world/scoreboard?dates={}",
            date_str
        );

        let resp: ScoreboardResponse = client.get(&url).send().await?.json().await?;

        for event in resp.events {
            if let Some(competition) = event.competitions.first() {
                // ESPN uses format like "2026-06-14T04:00Z" (no seconds)
                let event_date = event.date.trim_end_matches('Z');
                let naive = if event_date.len() == 16 {
                    NaiveDateTime::parse_from_str(event_date, "%Y-%m-%dT%H:%M")
                } else {
                    NaiveDateTime::parse_from_str(event_date, "%Y-%m-%dT%H:%M:%S")
                }
                .map_err(|e| format!("failed to parse date '{}': {}", event.date, e))?;
                let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

                let (team_a, team_b, code_a, code_b, score_a, score_b) = if competition.competitors.len() >= 2 {
                    (
                        competition.competitors[0].team.short_display_name.clone(),
                        competition.competitors[1].team.short_display_name.clone(),
                        competition.competitors[0].team.abbreviation.clone(),
                        competition.competitors[1].team.abbreviation.clone(),
                        competition.competitors[0].score.clone(),
                        competition.competitors[1].score.clone(),
                    )
                } else {
                    continue;
                };

                let venue = competition
                    .venue
                    .as_ref()
                    .and_then(|v| v.full_name.clone())
                    .unwrap_or_else(|| "TBD".to_string());

                all_events.push(MatchInfo {
                    datetime,
                    team_a,
                    team_b,
                    code_a,
                    code_b,
                    score_a,
                    score_b,
                    status_state: competition.status.type_.state.clone(),
                    display_clock: competition.status.displayclock.clone(),
                    venue,
                    event_id: event.id.clone(),
                    goals: Vec::new(),
                });
            }
        }
    }

    all_events.sort_by_key(|e| e.datetime);

    for event in &mut all_events {
        if event.status_state == "in" {
            if let Ok(goals) = fetch_summary(&event.event_id).await {
                event.goals = goals;
            }
        }
    }

    Ok(all_events)
}
