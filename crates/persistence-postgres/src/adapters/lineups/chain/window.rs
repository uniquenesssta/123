use crate::{PersistenceError, PersistenceResult};
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, Copy)]
pub(crate) struct LineupSnapshotWindow {
    pub start_time: Option<DateTime<Utc>>,
    pub cutoff_time: DateTime<Utc>,
}

pub(crate) fn normalize_lineup_snapshot_type(value: &str) -> PersistenceResult<&'static str> {
    match value.trim() {
        "T-N" => Ok("T-N"),
        "T-24h" => Ok("T-24h"),
        "T-6h" => Ok("T-6h"),
        "T-1h" => Ok("T-1h"),
        "T-90m" => Err(PersistenceError::InvalidState(
            "T-90m 已停止用于新阵容和新推演；请选择 T-N、T-24h、T-6h 或 T-1h".to_string(),
        )),
        other => Err(PersistenceError::InvalidState(format!(
            "不支持的阵容时间窗口：{other}"
        ))),
    }
}

pub(crate) fn lineup_snapshot_window(
    kickoff_time: DateTime<Utc>,
    snapshot_type: &str,
) -> PersistenceResult<LineupSnapshotWindow> {
    lineup_snapshot_window_at(kickoff_time, snapshot_type, Utc::now())
}

pub(crate) fn lineup_snapshot_window_at(
    kickoff_time: DateTime<Utc>,
    snapshot_type: &str,
    reference_time: DateTime<Utc>,
) -> PersistenceResult<LineupSnapshotWindow> {
    let snapshot_type = normalize_lineup_snapshot_type(snapshot_type)?;
    let cutoff_time = reference_time.min(kickoff_time - Duration::seconds(1));
    let start_time = match snapshot_type {
        "T-N" => None,
        "T-24h" => Some(kickoff_time - Duration::hours(24)),
        "T-6h" => Some(kickoff_time - Duration::hours(6)),
        "T-1h" => Some(kickoff_time - Duration::hours(1)),
        _ => unreachable!(),
    };
    if let Some(start_time) = start_time {
        if cutoff_time < start_time {
            return Err(PersistenceError::InvalidState(format!(
                "{snapshot_type} 数据窗口尚未开启；窗口从 {} 开始",
                start_time.to_rfc3339()
            )));
        }
    }
    Ok(LineupSnapshotWindow {
        start_time,
        cutoff_time,
    })
}

#[cfg(test)]
mod tests {
    use super::lineup_snapshot_window_at;
    use chrono::{Duration, TimeZone, Utc};

    #[test]
    fn latest_window_uses_reference_time_before_kickoff() {
        let kickoff = Utc
            .with_ymd_and_hms(2026, 7, 20, 12, 0, 0)
            .single()
            .unwrap();
        let reference = kickoff - Duration::hours(2);
        let window = lineup_snapshot_window_at(kickoff, "T-N", reference).unwrap();
        assert_eq!(window.start_time, None);
        assert_eq!(window.cutoff_time, reference);
    }

    #[test]
    fn fixed_window_means_within_declared_duration() {
        let kickoff = Utc
            .with_ymd_and_hms(2026, 7, 20, 12, 0, 0)
            .single()
            .unwrap();
        let reference = kickoff - Duration::hours(2);
        let window = lineup_snapshot_window_at(kickoff, "T-6h", reference).unwrap();
        assert_eq!(window.start_time, Some(kickoff - Duration::hours(6)));
        assert_eq!(window.cutoff_time, reference);
    }

    #[test]
    fn fixed_window_rejects_requests_before_window_opens() {
        let kickoff = Utc
            .with_ymd_and_hms(2026, 7, 20, 12, 0, 0)
            .single()
            .unwrap();
        let reference = kickoff - Duration::hours(7);
        assert!(lineup_snapshot_window_at(kickoff, "T-6h", reference).is_err());
    }

    #[test]
    fn latest_window_never_crosses_kickoff() {
        let kickoff = Utc
            .with_ymd_and_hms(2026, 7, 20, 12, 0, 0)
            .single()
            .unwrap();
        let reference = kickoff + Duration::hours(2);
        let window = lineup_snapshot_window_at(kickoff, "T-N", reference).unwrap();
        assert_eq!(window.cutoff_time, kickoff - Duration::seconds(1));
    }

    #[test]
    fn legacy_t90m_is_not_available_for_new_requests() {
        let kickoff = Utc
            .with_ymd_and_hms(2026, 7, 20, 12, 0, 0)
            .single()
            .unwrap();
        assert!(lineup_snapshot_window_at(kickoff, "T-90m", kickoff).is_err());
    }
    #[test]
    fn fixed_window_accepts_its_exact_start_and_rejects_one_nanosecond_before() {
        let kickoff = Utc
            .with_ymd_and_hms(2026, 7, 20, 12, 0, 0)
            .single()
            .unwrap();
        for (kind, hours) in [("T-24h", 24), ("T-6h", 6), ("T-1h", 1)] {
            let start = kickoff - Duration::hours(hours);
            let window = lineup_snapshot_window_at(kickoff, kind, start).unwrap();
            assert_eq!(window.start_time, Some(start));
            assert_eq!(window.cutoff_time, start);
            assert!(
                lineup_snapshot_window_at(kickoff, kind, start - Duration::nanoseconds(1)).is_err()
            );
        }
    }

    #[test]
    fn every_formal_window_stops_one_second_before_kickoff() {
        let kickoff = Utc
            .with_ymd_and_hms(2026, 7, 20, 12, 0, 0)
            .single()
            .unwrap();
        for kind in ["T-N", "T-24h", "T-6h", "T-1h"] {
            let window = lineup_snapshot_window_at(kickoff, kind, kickoff).unwrap();
            assert_eq!(window.cutoff_time, kickoff - Duration::seconds(1));
        }
    }

    #[test]
    fn snapshot_normalization_preserves_formal_names_and_rejects_unknown_names() {
        for kind in ["T-N", "T-24h", "T-6h", "T-1h"] {
            assert_eq!(
                super::normalize_lineup_snapshot_type(&format!(" {kind} ")).unwrap(),
                kind
            );
        }
        for kind in ["", "T-90m", "T-2h", "t-6h"] {
            assert!(super::normalize_lineup_snapshot_type(kind).is_err());
        }
    }
}
