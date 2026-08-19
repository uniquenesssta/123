use super::{
    normalization::{compact_query, COMPACT_SQL_PATTERN, LATIN_FOLD_SOURCE, LATIN_FOLD_TARGET},
    query::NameSearch,
};
use sqlx::{Postgres, QueryBuilder};

#[derive(Debug, Clone, Copy)]
pub(crate) struct NameSearchColumns<'a> {
    pub primary_normalized: &'a str,
    pub primary_display: &'a str,
    pub alias_table: &'a str,
    pub alias_owner: &'a str,
    pub owner_id: &'a str,
    pub alias_normalized: &'a str,
    pub alias_display: &'a str,
}

pub(crate) fn push_name_search(
    builder: &mut QueryBuilder<'_, Postgres>,
    search: &NameSearch,
    columns: NameSearchColumns<'_>,
) {
    for token in search.tokens() {
        let contains = format!("%{token}%");
        let compact = compact_query(token);
        let compact_contains = format!("%{compact}%");

        builder.push(" AND (");
        push_text_match(
            builder,
            columns.primary_normalized,
            columns.primary_display,
            &contains,
            &compact_contains,
        );
        builder.push(" OR EXISTS (SELECT 1 FROM ");
        builder.push(columns.alias_table);
        builder.push(" alias WHERE ");
        builder.push(columns.alias_owner);
        builder.push(" = ");
        builder.push(columns.owner_id);
        builder.push(" AND (");
        push_text_match(
            builder,
            columns.alias_normalized,
            columns.alias_display,
            &contains,
            &compact_contains,
        );
        builder.push(")))");
    }
}

fn push_text_match(
    builder: &mut QueryBuilder<'_, Postgres>,
    normalized_column: &str,
    display_column: &str,
    contains: &str,
    compact_contains: &str,
) {
    push_folded_sql_expression(builder, normalized_column, false);
    builder.push(" LIKE ");
    builder.push_bind(contains.to_string());
    builder.push(" OR ");
    push_folded_sql_expression(builder, display_column, true);
    builder.push(" LIKE ");
    builder.push_bind(contains.to_string());
    builder.push(" OR regexp_replace(");
    push_folded_sql_expression(builder, normalized_column, false);
    builder.push(", '");
    builder.push(COMPACT_SQL_PATTERN);
    builder.push("', '', 'g') LIKE ");
    builder.push_bind(compact_contains.to_string());
    builder.push(" OR regexp_replace(");
    push_folded_sql_expression(builder, display_column, true);
    builder.push(", '");
    builder.push(COMPACT_SQL_PATTERN);
    builder.push("', '', 'g') LIKE ");
    builder.push_bind(compact_contains.to_string());
}

fn push_folded_sql_expression(
    builder: &mut QueryBuilder<'_, Postgres>,
    column: &str,
    needs_lowercase: bool,
) {
    builder.push("translate(");
    if needs_lowercase {
        builder.push("lower(");
        builder.push(column);
        builder.push(")");
    } else {
        builder.push(column);
    }
    builder.push(", '");
    builder.push(LATIN_FOLD_SOURCE);
    builder.push("', '");
    builder.push(LATIN_FOLD_TARGET);
    builder.push("')");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_sql_checks_primary_and_alias_names() {
        let mut builder = QueryBuilder::<Postgres>::new("SELECT 1 WHERE 1=1");
        let search = NameSearch::parse(Some("索萨")).unwrap();
        push_name_search(
            &mut builder,
            &search,
            NameSearchColumns {
                primary_normalized: "player.normalized_name",
                primary_display: "player.canonical_name",
                alias_table: "football.player_names",
                alias_owner: "alias.player_id",
                owner_id: "player.id",
                alias_normalized: "alias.normalized_name",
                alias_display: "alias.name",
            },
        );
        let sql = builder.sql();
        assert!(sql.contains("translate(player.normalized_name"));
        assert!(sql.contains("translate(lower(player.canonical_name)"));
        assert!(sql.contains("football.player_names alias"));
        assert!(sql.contains("translate(alias.normalized_name"));
        assert!(sql.contains("translate(lower(alias.name)"));
        assert!(sql.contains("regexp_replace"));
    }
}
