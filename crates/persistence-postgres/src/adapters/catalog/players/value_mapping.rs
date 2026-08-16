use crate::{PersistenceError, PersistenceResult};
use football_domain::{AvailabilityStatus, PlayerStatus, PreferredFoot};

pub(crate) fn preferred_foot(value: &str) -> PersistenceResult<PreferredFoot> {
    match value {
        "left" => Ok(PreferredFoot::Left),
        "right" => Ok(PreferredFoot::Right),
        "both" => Ok(PreferredFoot::Both),
        "unknown" => Ok(PreferredFoot::Unknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知惯用脚类型：{other}"
        ))),
    }
}

pub(crate) fn player_status(value: &str) -> PersistenceResult<PlayerStatus> {
    match value {
        "active" => Ok(PlayerStatus::Active),
        "inactive" => Ok(PlayerStatus::Inactive),
        "retired" => Ok(PlayerStatus::Retired),
        "unknown" => Ok(PlayerStatus::Unknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知球员状态：{other}"
        ))),
    }
}

pub(crate) fn availability_status(value: &str) -> PersistenceResult<AvailabilityStatus> {
    match value {
        "available" => Ok(AvailabilityStatus::Available),
        "doubtful" => Ok(AvailabilityStatus::Doubtful),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        "injured" => Ok(AvailabilityStatus::Injured),
        "suspended" => Ok(AvailabilityStatus::Suspended),
        "rested" => Ok(AvailabilityStatus::Rested),
        "returning" => Ok(AvailabilityStatus::Returning),
        "unknown" => Ok(AvailabilityStatus::Unknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知球员可用性：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{availability_status, player_status, preferred_foot};
    use football_domain::{AvailabilityStatus, PlayerStatus, PreferredFoot};

    #[test]
    fn persisted_values_round_trip() {
        assert_eq!(preferred_foot("left").unwrap(), PreferredFoot::Left);
        assert_eq!(player_status("active").unwrap(), PlayerStatus::Active);
        assert_eq!(
            availability_status("returning").unwrap(),
            AvailabilityStatus::Returning
        );
    }

    #[test]
    fn unknown_values_are_rejected() {
        assert!(preferred_foot("ambidextrous").is_err());
        assert!(availability_status("missing").is_err());
    }
}
