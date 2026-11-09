use super::invalid;
use crate::{sha256_json, PersistenceResult};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const SEASON: &str = "2026/27";
pub(super) const COMPETITION: &str = "Premier League";
const FIXTURES_URL: &str = "https://fantasy.premierleague.com/api/fixtures/";

#[derive(Deserialize)]
pub(super) struct Fixture {
    pub fixture_id: u32,
    pub matchweek: u8,
    pub kickoff_utc: DateTime<Utc>,
    pub home_club_id: u32,
    pub home: String,
    pub away_club_id: u32,
    pub away: String,
}

#[derive(Deserialize)]
pub(super) struct HistoricalResult {
    #[serde(flatten)]
    pub fixture: Fixture,
    pub home_goals_90: i16,
    pub away_goals_90: i16,
    pub feed_finished: bool,
    pub feed_finished_provisional: bool,
    pub corroboration: Option<Corroboration>,
}

#[derive(Deserialize)]
pub(super) struct Corroboration {
    pub status: String,
    pub url: String,
    pub verified_at: DateTime<Utc>,
}

impl HistoricalResult {
    pub fn known_at(&self, feed_observed_at: DateTime<Utc>) -> DateTime<Utc> {
        self.corroboration
            .as_ref()
            .map_or(feed_observed_at, |report| {
                feed_observed_at.max(report.verified_at)
            })
    }
}

#[derive(Deserialize)]
pub(super) struct FixtureSource {
    pub url: String,
    pub sha256: String,
    pub observed_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct Sources {
    fixtures: FixtureSource,
}

#[derive(Deserialize)]
struct Body {
    schema: String,
    status: String,
    competition: String,
    season: String,
    verified_at: DateTime<Utc>,
    sources: Sources,
    history: Vec<HistoricalResult>,
    scheduled: Vec<Fixture>,
}

#[derive(Deserialize)]
struct SealedPacket {
    body: Value,
    sha256: String,
}

pub(super) struct Packet {
    pub digest: String,
    pub source: FixtureSource,
    pub verified_at: DateTime<Utc>,
    pub clubs: BTreeMap<u32, String>,
    pub history: Vec<HistoricalResult>,
    pub scheduled: Vec<Fixture>,
}

impl Packet {
    pub fn open(bytes: &[u8]) -> PersistenceResult<Self> {
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(invalid("赛果制品超过 4 MiB"));
        }
        let sealed: SealedPacket = serde_json::from_slice(bytes)?;
        if sha256_json(&sealed.body)? != sealed.sha256 {
            return Err(invalid("赛果制品摘要不匹配"));
        }
        let body: Body = serde_json::from_value(sealed.body)?;
        let source = body.sources.fixtures;
        if body.schema != "football.initial-calibration.source-preparation.v1"
            || body.status != "SOURCES_VERIFIED_AWAITING_IMPORT_AND_PREMATCH_WINDOW"
            || body.competition != COMPETITION
            || body.season != SEASON
            || source.url != FIXTURES_URL
            || source.sha256.len() != 64
            || !source.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            || source.observed_at > body.verified_at
            || body.verified_at > Utc::now()
        {
            return Err(invalid("赛果制品的来源、范围或核对时间无效"));
        }
        let mut clubs = BTreeMap::new();
        let mut fixture_ids = BTreeSet::new();
        let mut fixtures_by_pair = BTreeSet::new();
        let mut week_clubs = BTreeSet::new();
        for fixture in body
            .history
            .iter()
            .map(|row| &row.fixture)
            .chain(&body.scheduled)
        {
            if fixture.fixture_id == 0
                || !(1..=38).contains(&fixture.matchweek)
                || !fixture_ids.insert(fixture.fixture_id)
                || !fixtures_by_pair.insert((fixture.home_club_id, fixture.away_club_id))
            {
                return Err(invalid("赛事轮次或比赛来源身份无效或重复"));
            }
            for (id, name) in [
                (fixture.home_club_id, &fixture.home),
                (fixture.away_club_id, &fixture.away),
            ] {
                if !(1..=20).contains(&id)
                    || name.trim().is_empty()
                    || name.trim() != name
                    || !week_clubs.insert((fixture.matchweek, id))
                    || clubs
                        .insert(id, name.clone())
                        .is_some_and(|previous| previous != *name)
                {
                    return Err(invalid("球队身份冲突或同一轮出现多次"));
                }
            }
        }
        if fixture_ids.is_empty() || clubs.values().collect::<BTreeSet<_>>().len() != clubs.len() {
            return Err(invalid("赛果制品没有比赛或包含重复的球队名称"));
        }
        for row in &body.history {
            let report_ok = row.corroboration.as_ref().is_some_and(|report| {
                report.status == "FT"
                    && report.url.starts_with("https://")
                    && report.verified_at >= row.fixture.kickoff_utc
                    && report.verified_at <= body.verified_at
            });
            if row.home_goals_90 < 0
                || row.away_goals_90 < 0
                || row.fixture.kickoff_utc >= source.observed_at
                || !(row.feed_finished || (row.feed_finished_provisional && report_ok))
                || (row.corroboration.is_some() && !report_ok)
            {
                return Err(invalid(format!(
                    "比赛 {} 缺少合格的完场赛果",
                    row.fixture.fixture_id
                )));
            }
        }
        if body
            .scheduled
            .iter()
            .any(|fixture| fixture.kickoff_utc <= source.observed_at)
        {
            return Err(invalid("未来赛程早于数据来源的观察时间"));
        }
        Ok(Self {
            digest: sealed.sha256,
            source,
            verified_at: body.verified_at,
            clubs,
            history: body.history,
            scheduled: body.scheduled,
        })
    }
}
