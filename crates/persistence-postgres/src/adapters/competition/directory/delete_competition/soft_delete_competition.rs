use crate::PersistenceResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn soft_delete_competition(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
        UPDATE football.competitions
        SET is_active = false,
            code = code || '-DELETED-' || left(id::text, 8),
            metadata = metadata || jsonb_build_object(
                'deleted_at', now(),
                'original_code', code
            ),
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
