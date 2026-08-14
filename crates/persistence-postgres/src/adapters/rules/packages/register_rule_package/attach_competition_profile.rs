use crate::PersistenceResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn attach_competition_profile(
    tx: &mut Transaction<'_, Postgres>,
    package_id: Uuid,
    competition_profile_id: Uuid,
) -> PersistenceResult<()> {
    sqlx::query(
        "UPDATE model.rule_packages SET competition_profile_id = $2 WHERE id = $1 AND competition_profile_id IS NULL",
    )
    .bind(package_id)
    .bind(competition_profile_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
