use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum P4Horizon {
    #[serde(rename = "T-24h")]
    T24h,
    #[serde(rename = "T-6h")]
    T6h,
    #[serde(rename = "T-90m")]
    T90m,
    #[serde(rename = "T-1h")]
    T1h,
    #[serde(rename = "T-N")]
    LegacyTN,
}

impl P4Horizon {
    pub const CANONICAL: [Self; 3] = [Self::T24h, Self::T6h, Self::T1h];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::T24h => "T-24h",
            Self::T6h => "T-6h",
            Self::T90m => "T-90m",
            Self::T1h => "T-1h",
            Self::LegacyTN => "T-N",
        }
    }

    pub const fn is_canonical(self) -> bool {
        matches!(self, Self::T24h | Self::T6h | Self::T1h)
    }
}

impl P4Horizon {
    pub const fn offset_minutes(self) -> Option<i64> {
        match self {
            Self::T24h => Some(24 * 60),
            Self::T6h => Some(6 * 60),
            Self::T90m => Some(90),
            Self::T1h => Some(60),
            Self::LegacyTN => None,
        }
    }

    pub fn data_cutoff_at(self, kickoff_at: DateTime<Utc>) -> Option<DateTime<Utc>> {
        self.offset_minutes()
            .map(|minutes| kickoff_at - Duration::minutes(minutes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn canonical_horizon_cutoffs_are_exact() {
        let kickoff = Utc
            .with_ymd_and_hms(2026, 7, 20, 12, 0, 0)
            .single()
            .expect("valid kickoff");
        assert_eq!(
            P4Horizon::T24h.data_cutoff_at(kickoff),
            Some(kickoff - Duration::hours(24))
        );
        assert_eq!(
            P4Horizon::T6h.data_cutoff_at(kickoff),
            Some(kickoff - Duration::hours(6))
        );
        assert_eq!(
            P4Horizon::T90m.data_cutoff_at(kickoff),
            Some(kickoff - Duration::minutes(90))
        );
        assert_eq!(
            P4Horizon::T1h.data_cutoff_at(kickoff),
            Some(kickoff - Duration::hours(1))
        );
        assert_eq!(P4Horizon::LegacyTN.data_cutoff_at(kickoff), None);
    }
}
