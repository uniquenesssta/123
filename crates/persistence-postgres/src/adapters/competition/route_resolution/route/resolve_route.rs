use super::binding_candidate::read_binding_candidate;
use super::explicit_rule_package::read_explicit_rule_package;
use super::record_mapper::{binding_source, map_route_row};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{RouteDecision, RouteRequest, RouteSource};

impl PostgresStore {
    pub async fn resolve_route(&self, request: &RouteRequest) -> PersistenceResult<RouteDecision> {
        if let Some(package_id) = request.explicit_rule_package_id {
            let row = read_explicit_rule_package(
                &self.pool,
                package_id,
                request.preferred_model_family.as_deref(),
                request.preferred_model_id.as_deref(),
            )
            .await?;
            return map_route_row(row, RouteSource::ExplicitRulePackage, request);
        }

        let row = read_binding_candidate(&self.pool, request).await?;
        let source = binding_source(&row);
        map_route_row(row, source, request)
    }
}
