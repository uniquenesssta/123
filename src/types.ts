export type Theme = "light" | "dark";

export type CompetitionKind =
  | "league"
  | "group_stage"
  | "knockout_single_leg"
  | "knockout_two_leg"
  | "friendly"
  | "custom";

export interface DatabaseOptions {
  connection_url: string;
  max_connections: number;
  connect_timeout_seconds: number;
}

export interface DatabaseHealth {
  connected: boolean;
  database_name: string;
  server_version: string;
  migration_count: number;
  database_size_bytes: number;
  checked_at: string;
  latency_ms: number;
}

export interface DatabaseStats {
  competitions: number;
  teams: number;
  players: number;
  matches: number;
  model_runs: number;
  rule_packages: number;
  route_bindings: number;
  ability_observations: number;
  pending_ability_updates: number;
  data_providers: number;
  availability_records: number;
  active_lineups: number;
  large_counts_are_estimates: boolean;
}

export interface ModelDescriptor {
  model_id: string;
  display_name: string;
  engine_version: string;
  supported_competitions: CompetitionKind[];
  input_schema_version: string;
  output_schema_version: string;
}

export interface CompetitionRecord {
  id: string;
  code: string;
  name: string;
  country_code: string | null;
  timezone: string;
  competition_kind: CompetitionKind;
  is_active: boolean;
  metadata: Record<string, unknown>;
  created_at: string;
}

export interface CompetitionDraft {
  code: string;
  name: string;
  country_code: string | null;
  timezone: string;
  competition_kind: CompetitionKind;
  metadata: Record<string, unknown>;
}

export interface SeasonDraft {
  competition_id: string;
  name: string;
  starts_on: string | null;
  ends_on: string | null;
  status: "planned" | "active" | "completed" | "archived";
  metadata: Record<string, unknown>;
}

export interface SeasonRecord {
  id: string;
  competition_id: string;
  competition_name: string;
  name: string;
  starts_on: string | null;
  ends_on: string | null;
  status: string;
}

export interface StageDraft {
  season_id: string;
  code: string;
  name: string;
  stage_kind: CompetitionKind;
  sequence_no: number;
  rules: Record<string, unknown>;
}

export interface StageRecord {
  id: string;
  season_id: string;
  season_name: string;
  competition_id: string;
  competition_name: string;
  code: string;
  name: string;
  stage_kind: CompetitionKind;
  sequence_no: number;
}

export interface RoundDraft {
  stage_id: string;
  code: string;
  name: string;
  sequence_no: number;
  starts_at: string | null;
  ends_at: string | null;
}

export interface RoundRecord {
  id: string;
  stage_id: string;
  stage_name: string;
  code: string;
  name: string;
  sequence_no: number;
  starts_at: string | null;
  ends_at: string | null;
}

export interface CompetitionProfile {
  profile_id: string;
  name: string;
  competition_kind: CompetitionKind;
  normal_time_minutes: number;
  extra_time_possible: boolean;
  penalties_possible: boolean;
  two_legged: boolean;
  neutral_venue: boolean;
  metadata: Record<string, unknown>;
}

export interface RuleRouting {
  model_id: string;
  model_version: string;
  parameter_version: string;
  priority: number;
  activate_as_type_default: boolean;
  supported_snapshot_types: string[];
}

export interface RulePackageDraft {
  format_version: string;
  package_key: string;
  version: string;
  display_name: string;
  competition_profile: CompetitionProfile;
  routing: RuleRouting;
  parameters: Record<string, unknown>;
  feature_requirements: Record<string, unknown>;
  output_contract: Record<string, unknown>;
  source_document: Record<string, unknown> | null;
  metadata: Record<string, unknown>;
}

export interface RulePackageSummary {
  id: string;
  format_version: string;
  package_key: string;
  version: string;
  display_name: string;
  competition_kind: CompetitionKind;
  model_id: string;
  model_version: string;
  parameter_version: string;
  priority: number;
  content_sha256: string;
  status: string;
  created_at: string;
}

export interface CompetitionBindingDraft {
  binding_name: string | null;
  competition_id: string | null;
  season_id: string | null;
  stage_id: string | null;
  competition_kind: CompetitionKind | null;
  rule_package_id: string;
  priority: number;
  valid_from: string | null;
  valid_to: string | null;
}

export interface CompetitionBindingSummary {
  id: string;
  binding_name: string;
  competition_id: string | null;
  competition_name: string | null;
  season_id: string | null;
  stage_id: string | null;
  competition_kind: CompetitionKind | null;
  rule_package_id: string;
  rule_package_name: string;
  model_id: string;
  priority: number;
  is_active: boolean;
  created_at: string;
}

export type RouteSource =
  | "explicit_rule_package"
  | "stage_binding"
  | "season_binding"
  | "competition_binding"
  | "competition_kind_default";

export interface RouteDecision {
  source: RouteSource;
  binding_id: string | null;
  rule_package_id: string;
  package_key: string;
  package_version: string;
  package_display_name: string;
  model_id: string;
  model_version_id: string;
  model_version: string;
  parameter_set_id: string;
  parameter_version: string;
  competition_profile_id: string;
  parameters: Record<string, unknown>;
  routing: RuleRouting;
  competition_profile: CompetitionProfile;
  feature_requirements: Record<string, unknown>;
  output_contract: Record<string, unknown>;
  priority: number;
  reason: Record<string, unknown>;
}

export interface ModelRunListItem {
  id: string;
  match_key: string;
  competition_name: string | null;
  home_team_name: string | null;
  away_team_name: string | null;
  kickoff_time: string | null;
  snapshot_type: string;
  model_key: string;
  model_version: string;
  parameter_version: string;
  rule_package_name: string | null;
  summary: {
    home_win?: number;
    draw?: number;
    away_win?: number;
    btts?: number | null;
    over_2_5?: number | null;
  };
  top_scoreline: string | null;
  top_scoreline_probability: number | null;
  created_at: string;
  completed_at: string | null;
  duration_ms: number | null;
  input_readiness_level: PredictionReadinessLevel | "not_assessed" | "legacy_unknown";
  input_readiness_score: number | null;
  input_manifest_sha256: string;
}

export interface BootstrapData {
  app_version: string;
  database_configured: boolean;
  database_url: string | null;
  database_health: DatabaseHealth | null;
  stats: DatabaseStats | null;
  models: ModelDescriptor[];
  competitions: CompetitionRecord[];
  seasons: SeasonRecord[];
  stages: StageRecord[];
  rounds: RoundRecord[];
  rule_packages: RulePackageSummary[];
  competition_bindings: CompetitionBindingSummary[];
  recent_runs: ModelRunListItem[];
  default_match: Record<string, unknown>;
  default_rule_package: RulePackageDraft;
}

export interface BootstrapResponse {
  data: BootstrapData;
  connection_error: string | null;
  config_path: string;
  runtime_log_path: string;
}

export type PredictionModelFamily = string;

export interface PredictionCommand {
  match_input: Record<string, unknown>;
  snapshot_type: string;
  competition_id: string | null;
  season_id: string | null;
  stage_id: string | null;
  competition_kind: CompetitionKind;
  explicit_rule_package_id: string | null;
  model_family: PredictionModelFamily;
}

export interface StoredMatchPredictionCommand {
  match_id: string;
  snapshot_type: string;
  explicit_rule_package_id: string | null;
  model_family: PredictionModelFamily;
}

export interface RoutePreviewCommand {
  kickoff_time: string;
  competition_id: string | null;
  season_id: string | null;
  stage_id: string | null;
  competition_kind: CompetitionKind;
  explicit_rule_package_id: string | null;
  model_family: PredictionModelFamily;
}

export type PredictionReadinessLevel =
  | "formal_ready"
  | "ready_with_warnings"
  | "shadow_only"
  | "blocked";

export type PredictionReadinessCheckStatus = "passed" | "warning" | "blocked";

export interface PredictionReadinessCheck {
  code: string;
  label: string;
  status: PredictionReadinessCheckStatus;
  weight: number;
  score: number;
  summary: string;
  details: string[];
  metadata: unknown;
}

export interface MatchPredictionReadiness {
  audit_version: string;
  match_id: string;
  match_key: string;
  snapshot_type: string;
  model_family: string;
  assessed_at: string;
  data_cutoff_at: string | null;
  level: PredictionReadinessLevel;
  score: number;
  can_run_formal: boolean;
  can_run_shadow: boolean;
  blockers: string[];
  warnings: string[];
  checks: PredictionReadinessCheck[];
  input_manifest: Record<string, unknown> | null;
  input_manifest_sha256: string | null;
  route_identity: Record<string, unknown> | null;
}

export interface PredictionInputAuditSummary {
  audit_version: string;
  readiness_level: string;
  readiness_score: number | null;
  input_manifest_sha256: string;
  input_sha256: string;
}

export interface PredictionExecution {
  run_id: string;
  duration_ms: number;
  route: RouteDecision;
  input_audit: PredictionInputAuditSummary | null;
  output: {
    identity: {
      model_id: string;
      model_version: string;
      parameter_version: string;
      rule_package_version: string | null;
    };
    summary: {
      home_win: number;
      draw: number;
      away_win: number;
      btts: number | null;
      over_2_5: number | null;
    };
    payload: Record<string, unknown>;
    explanation: Record<string, unknown>;
  };
}

export type IssueSeverity = "warning" | "error" | "critical";

export interface IssueLogDraft {
  severity: IssueSeverity;
  source: string;
  operation: string;
  user_message: string;
  technical_message: string;
  occurrence_key?: string | null;
}

export interface IssueLogEntry {
  id: string;
  severity: IssueSeverity;
  source: string;
  operations: string[];
  user_message: string;
  technical_message: string;
  occurrence_count: number;
  first_seen_at: string;
  last_seen_at: string;
  app_version: string;
}

export type Page =
  | "dashboard"
  | "database"
  | "openai"
  | "api_workspace"
  | "workbooks"
  | "lineup_presets"
  | "rules"
  | "players"
  | "teams"
  | "lineups"
  | "prediction"
  | "review"
  | "analytics"
  | "runs"
  | "logs"
  | "architecture"
  | "release";

export type PreferredFoot = "left" | "right" | "both" | "unknown";
export type PlayerStatus = "active" | "inactive" | "retired" | "unknown";
export type AvailabilityStatus =
  | "available"
  | "doubtful"
  | "unavailable"
  | "injured"
  | "suspended"
  | "rested"
  | "returning"
  | "unknown";
export type MatchStatus =
  "scheduled" | "live" | "finished" | "postponed" | "cancelled";
export type LineupType = "expected" | "confirmed" | "actual";

export interface TeamDraft {
  canonical_name: string;
  country_code: string | null;
  metadata: Record<string, unknown>;
}

export interface TeamRecord {
  id: string;
  canonical_name: string;
  normalized_name: string;
  country_code: string | null;
  is_active: boolean;
  created_at: string;
}

export type TeamType = "club" | "national" | "reserve" | "youth" | "women" | "other";

export interface TeamOption {
  id: string;
  canonical_name: string;
  country_code: string | null;
  team_type: TeamType;
}

export interface SeasonTeamMembershipOption {
  season_id: string;
  team_id: string;
  registration_status: "registered" | "withdrawn" | "suspended" | "guest";
}

export interface TeamListQuery {
  search: string | null;
  country_code: string | null;
  team_type: TeamType | null;
  active_only: boolean;
  limit: number;
  cursor_name: string | null;
  cursor_id: string | null;
}

export interface TeamListItem {
  id: string;
  canonical_name: string;
  normalized_name: string;
  country_code: string | null;
  team_type: "club" | "national" | "reserve" | "youth" | "women" | "other";
  current_coach_name: string | null;
  is_active: boolean;
  current_player_count: number;
  unavailable_player_count: number;
  squad_ability_average: number | null;
  profile_confidence: number | null;
}

export interface TeamListPage {
  items: TeamListItem[];
  next_cursor_name: string | null;
  next_cursor_id: string | null;
  has_more: boolean;
}

export interface TeamNameDraft {
  team_id: string;
  name: string;
  language_code: string | null;
  valid_from: string | null;
  valid_to: string | null;
}

export interface TeamNameRecord {
  id: string;
  team_id: string;
  name: string;
  normalized_name: string;
  language_code: string | null;
  valid_from: string | null;
  valid_to: string | null;
}

export interface TeamProfileDraft {
  short_name: string | null;
  team_type: "club" | "national" | "reserve" | "youth" | "women" | "other";
  founded_year: number | null;
  city: string | null;
  stadium: string | null;
  head_coach: string | null;
  default_formation: string | null;
  tactical_style:
    | "balanced"
    | "possession"
    | "direct"
    | "counter"
    | "pressing"
    | "defensive"
    | "custom";
  attack_rating: number | null;
  midfield_rating: number | null;
  defence_rating: number | null;
  goalkeeper_rating: number | null;
  reputation: number | null;
  data_confidence: number;
  notes: string | null;
  metadata: Record<string, unknown>;
}

export interface TeamProfileRecord extends TeamProfileDraft {
  team_id: string;
  updated_at: string;
}

export interface TeamSquadPlayer {
  player_id: string;
  player_name: string;
  localized_name: string | null;
  position_code: string | null;
  role_code: string | null;
  squad_number: number | null;
  registration_status: string;
  availability_status: AvailabilityStatus | null;
  ability_average: number | null;
}

export interface TeamRecentMatch {
  match_id: string;
  opponent_team_id: string;
  opponent_team_name: string;
  kickoff_time: string;
  venue_side: string;
  status: MatchStatus;
  goals_for: number | null;
  goals_against: number | null;
}


export interface FormationRecord {
  id: string;
  code: string;
  name: string;
  line_structure: string;
  slot_definition: unknown;
  is_builtin: boolean;
  is_active: boolean;
  sort_order: number;
  metadata: Record<string, unknown>;
}

export interface FormationUsageEntryDraft {
  formation_id: string;
  usage_count: number;
}

export interface FormationUsageDistributionDraft {
  scope_type: "team" | "coach" | "team_coach" | "competition_default" | "system_default";
  team_id: string | null;
  coach_id: string | null;
  competition_id: string | null;
  window_preset: "last_5" | "last_10" | "last_20" | "current_season" | "current_coach_term" | "custom";
  window_start: string | null;
  window_end: string | null;
  observed_matches: number;
  confidence: number;
  alpha: number;
  source_document_id: string | null;
  metadata: Record<string, unknown>;
  entries: FormationUsageEntryDraft[];
}

export interface FormationUsageEntryRecord {
  id: string;
  formation_id: string;
  formation_code: string;
  formation_name: string;
  usage_count: number;
  raw_probability: number;
  smoothed_probability: number;
}

export interface FormationUsageDistributionRecord {
  scope_type: string;
  team_id: string | null;
  team_name: string | null;
  coach_id: string | null;
  coach_name: string | null;
  competition_id: string | null;
  competition_name: string | null;
  window_preset: string;
  window_start: string;
  window_end: string;
  observed_matches: number;
  confidence: number;
  alpha: number;
  observed_at: string;
  entries: FormationUsageEntryRecord[];
}

export interface FormationUsageListQuery {
  team_id: string | null;
  coach_id: string | null;
  competition_id: string | null;
  limit: number;
}

export interface FormationDistributionQuery {
  team_id: string;
  coach_id: string | null;
  competition_id: string | null;
  match_id: string | null;
  as_of: string | null;
}

export interface ResolvedFormationDistribution {
  source_level: string;
  source_label: string;
  team_id: string;
  coach_id: string | null;
  competition_id: string | null;
  window_start: string | null;
  window_end: string | null;
  observed_matches: number;
  confidence: number;
  entries: FormationUsageEntryRecord[];
}

export interface TeamDetail {
  team: TeamRecord;
  names: TeamNameRecord[];
  profile: TeamProfileRecord | null;
  squad: TeamSquadPlayer[];
  player_periods: TeamPlayerPeriodRecord[];
  coach_periods: TeamCoachPeriodRecord[];
  formation_usage: FormationUsageDistributionRecord[];
  resolved_formation_distribution: ResolvedFormationDistribution;
  recent_matches: TeamRecentMatch[];
}



export interface CoachDraft {
  canonical_name: string;
  nationality_code: string | null;
  status: "active" | "inactive" | "retired" | "unknown";
  metadata: Record<string, unknown>;
}

export interface CoachRecord extends CoachDraft {
  id: string;
  normalized_name: string;
  created_at: string;
  updated_at: string;
}

export interface CoachListQuery {
  search: string | null;
  active_only: boolean;
  limit: number;
}

export interface CoachListItem {
  id: string;
  canonical_name: string;
  nationality_code: string | null;
  status: string;
  current_team_id: string | null;
  current_team_name: string | null;
  current_role: string | null;
}

export interface CoachNameDraft {
  coach_id: string;
  name: string;
  language_code: string | null;
  is_primary: boolean;
  valid_from: string | null;
  valid_to: string | null;
}

export interface CoachNameRecord extends CoachNameDraft {
  id: string;
  normalized_name: string;
}

export interface TeamCoachPeriodDraft {
  team_id: string;
  coach_id: string;
  role: "head_coach" | "assistant_coach" | "interim_head_coach" | "caretaker" | "other";
  valid_from: string;
  valid_to: string | null;
  is_interim: boolean;
  confidence: number;
  source_document_id: string | null;
  end_previous: boolean;
  metadata: Record<string, unknown>;
}

export interface TeamCoachPeriodRecord {
  id: string;
  team_id: string;
  team_name: string;
  coach_id: string;
  coach_name: string;
  role: string;
  valid_from: string;
  valid_to: string | null;
  is_interim: boolean;
  confidence: number;
}

export interface TeamPlayerPeriodRecord {
  id: string;
  team_id: string;
  team_name: string;
  player_id: string;
  player_name: string;
  season_id: string | null;
  season_name: string | null;
  squad_number: number | null;
  valid_from: string;
  valid_to: string | null;
  registration_status: string;
}

export interface CoachDetail {
  coach: CoachRecord;
  names: CoachNameRecord[];
  team_periods: TeamCoachPeriodRecord[];
  external_ids: Array<Record<string, unknown>>;
}

export type EntityReferenceType = "team" | "player" | "coach";

export interface EntityReferenceQuery {
  entity_type: EntityReferenceType;
  search: string | null;
  active_only: boolean;
  limit: number;
}

export interface EntityReferenceRecord {
  entity_type: EntityReferenceType;
  id: string;
  canonical_name: string;
  normalized_name: string;
  country_code: string | null;
  nationality_code: string | null;
  date_of_birth: string | null;
  status: string;
  aliases: string[];
  external_ids: string[];
}

export interface EntityMatchRequest {
  entity_type: EntityReferenceType;
  entity_id: string | null;
  provider_id: string | null;
  external_id: string | null;
  canonical_name: string | null;
  country_code: string | null;
  nationality_code: string | null;
  date_of_birth: string | null;
}

export interface EntityMatchCandidate {
  id: string;
  label: string;
  reason: string;
  score: number;
}

export interface EntityMatchResult {
  status: "exact" | "ambiguous" | "no_match";
  matched_id: string | null;
  candidates: EntityMatchCandidate[];
}

export interface EntityReferenceCount {
  relation: string;
  count: number;
}

export interface EntityDeletionCheck {
  entity_type: EntityReferenceType;
  entity_id: string;
  label: string;
  exists: boolean;
  can_permanently_delete: boolean;
  must_archive: boolean;
  references: EntityReferenceCount[];
  reason: string;
}

export interface BulkArchiveResult {
  entity_type: EntityReferenceType;
  requested_count: number;
  archived_ids: string[];
  already_archived_ids: string[];
  failed: BulkDeleteBlockedItem[];
}

export interface BulkDeleteBlockedItem {
  id: string;
  label: string;
  reason: string;
}

export interface BulkDeleteResult {
  requested_count: number;
  deleted_ids: string[];
  blocked: BulkDeleteBlockedItem[];
}

export interface TeamForceDeleteRequest {
  team_id: string;
  confirmation_text: string;
}

export interface TeamForceDeletePreview {
  team_id: string;
  label: string;
  confirmation_text: string;
  total_rows: number;
  references: EntityReferenceCount[];
  warning: string;
}

export interface TeamForceDeleteResult {
  team_id: string;
  label: string;
  deleted_match_ids: string[];
  deleted_player_ids: string[];
  deleted_coach_ids: string[];
  deleted_import_batch_ids: string[];
  deleted_counts: Record<string, number>;
}

export interface DataProviderDraft {
  code: string;
  name: string;
  provider_type: string;
  base_url: string | null;
  metadata: Record<string, unknown>;
}

export interface DataProviderRecord {
  id: string;
  code: string;
  name: string;
  provider_type: string;
  base_url: string | null;
  is_active: boolean;
}

export interface PlayerDraft {
  canonical_name: string;
  date_of_birth: string | null;
  nationality_code: string | null;
  preferred_foot: PreferredFoot;
  height_cm: number | null;
  status: PlayerStatus;
  metadata: Record<string, unknown>;
}

export interface PlayerRecord {
  id: string;
  canonical_name: string;
  normalized_name: string;
  date_of_birth: string | null;
  nationality_code: string | null;
  preferred_foot: PreferredFoot;
  height_cm: number | null;
  status: PlayerStatus;
  created_at: string;
}

export interface PlayerListQuery {
  search: string | null;
  team_id: string | null;
  position_code: string | null;
  availability_status: AvailabilityStatus | null;
  player_status: PlayerStatus | null;
  limit: number;
  cursor_name: string | null;
  cursor_id: string | null;
}

export interface PlayerNavigationContext {
  source: "team_roster" | "match_lineup";
  team_id: string;
  team_name: string;
  player_id: string | null;
  origin_page: "teams" | "lineups";
  return_section: "builder" | "chain" | null;
  created_at: string;
  updated_at: string;
}

export interface PlayerListItem {
  id: string;
  canonical_name: string;
  localized_name: string | null;
  alternate_name: string | null;
  normalized_name: string;
  date_of_birth: string | null;
  nationality_code: string | null;
  preferred_foot: PreferredFoot;
  status: PlayerStatus;
  current_team_id: string | null;
  current_team_name: string | null;
  primary_position_code: string | null;
  primary_role_code: string | null;
  position_role_map: Record<string, string>;
  availability_status: AvailabilityStatus | null;
  availability_reason: string | null;
  availability_confidence: number | null;
  availability_valid_to: string | null;
  availability_competition_name: string | null;
  ability_average: number | null;
  ability_confidence: number | null;
  ability_dimension_count: number;
}

export interface PlayerListPage {
  items: PlayerListItem[];
  next_cursor_name: string | null;
  next_cursor_id: string | null;
  has_more: boolean;
}

export interface PlayerNameDraft {
  player_id: string;
  name: string;
  language_code: string | null;
  is_primary: boolean;
  valid_from: string | null;
  valid_to: string | null;
}

export interface PlayerPositionDraft {
  player_id: string;
  position_code: string;
  proficiency: number;
  default_role_code: string | null;
  is_primary: boolean;
  valid_from: string | null;
  valid_to: string | null;
  source_document_id: string | null;
}

export interface PlayerTeamPeriodDraft {
  player_id: string;
  team_id: string;
  season_id: string | null;
  squad_number: number | null;
  valid_from: string;
  valid_to: string | null;
  registration_status: string;
  source_document_id: string | null;
}

export interface PlayerAvailabilityDraft {
  player_id: string;
  team_id: string | null;
  competition_id: string | null;
  status: AvailabilityStatus;
  reason: string | null;
  confidence: number;
  valid_from: string;
  valid_to: string | null;
  source_document_id: string | null;
  metadata: Record<string, unknown>;
}

export interface AbilityDimensionRecord {
  code: string;
  name: string;
  category: string;
  minimum_value: number;
  maximum_value: number;
  description: string | null;
}

export interface PlayerAbilityObservationDraft {
  player_id: string;
  dimension_code: string;
  context_type: string;
  context_id: string | null;
  value: number;
  confidence: number;
  sample_size: number;
  observed_at: string;
  effective_from: string;
  effective_to: string | null;
  calculation_version: string;
  source_document_id: string | null;
  metadata: Record<string, unknown>;
}

export interface ExternalEntityIdDraft {
  provider_id: string;
  entity_type: "competition" | "season" | "team" | "player" | "coach" | "match";
  entity_id: string;
  external_id: string;
  metadata: Record<string, unknown>;
}

export interface PlayerTeamPeriodRecord {
  id: string;
  player_id: string;
  team_id: string;
  team_name: string;
  season_id: string | null;
  season_name: string | null;
  squad_number: number | null;
  valid_from: string;
  valid_to: string | null;
  registration_status: string;
}

export interface PlayerDetail {
  player: PlayerRecord;
  names: Array<Record<string, unknown>>;
  positions: Array<Record<string, unknown>>;
  team_periods: PlayerTeamPeriodRecord[];
  availability: Array<Record<string, unknown>>;
  ability_profile: {
    player_id: string;
    abilities: Record<string, unknown>;
    average_value: number | null;
    average_confidence: number | null;
    dimension_count: number;
    latest_observed_at: string | null;
    next_expiry_at: string | null;
    updated_at: string;
  } | null;
  ability_observations: Array<Record<string, unknown>>;
  dynamic_tags: PlayerDynamicTagRecord[];
  external_ids: Array<Record<string, unknown>>;
}

export interface MatchDraft {
  external_key: string;
  competition_id: string | null;
  season_id: string | null;
  stage_id: string | null;
  round_id: string | null;
  home_team_id: string;
  away_team_id: string;
  kickoff_time: string;
  status: MatchStatus;
  venue: string | null;
  metadata: Record<string, unknown>;
}

export interface MatchRecord {
  id: string;
  external_key: string;
  competition_id: string | null;
  competition_name: string | null;
  season_id: string | null;
  stage_id: string | null;
  round_id: string | null;
  home_team_id: string;
  home_team_name: string;
  away_team_id: string;
  away_team_name: string;
  kickoff_time: string;
  status: MatchStatus;
  venue: string | null;
}

export interface LineupPlayerDraft {
  player_id: string;
  position_code: string | null;
  role_code: string | null;
  is_starter: boolean;
  shirt_number: number | null;
  expected_minutes: number | null;
  actual_minutes: number | null;
  sequence_no: number;
  bench_order: number | null;
  availability_status: AvailabilityStatus | null;
  starting_probability: number | null;
  membership_override: boolean;
  source_urls: string[];
  metadata: Record<string, unknown>;
}

export type LineupSnapshotType = "T-N" | "T-24h" | "T-6h" | "T-1h";

export interface LineupDraft {
  match_id: string;
  team_id: string;
  lineup_type: LineupType;
  snapshot_type: LineupSnapshotType;
  formation: string | null;
  formation_id: string | null;
  coach_id: string | null;
  captured_at: string;
  source_document_id: string | null;
  source_urls: string[];
  quality_score: number | null;
  metadata: Record<string, unknown>;
  players: LineupPlayerDraft[];
}

export interface LineupPairDraft {
  home: LineupDraft;
  away: LineupDraft;
}

export interface LineupPairRecord {
  home: LineupRecord;
  away: LineupRecord;
}

export interface LineupPlayerRecord {
  player_id: string;
  player_name: string;
  position_code: string | null;
  role_code: string | null;
  role_origin: "lineup_override" | "player_position_default" | "missing";
  role_source_position_code: string | null;
  is_starter: boolean;
  shirt_number: number | null;
  expected_minutes: number | null;
  actual_minutes: number | null;
  sequence_no: number;
  bench_order: number | null;
  availability_status: AvailabilityStatus | null;
  starting_probability: number | null;
  membership_override: boolean;
  source_urls: string[];
  validation_warning: string | null;
}

export interface LineupRecord {
  id: string;
  match_id: string;
  match_key: string;
  team_id: string;
  team_name: string;
  lineup_type: LineupType;
  snapshot_type: string;
  formation: string | null;
  formation_id: string | null;
  formation_code: string | null;
  formation_name: string | null;
  coach_id: string | null;
  coach_name: string | null;
  captured_at: string;
  status: string;
  quality_score: number | null;
  source_urls: string[];
  supersedes_lineup_id: string | null;
  model_validation_status: string;
  model_eligible: boolean;
  validation_errors: string[];
  validation_warnings: string[];
  player_count: number;
  starter_count: number;
  players: LineupPlayerRecord[];
}

export interface MatchLineupTeamChain {
  team_id: string;
  team_name: string;
  team_side: "home" | "away";
  selected_lineup_id: string | null;
  versions: LineupRecord[];
  blocking_issues: string[];
}

export interface MatchLineupChain {
  match_record: MatchRecord;
  snapshot_type: string;
  data_window_start_time: string | null;
  data_cutoff_time: string;
  home: MatchLineupTeamChain;
  away: MatchLineupTeamChain;
  ready_for_model: boolean;
  blocking_issues: string[];
}

export interface TeamMatchLineupHistoryItem {
  match_id: string;
  match_key: string;
  opponent_team_id: string;
  opponent_team_name: string;
  venue_side: "home" | "away";
  kickoff_time: string;
  lineup: LineupRecord;
}

export interface PositionReference {
  code: string;
  name: string;
  position_group: string;
  sort_order: number;
}

export interface TeamLineupPresetMemberDraft {
  player_id: string;
  position_code: string | null;
  role_code: string | null;
  is_starter: boolean;
  shirt_number: number | null;
  expected_minutes: number | null;
  sequence_no: number;
  bench_order: number | null;
  is_captain: boolean;
  metadata: Record<string, unknown>;
}

export interface TeamLineupPresetDraft {
  id: string | null;
  team_id: string;
  name: string;
  formation_id: string | null;
  coach_id: string | null;
  usage_context: string;
  usage_probability: number | null;
  is_default: boolean;
  source_lineup_id: string | null;
  notes: string | null;
  members: TeamLineupPresetMemberDraft[];
}

export interface TeamLineupPresetMemberRecord {
  player_id: string;
  player_name: string;
  alternate_name: string | null;
  position_code: string | null;
  role_code: string | null;
  role_origin: "lineup_override" | "player_position_default" | "missing";
  role_source_position_code: string | null;
  is_starter: boolean;
  shirt_number: number | null;
  expected_minutes: number | null;
  sequence_no: number;
  bench_order: number | null;
  is_captain: boolean;
  current_team_id: string | null;
  current_team_name: string | null;
  player_status: string;
  availability_status: AvailabilityStatus | null;
  metadata: Record<string, unknown>;
}

export interface TeamLineupPresetRecord {
  id: string;
  team_id: string;
  team_name: string;
  name: string;
  formation_id: string | null;
  formation_code: string | null;
  formation_name: string | null;
  coach_id: string | null;
  coach_name: string | null;
  usage_context: string;
  usage_probability: number | null;
  is_default: boolean;
  status: "active" | "archived";
  version: number;
  source_lineup_id: string | null;
  notes: string | null;
  starter_count: number;
  member_count: number;
  members: TeamLineupPresetMemberRecord[];
  created_at: string;
  updated_at: string;
}

export interface TeamLineupPresetApplicationPreview {
  preset: TeamLineupPresetRecord;
  can_apply: boolean;
  blockers: string[];
  warnings: string[];
}

export interface LineupHistoryRemovalResult {
  lineup_id: string;
  removal_mode: "deleted" | "archived";
  restored_lineup_id: string | null;
}

export interface PlayerCatalogReferenceData {
  teams: TeamOption[];
  season_team_memberships: SeasonTeamMembershipOption[];
  formations: FormationRecord[];
  providers: DataProviderRecord[];
  positions: PositionReference[];
  ability_dimensions: AbilityDimensionRecord[];
  dynamic_tag_definitions: PlayerDynamicTagDefinitionRecord[];
  upcoming_matches: MatchRecord[];
  managed_matches: MatchRecord[];
}

export type SpreadsheetImportMode = "add_only" | "add_and_update";
export type SpreadsheetRowStatus =
  | "ready_add"
  | "ready_update"
  | "ready_end_previous"
  | "conflict"
  | "error"
  | "skip"
  | "imported";

export interface SpreadsheetConflictCandidate {
  entity_id: string;
  display_name: string;
  detail: string | null;
}

export interface SpreadsheetImportRow {
  id: string;
  sheet_name: string;
  row_number: number;
  entity_type: string;
  action: "add" | "update" | "clear" | "skip";
  status: SpreadsheetRowStatus;
  message: string | null;
  payload: Record<string, unknown>;
  matched_entity_id: string | null;
  conflict_candidates: SpreadsheetConflictCandidate[];
}

export interface SpreadsheetImportCounts {
  total: number;
  ready_add: number;
  ready_update: number;
  ready_end_previous: number;
  conflict: number;
  error: number;
  skipped: number;
  imported: number;
}

export interface SpreadsheetImportPreview {
  batch_id: string;
  source_file_name: string;
  source_sha256: string;
  import_mode: SpreadsheetImportMode;
  counts: SpreadsheetImportCounts;
  rows: SpreadsheetImportRow[];
  created_at: string;
}

export interface SpreadsheetImportResolution {
  row_id: string;
  selected_entity_id: string | null;
  skip: boolean;
}

export interface SpreadsheetImportCommitResult {
  batch_id: string;
  inserted_count: number;
  updated_count: number;
  ended_previous_count: number;
  skipped_count: number;
  error_count: number;
  finished_at: string;
}


export interface TeamPackageExportSummary {
  output_path: string;
  format_version: string;
  visible_sheet_count: number;
}

export interface TeamPackagePreviewExportSummary {
  output_path: string;
  format_version: string;
  exported_row_count: number;
}

export interface TeamPackageCoverage {
  team_count: number;
  player_count: number;
  coach_count: number;
  formation_usage_count: number;
  team_ability_count: number;
  player_ability_count: number;
  player_dynamic_tag_count: number;
  player_role_count: number;
  availability_count: number;
  readiness_score: number;
  p4_input_ready: boolean;
  blockers: string[];
  warnings: string[];
}

export interface TeamPackageImportPreview {
  source_file_name: string;
  source_sha256: string;
  team_preview: SpreadsheetImportPreview | null;
  player_preview: SpreadsheetImportPreview | null;
  coverage: TeamPackageCoverage;
}

export interface TeamPackageCommitRequest {
  team_batch_id: string | null;
  player_batch_id: string | null;
}

export interface TeamPackageCommitResult {
  team_result: SpreadsheetImportCommitResult | null;
  player_result: SpreadsheetImportCommitResult | null;
  inserted_count: number;
  updated_count: number;
  ended_previous_count: number;
  skipped_count: number;
  error_count: number;
}

export interface MonthlyWorkbookExportSummary {
  output_path: string;
  workbook_kind: "team" | "player";
  team_count: number;
  player_count: number;
  coach_count: number;
  related_row_count: number;
  data_gap_count: number;
}

export interface SpreadsheetExportSummary {
  output_path: string;
  team_count: number;
  player_count: number;
  related_row_count: number;
}

export interface PlayerDynamicTagDefinitionRecord {
  code: string;
  name: string;
  category: string;
  minimum_value: number;
  maximum_value: number;
  default_value: number;
  default_ttl_hours: number;
  is_multiplier: boolean;
  description: string | null;
}

export interface PlayerDynamicTagDraft {
  player_id: string;
  tag_code: string;
  value: number;
  label: string | null;
  confidence: number;
  observed_at: string;
  valid_from: string;
  valid_to: string;
  competition_id: string | null;
  position_code: string | null;
  opponent_team_id: string | null;
  sample_size: number;
  source_type: string;
  calculation_version: string;
  source_document_id: string | null;
  metadata: Record<string, unknown>;
}

export interface PlayerDynamicTagRecord extends PlayerDynamicTagDraft {
  id: string;
  tag_name: string;
  category: string;
  competition_name: string | null;
  opponent_team_name: string | null;
}

export interface PlayerMatchContributionRequest {
  player_id: string;
  match_id: string | null;
  competition_id: string | null;
  position_code: string | null;
  role_code?: string | null;
  role_origin?: "lineup_override" | "player_position_default" | "missing" | null;
  role_source_position_code?: string | null;
  opponent_team_id: string | null;
  as_of: string;
  data_cutoff_time?: string | null;
  expected_minutes: number | null;
}

export interface PlayerMatchContribution {
  player_id: string;
  player_name: string;
  match_id: string | null;
  as_of: string;
  position_code: string | null;
  tactical_role_code: string | null;
  tactical_role_origin: "lineup_override" | "player_position_default" | "missing";
  tactical_role_source_position_code: string | null;
  tactical_role_confidence: number;
  base_ability: number;
  base_ability_confidence: number;
  effective_contribution: number;
  overall_confidence: number;
  expected_minutes_share: number;
  starting_probability: number | null;
  components: Array<{
    code: string;
    label: string;
    value: number;
    confidence: number;
    source: string;
  }>;
  applied_tags: PlayerDynamicTagRecord[];
  calculation_version: string;
}

export interface MatchLineupExportSummary {
  output_path: string;
  match_count: number;
  lineup_count: number;
  player_count: number;
}

export interface AiMatchPackageSummary {
  output_path: string;
  match_id: string;
  match_key: string;
  player_count: number;
  content_sha256: string;
}

export type AbilityCandidateStatus =
  "pending" | "accepted" | "rejected" | "superseded";
export type AbilityCandidateDecision = "accept" | "reject";

export interface MatchResultDraft {
  match_id: string;
  home_goals_90: number;
  away_goals_90: number;
  home_goals_extra_time: number | null;
  away_goals_extra_time: number | null;
  home_penalties: number | null;
  away_penalties: number | null;
  finalized_at: string;
  source_document_id: string | null;
  metadata: Record<string, unknown>;
}

export interface MatchResultRecord extends MatchResultDraft {}

export interface SubstitutionDraft {
  team_id: string;
  player_out_id: string | null;
  player_in_id: string | null;
  minute: number;
  period: string;
  reason: string | null;
  source_document_id: string | null;
  metadata: Record<string, unknown>;
}

export interface SubstitutionRecord extends SubstitutionDraft {
  id: string;
  match_id: string;
  team_name: string;
  player_out_name: string | null;
  player_in_name: string | null;
}

export interface PlayerPerformanceMetrics {
  goals: number;
  assists: number;
  expected_goals: number;
  expected_assists: number;
  shots: number;
  shots_on_target: number;
  key_passes: number;
  progressive_actions: number;
  tackles: number;
  interceptions: number;
  clearances: number;
  blocks: number;
  duels_won: number;
  duels_total: number;
  fouls: number;
  yellow_cards: number;
  red_cards: number;
  errors_leading_to_shot: number;
  provider_rating: number | null;
  extra: Record<string, unknown>;
}

export interface PlayerMatchObservationDraft {
  player_id: string;
  team_id: string;
  position_code: string | null;
  role_code: string | null;
  started: boolean;
  minutes_played: number;
  performance_score: number | null;
  input_confidence: number;
  metrics: PlayerPerformanceMetrics;
  source_document_id: string | null;
}

export interface MatchReviewDraft {
  match_id: string;
  review_version: string | null;
  data_coverage: number;
  source_run_id: string | null;
  result: MatchResultDraft;
  substitutions: SubstitutionDraft[];
  events: MatchReviewEventDraft[];
  player_observations: PlayerMatchObservationDraft[];
  notes: string | null;
}

export interface MatchReviewPackageSnapshotSummary {
  home_goals_90: number | null;
  away_goals_90: number | null;
  home_player_count: number;
  away_player_count: number;
  home_starter_count: number;
  away_starter_count: number;
}

export interface MatchReviewPackageIdentityCheck {
  package_id_matches_current_export: boolean;
  match_id_matches_selection: boolean;
  match_key_matches_database: boolean;
  team_identity_matches_database: boolean;
}

export interface MatchReviewPackageComparison {
  pre_match: MatchReviewPackageSnapshotSummary;
  current_database: MatchReviewPackageSnapshotSummary;
  proposed_import: MatchReviewPackageSnapshotSummary;
  identity: MatchReviewPackageIdentityCheck;
}

export type MatchReviewPackageWorkflowStatus =
  | "exported"
  | "preview_blocked"
  | "preview_valid"
  | "confirmed"
  | "facts_committed"
  | "review_created"
  | "settled"
  | "superseded";

export type MatchReviewPackageWorkflowStep =
  | "export_package"
  | "complete_external_data"
  | "preview_import"
  | "confirm_import"
  | "commit_facts"
  | "generate_review"
  | "settle_review"
  | "open_analytics";

export type MatchReviewPackageWorkflowAction =
  | "export_package"
  | "preview_import"
  | "confirm_import"
  | "commit_facts"
  | "generate_review"
  | "inspect_settlement_readiness"
  | "settle_review"
  | "open_analytics";

export interface MatchReviewPackageActionBlocker {
  action: MatchReviewPackageWorkflowAction;
  reason: string;
}

export interface MatchReviewPackageWorkflowRecord {
  package_id: string;
  match_id: string;
  match_key: string;
  status: MatchReviewPackageWorkflowStatus;
  completed_steps: MatchReviewPackageWorkflowStep[];
  allowed_actions: MatchReviewPackageWorkflowAction[];
  blocking_reasons: MatchReviewPackageActionBlocker[];
  next_action: MatchReviewPackageWorkflowAction | null;
  export_path: string;
  export_sha256: string;
  pre_match_snapshot: MatchReviewPackageSnapshotSummary;
  export_database_snapshot: MatchReviewPackageSnapshotSummary;
  import_path: string | null;
  import_sha256: string | null;
  preview_ready: boolean;
  preview: MatchReviewPackagePreview | null;
  confirmed_by: string | null;
  confirmation_note: string | null;
  review_id: string | null;
  exported_at: string;
  previewed_at: string | null;
  confirmed_at: string | null;
  facts_committed_at: string | null;
  review_created_at: string | null;
  settled_at: string | null;
  updated_at: string;
}

export interface MatchReviewPackageConfirmationRequest {
  package_id: string;
  confirmed_by: string | null;
  confirmation_note: string | null;
}

export interface MatchReviewPackageFactsCommitResult {
  home_lineup_id: string;
  away_lineup_id: string;
  workflow: MatchReviewPackageWorkflowRecord;
}

export interface MatchReviewPackageReviewResult {
  review: MatchReviewDetail;
  workflow: MatchReviewPackageWorkflowRecord;
}

export interface MatchReviewPackageSummary {
  output_path: string;
  package_id: string;
  match_id: string;
  match_key: string;
  lineup_count: number;
  player_count: number;
  content_sha256: string;
  pre_match_snapshot: MatchReviewPackageSnapshotSummary;
  export_database_snapshot: MatchReviewPackageSnapshotSummary;
}

export type MatchEventType =
  | "substitution"
  | "goal"
  | "own_goal"
  | "assist"
  | "penalty_goal"
  | "penalty_missed"
  | "yellow_card"
  | "second_yellow_card"
  | "red_card"
  | "injury"
  | "var"
  | "formation_change"
  | "goalkeeper_change"
  | "other";

export type MatchEventVerificationStatus = "unverified" | "verified" | "disputed";
export type MatchEventRevisionStatus = "active" | "corrected" | "cancelled" | "superseded";

export interface MatchReviewEventDraft {
  event_key: string | null;
  sequence_no: number | null;
  event_type: MatchEventType;
  team_id: string | null;
  player_id: string | null;
  related_player_id: string | null;
  minute: number;
  stoppage_minute: number | null;
  period: string;
  home_score: number | null;
  away_score: number | null;
  verification_status: MatchEventVerificationStatus;
  revision_status: MatchEventRevisionStatus;
  verified_at: string | null;
  source_document_id: string | null;
  source_package_id: string | null;
  revision_of_event_id: string | null;
  description: string | null;
  source_urls: string[];
  confidence: number;
  metadata: Record<string, unknown>;
}

export interface MatchReviewPackageDiffSummary {
  home_added_starters: string[];
  home_removed_starters: string[];
  away_added_starters: string[];
  away_removed_starters: string[];
  added_matchday_players: string[];
  removed_matchday_players: string[];
}

export interface MatchReviewPackagePreview {
  source_path: string;
  source_file_name: string;
  source_sha256: string;
  format_version: string;
  package_id: string;
  match_id: string;
  match_key: string;
  home_team_name: string;
  away_team_name: string;
  lineup_pair: LineupPairDraft;
  review: MatchReviewDraft;
  events: MatchReviewEventDraft[];
  comparison: MatchReviewPackageComparison;
  diff: MatchReviewPackageDiffSummary;
  warnings: string[];
  errors: string[];
  home_player_count: number;
  away_player_count: number;
  home_starter_count: number;
  away_starter_count: number;
  substitution_count: number;
  observation_count: number;
  ready: boolean;
}

export interface MatchReviewPackageCommitRequest {
  preview: MatchReviewPackagePreview;
  confirmed_by: string | null;
  confirmation_note: string | null;
}

export interface MatchReviewPackageCommitResult {
  home_lineup_id: string;
  away_lineup_id: string;
  review: MatchReviewDetail;
}

export interface MatchReviewSummary {
  id: string;
  match_id: string;
  match_key: string;
  home_team_name: string;
  away_team_name: string;
  review_version: string;
  status: string;
  data_coverage: number;
  source_run_id: string | null;
  calculation_version: string;
  result_snapshot: Record<string, unknown>;
  substitutions_snapshot: SubstitutionRecord[];
  prediction_evaluation: Record<string, unknown>;
  conclusions: Record<string, unknown>;
  created_at: string;
  finalized_at: string | null;
}

export interface PlayerMatchReviewRecord {
  id: string;
  match_review_id: string;
  player_id: string;
  player_name: string;
  team_id: string;
  team_name: string;
  role_code: string | null;
  started: boolean;
  entry_type: string;
  minutes_played: number | null;
  expected_performance: number | null;
  actual_performance: number | null;
  realization_ratio: number | null;
  confidence: number;
  contribution_weight: number;
  ability_candidate_count: number;
  metrics: Record<string, unknown>;
}

export interface TeamMatchReviewRecord {
  id: string;
  match_review_id: string;
  team_id: string;
  team_name: string;
  chemistry_score: number | null;
  lineup_continuity: number | null;
  performance_cohesion: number | null;
  bench_strength: number | null;
  bench_dropoff: number | null;
  substitution_impact: number | null;
  substitute_count: number;
  realization_score: number | null;
  confidence: number;
  metrics: Record<string, unknown>;
}

export interface AbilityUpdateCandidateRecord {
  id: string;
  match_review_id: string | null;
  player_match_review_id: string | null;
  player_id: string;
  player_name: string;
  dimension_code: string;
  dimension_name: string;
  current_value: number | null;
  proposed_value: number;
  confidence: number;
  sample_size: number;
  evidence: Record<string, unknown>;
  calculation_version: string;
  status: AbilityCandidateStatus;
  created_at: string;
  decided_at: string | null;
  decided_by: string | null;
  decision_note: string | null;
  accepted_observation_id: string | null;
}

export interface AbilityCandidateDecisionDraft {
  candidate_id: string;
  decision: AbilityCandidateDecision;
  decided_by: string | null;
  decision_note: string | null;
}

export interface MatchReviewEventRecord extends MatchReviewEventDraft {
  id: string;
  match_id: string;
  event_key: string;
  sequence_no: number;
  team_name: string | null;
  player_name: string | null;
  related_player_name: string | null;
  recorded_at: string;
  updated_at: string;
}

export interface MatchEventSummary {
  total_count: number;
  effective_count: number;
  cancelled_count: number;
  disputed_count: number;
  verified_count: number;
  event_type_counts: Record<string, number>;
  latest_home_score: number | null;
  latest_away_score: number | null;
  last_event_minute: number | null;
}

export interface MatchReviewDetail {
  summary: MatchReviewSummary;
  result: MatchResultRecord;
  substitutions: SubstitutionRecord[];
  events: MatchReviewEventRecord[];
  event_summary: MatchEventSummary;
  player_reviews: PlayerMatchReviewRecord[];
  team_reviews: TeamMatchReviewRecord[];
  ability_candidates: AbilityUpdateCandidateRecord[];
}

export interface ReviewableMatch {
  match_record: MatchRecord;
  result: MatchResultRecord | null;
  latest_review: MatchReviewSummary | null;
  player_observation_count: number;
  actual_lineup_count: number;
}

export interface CalibrationBucket {
  outcome: "home_win" | "draw" | "away_win";
  bucket_index: number;
  lower_bound: number;
  upper_bound: number;
  sample_size: number;
  predicted_mean: number;
  actual_rate: number;
  absolute_gap: number;
  ece_component: number;
}

export interface ModelComparisonRow {
  model_key: string;
  model_version: string;
  parameter_version: string;
  snapshot_type: string;
  sample_size: number;
  average_log_loss: number;
  average_brier: number;
  average_scoreline_nll: number | null;
  average_data_coverage: number;
  rank: number;
}

export interface DriftFinding {
  metric_name: string;
  baseline_mean: number;
  current_mean: number;
  absolute_delta: number;
  relative_delta: number | null;
  baseline_size: number;
  current_size: number;
  severity: "stable" | "warning" | "critical";
  direction: "up" | "down" | "flat";
}

export interface DataQualityFinding {
  id: string;
  scan_id: string;
  severity: "critical" | "warning" | "info";
  category: string;
  finding_code: string;
  entity_type: string;
  entity_id: string | null;
  message: string;
  evidence: Record<string, unknown>;
  status: string;
  detected_at: string;
}

export interface DataQualitySummary {
  scan_id: string | null;
  generated_at: string | null;
  critical: number;
  warning: number;
  info: number;
  open_total: number;
  findings: DataQualityFinding[];
}

export interface QueryPerformanceFinding {
  schema_name: string;
  table_name: string;
  estimated_rows: number;
  table_size_bytes: number;
  sequential_scans: number;
  index_scans: number;
  dead_rows: number;
  last_analyze: string | null;
  severity: string;
  recommendation: string | null;
}

export interface QueryPerformanceSummary {
  captured_at: string | null;
  database_size_bytes: number;
  tables: QueryPerformanceFinding[];
}

export interface AnalyticsOverview {
  generated_at: string | null;
  calculation_version: string;
  sample_size: number;
  average_log_loss: number | null;
  average_brier: number | null;
  average_scoreline_nll: number | null;
  expected_calibration_error: number | null;
  comparisons: ModelComparisonRow[];
  calibration: CalibrationBucket[];
  drift: DriftFinding[];
  data_quality: DataQualitySummary;
  query_performance: QueryPerformanceSummary;
}

export type P4Horizon = "T-24h" | "T-6h" | "T-1h" | "T-N";

export type P4FreezeTaskState =
  | "PLANNED"
  | "RESEARCH_QUEUED"
  | "RESEARCH_RUNNING"
  | "RESEARCH_SUCCEEDED"
  | "RESEARCH_PARTIAL"
  | "READY_TO_FREEZE"
  | "FREEZING"
  | "FROZEN"
  | "BLOCKED"
  | "MISSED"
  | "FAILED"
  | "CANCELLED";

export interface PlanP4HorizonsCommand {
  match_id: string;
  explicit_rule_package_id: string;
  requested_fact_keys: string[];
}

export interface P4FreezeTaskRecord {
  id: string;
  match_id: string;
  match_key: string;
  horizon: P4Horizon;
  kickoff_at: string;
  data_cutoff_at: string;
  research_due_at: string;
  freeze_deadline_at: string;
  rule_package_id: string;
  model_version_id: string;
  parameter_set_id: string;
  competition_profile_id: string;
  research_schema_version_id: string;
  snapshot_schema_version_id: string;
  requested_fact_keys: string[];
  trace_id: string;
  state: P4FreezeTaskState;
  research_run_id: string | null;
  research_job_id: string | null;
  freeze_job_id: string | null;
  snapshot_id: string | null;
  blockers: unknown;
  task_fingerprint: string;
  idempotency_key: string;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface P4FreezeTaskEventRecord {
  id: string;
  task_id: string;
  from_state: P4FreezeTaskState | null;
  to_state: P4FreezeTaskState;
  reason: string;
  payload: Record<string, unknown>;
  occurred_at: string;
}

export interface P4FreezeReadiness {
  task_id: string;
  ready: boolean;
  research_status: string | null;
  requested_fact_count: number;
  routed_fact_count: number;
  missing_fact_count: number;
  ignored_fact_count: number;
  blocked_fact_count: number;
  blockers: string[];
}

export interface P4MatchWorkspace {
  match_id: string;
  match_key: string;
  home_team_name: string;
  away_team_name: string;
  kickoff_at: string;
  competition_name: string | null;
  tasks: P4FreezeTaskRecord[];
}

export interface P4ResearchRunWorkspace {
  id: string;
  status: string;
  attempt_count: number;
  response_id: string | null;
  model_id: string | null;
  error_category: string | null;
  error_message: string | null;
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
}

export interface P4RoutedFact {
  route_key: string;
  field_key: string;
  target_module: string;
  target_slot: string;
  route_status: string;
  verification_state: string;
  selected_evidence_ids: string[];
  selected_value: unknown;
  reason: string;
}

export interface P4EvidenceWorkspaceRecord {
  id: string;
  field_key: string;
  entity_type: string;
  entity_id: string | null;
  value: unknown;
  verification_state: string;
  source_tier: string;
  source_url: string | null;
  source_title: string | null;
  source_domain: string | null;
  published_at: string | null;
  observed_at: string;
  effective_at: string | null;
  retrieved_at: string;
  timezone: string;
  conflict_group_id: string | null;
  created_at: string;
}

export interface P4ConflictWorkspaceRecord {
  id: string;
  field_key: string;
  entity_type: string;
  entity_id: string | null;
  conflict_key: string;
  status: string;
  evaluation_status: string | null;
  evidence_ids: string[];
  selected_evidence_ids: string[];
  manual_decision_kind: string | null;
  manual_decision_note: string | null;
  manual_decision_at: string | null;
  created_at: string;
}

export interface P4TaskWorkspace {
  task: P4FreezeTaskRecord;
  readiness: P4FreezeReadiness;
  events: P4FreezeTaskEventRecord[];
  research_run: P4ResearchRunWorkspace | null;
  routes: P4RoutedFact[];
  evidence: P4EvidenceWorkspaceRecord[];
  conflicts: P4ConflictWorkspaceRecord[];
  snapshot: Record<string, unknown> | null;
}

export type P4ManualConflictDecisionKind = "select_evidence" | "accept_unknown";

export interface ResolveP4ConflictCommand {
  task_id: string;
  conflict_id: string;
  decision_kind: P4ManualConflictDecisionKind;
  selected_evidence_ids: string[];
  note: string | null;
}

export type JobStatus =
  "queued" | "running" | "succeeded" | "failed" | "cancelled";

export interface BackgroundJob {
  id: string;
  job_type: string;
  status: JobStatus;
  progress: number;
  payload: Record<string, unknown>;
  result: Record<string, unknown> | null;
  error_message: string | null;
  priority: number;
  attempts: number;
  max_attempts: number;
  cancellation_requested: boolean;
  available_at: string;
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
  updated_at: string;
}

export interface EnqueueJobDraft {
  job_type:
    | "refresh_analytics"
    | "data_quality_scan"
    | "query_performance_scan"
    | "full_analysis_refresh";
  payload: Record<string, unknown>;
  idempotency_key: string | null;
  available_at?: string | null;
  priority: number;
  max_attempts: number;
}

export interface AiAnalysisPackageSummary {
  package_id: string;
  output_path: string;
  content_sha256: string;
  sample_size: number;
  created_at: string;
}

export interface AiAnalysisSuggestionDraft {
  suggestion_type: string;
  title: string;
  summary: string;
  severity: string;
  scope: Record<string, unknown>;
  payload: Record<string, unknown>;
  evidence: Record<string, unknown>;
}

export interface AiAnalysisResponsePreview {
  manifest: {
    format_version: string;
    response_id: string;
    created_at: string;
    source_package_id: string | null;
    content_sha256: string;
  };
  suggestions: AiAnalysisSuggestionDraft[];
  blocking_errors: string[];
  warnings: string[];
}

export interface AiAnalysisSuggestionRecord {
  id: string;
  response_id: string;
  suggestion_type: string;
  title: string;
  summary: string;
  severity: string;
  scope: Record<string, unknown>;
  payload: Record<string, unknown>;
  evidence: Record<string, unknown>;
  status: string;
  created_at: string;
  decided_at: string | null;
  decision_note: string | null;
  linked_candidate_id: string | null;
}

export interface AiSuggestionDecisionDraft {
  suggestion_id: string;
  decision: "accept" | "reject";
  decided_by: string | null;
  decision_note: string | null;
}

export interface DataQualityDecisionDraft {
  finding_id: string;
  decision: "resolve" | "ignore";
  resolution_note: string | null;
}

export interface ParameterTuningDraft {
  competition_id: string | null;
  snapshot_type: "T-N" | "T-24h" | "T-6h" | "T-1h";
  target_module:
    | "lineup_realization"
    | "history"
    | "state"
    | "venue"
    | "draw_correction"
    | "synergy";
  max_relative_change: number;
  minimum_sample_size: number;
}

export interface ParameterTuningDecisionDraft {
  candidate_id: string;
  decision: "accept_for_backtest" | "reject";
  decision_note: string | null;
}

export interface ParameterTuningCandidateRecord {
  id: string;
  competition_id: string | null;
  competition_name: string | null;
  competition_profile_id: string | null;
  partition_key: string | null;
  model_key: string;
  model_version: string;
  parameter_version: string;
  snapshot_type: string;
  target_module: string;
  sample_size: number;
  baseline_model_version_id: string | null;
  baseline_parameter_set_id: string | null;
  candidate_model_version_id: string | null;
  candidate_parameter_set_id: string | null;
  candidate_model_version: string | null;
  candidate_parameter_version: string | null;
  candidate_definition_sha256: string | null;
  baseline_metrics: Record<string, unknown>;
  calibration_bias: Record<string, unknown>;
  proposed_adjustments: Record<string, unknown>;
  constraints: Record<string, unknown>;
  training_window: Record<string, unknown>;
  validation_window: Record<string, unknown>;
  holdout_window: Record<string, unknown>;
  rationale: string;
  status:
    | "pending"
    | "accepted_for_backtest"
    | "rejected"
    | "shadow_running"
    | "shadow_passed"
    | "shadow_failed"
    | "promoted"
    | "rolled_back"
    | "blocked_by_h"
    | "superseded";
  created_at: string;
  decided_at: string | null;
  decision_note: string | null;
}

export interface ParameterLifecycleReadinessRequest {
  competition_id: string | null;
  snapshot_type: "T-N" | "T-24h" | "T-6h" | "T-1h";
  minimum_sample_size: number;
}

export interface ParameterLifecycleReadiness {
  partition_key: string;
  competition_id: string | null;
  competition_name: string | null;
  competition_profile_id: string | null;
  snapshot_type: string;
  h_contract_ready: boolean;
  h_contract_version: string | null;
  settled_sample_count: number;
  eligible_sample_count: number;
  minimum_sample_size: number;
  active_model_version_id: string | null;
  active_parameter_set_id: string | null;
  active_model_version: string | null;
  active_parameter_version: string | null;
  blocked_reasons: string[];
  ready_for_shadow_validation: boolean;
  ready_for_promotion: boolean;
}

export interface ParameterShadowValidationRequest {
  candidate_id: string;
}

export interface ParameterShadowValidationRecord {
  id: string;
  candidate_id: string;
  validation_key: string;
  partition_key: string;
  sample_count: number;
  baseline_metrics: Record<string, unknown>;
  candidate_metrics: Record<string, unknown>;
  metric_deltas: Record<string, unknown>;
  gate_results: Record<string, unknown>;
  status: "passed" | "failed" | "blocked";
  generated_at: string;
}

export interface ParameterPromotionRequest {
  candidate_id: string;
  decided_by: string | null;
  decision_note: string;
}

export interface ParameterRollbackRequest {
  candidate_id: string;
  decided_by: string | null;
  decision_note: string;
}

export interface ParameterPromotionDecisionRecord {
  id: string;
  candidate_id: string;
  decision: "promote" | "rollback";
  previous_binding_state: unknown;
  new_binding_state: unknown;
  decided_by: string | null;
  decision_note: string;
  created_at: string;
}

export interface LineupBuilderPlayer {
  player_id: string;
  player_name: string;
  player_secondary_name: string | null;
  position_code: string | null;
  role_code: string | null;
  is_starter: boolean;
  expected_minutes: number | null;
  shirt_number: number | null;
  bench_order: number | null;
  starting_probability: number | null;
  membership_override: boolean;
  availability_status: AvailabilityStatus | null;
}

export interface LineupBuilderFormState {
  match_id: string;
  team_id: string;
  lineup_type: LineupType;
  snapshot_type: LineupSnapshotType;
  formation_id: string;
  formation: string;
  coach_id: string;
  source_urls: string;
  captured_at: string;
  quality_score: number;
}

export interface PairedLineupSideState {
  team_id: string;
  team_name: string;
  formation_id: string;
  formation: string;
  coach_id: string;
  quality_score: number;
  players: LineupBuilderPlayer[];
  candidates: PlayerListItem[];
}

export interface PairedLineupBuilderState {
  match_id: string;
  lineup_type: LineupType;
  snapshot_type: LineupSnapshotType;
  captured_at: string;
  source_urls: string;
  home: PairedLineupSideState;
  away: PairedLineupSideState;
}

export type OpenAiReasoningEffort =
  "none" | "minimal" | "low" | "medium" | "high" | "xhigh";
export type OpenAiSearchContextSize = "low" | "medium" | "high";
export type OpenAiApiProtocol = "responses" | "chat_completions";
export type OpenAiTokenLimitField = "max_output_tokens" | "max_tokens";
export type OpenAiApiWorkspaceWebSearchMode =
  "disabled" | "auto" | "responses_web_search";

export interface OpenAiApiExampleCandidate {
  protocol: OpenAiApiProtocol;
  endpoint_url: string;
  api_base_url: string;
  model_id: string | null;
  max_output_tokens: number | null;
  token_limit_field: OpenAiTokenLimitField;
  api_key: string | null;
  has_authorization_header: boolean;
  sanitized_example: string;
  formal_research_candidate: boolean;
  warnings: string[];
}

export interface OpenAiApiExampleParseResult {
  selected: OpenAiApiExampleCandidate;
  candidates: OpenAiApiExampleCandidate[];
}

export interface OpenAiProfileDraft {
  id: string | null;
  name: string;
  api_key: string | null;
  api_base_url: string;
  api_protocol: OpenAiApiProtocol;
  api_endpoint: string;
  token_limit_field: OpenAiTokenLimitField;
  api_workspace_web_search_mode: OpenAiApiWorkspaceWebSearchMode;
  api_example_template: string | null;
  research_model: string;
  extraction_model: string;
  fallback_model: string | null;
  reasoning_effort: OpenAiReasoningEffort;
  timeout_seconds: number;
  max_retries: number;
  max_concurrency: number;
  max_output_tokens: number;
  max_tool_calls: number;
  search_context_size: OpenAiSearchContextSize;
}

export interface OpenAiProfileSummary {
  id: string;
  name: string;
  provider: "openai_compatible";
  api_protocol: OpenAiApiProtocol;
  api_endpoint: string;
  token_limit_field: OpenAiTokenLimitField;
  api_workspace_web_search_mode: OpenAiApiWorkspaceWebSearchMode;
  api_example_template: string | null;
  formal_research_candidate: boolean;
  is_active: boolean;
  has_api_key: boolean;
  api_key_mask: string | null;
  api_base_url: string;
  research_model: string;
  extraction_model: string;
  fallback_model: string | null;
  reasoning_effort: OpenAiReasoningEffort;
  timeout_seconds: number;
  max_retries: number;
  max_concurrency: number;
  max_output_tokens: number;
  max_tool_calls: number;
  search_context_size: OpenAiSearchContextSize;
  last_test_status: "untested" | "success" | "failed";
  last_test_message: string | null;
  last_tested_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface OpenAiProfilesState {
  active_profile_id: string | null;
  profiles: OpenAiProfileSummary[];
  config_path: string;
}

export interface OpenAiProfileTestResult {
  profile_id: string;
  profile_name: string;
  model_id: string;
  protocol: OpenAiApiProtocol;
  endpoint_url: string;
  latency_ms: number;
  provider_request_id: string | null;
  tested_at: string;
}

export interface ApiWorkspacePreset {
  key: string;
  title: string;
  description: string;
  category: string;
  web_search_enabled: boolean;
  requires_match: boolean;
  allowed_operation_types: string[];
  suggested_questions: string[];
}

export interface ApiWorkspaceAttachment {
  name: string;
  media_type: string;
  content: string;
  content_sha256: string;
  original_size_bytes: number;
  truncated: boolean;
}

export interface ApiWorkspaceSessionRecord {
  id: string;
  profile_id: string;
  preset_key: string;
  title: string;
  match_id: string | null;
  match_label: string | null;
  metadata: Record<string, unknown>;
  status: string;
  message_count: number;
  pending_operation_count: number;
  created_at: string;
  updated_at: string;
}

export interface ApiWorkspaceMessageRecord {
  id: string;
  session_id: string;
  role: "user" | "assistant";
  content: string;
  structured_payload: Record<string, unknown>;
  citations:
    | {
        citations?: Array<{ url: string; title: string; domain: string }>;
        sources?: Array<{ url: string; title: string | null; domain: string }>;
      }
    | unknown[];
  attachments: Array<{
    name: string;
    media_type: string;
    content_sha256: string;
    original_size_bytes: number;
    truncated: boolean;
  }>;
  provider_response_id: string | null;
  model_id: string | null;
  token_usage: Record<string, number>;
  created_at: string;
}

export interface ApiWorkspaceOperationRecord {
  id: string;
  session_id: string;
  message_id: string;
  proposal_key: string;
  operation_type: string;
  payload: Record<string, unknown>;
  rationale: string;
  confidence: number;
  status:
    | "pending"
    | "applying"
    | "applied"
    | "failed"
    | "rejected"
    | "manual_review";
  result: Record<string, unknown>;
  error_message: string | null;
  idempotency_key: string;
  created_at: string;
  decided_at: string | null;
}

export interface ApiWorkspaceGeneratedFileRecord {
  id: string;
  session_id: string;
  message_id: string;
  filename: string;
  media_type: "text/plain" | "text/markdown" | "application/json" | "text/csv";
  content_sha256: string;
  size_bytes: number;
  created_at: string;
}

export interface ApiWorkspaceSessionDetail {
  session: ApiWorkspaceSessionRecord;
  messages: ApiWorkspaceMessageRecord[];
  operations: ApiWorkspaceOperationRecord[];
  files: ApiWorkspaceGeneratedFileRecord[];
}

export interface SendApiWorkspaceCommand {
  session_id: string | null;
  profile_id: string;
  preset_key: string;
  title: string | null;
  match_id: string | null;
  context_entity_type: "team" | "player" | null;
  context_entity_id: string | null;
  context_entity_label: string | null;
  include_context: boolean;
  request_id: string;
  message: string;
  attachments: ApiWorkspaceAttachment[];
}

export interface ApiWorkspaceExportResult {
  file_id: string;
  output_path: string;
  size_bytes: number;
  content_sha256: string;
}

export interface PostmatchSettlementReadiness {
  match_review_id: string;
  match_id: string;
  match_key: string;
  home_team_name: string;
  away_team_name: string;
  result_ready: boolean;
  finalized_review_ready: boolean;
  successful_run_ready: boolean;
  frozen_snapshot_ready: boolean;
  snapshot_identity_ready: boolean;
  real_evidence_snapshot_ready: boolean;
  competition_profile_ready: boolean;
  formal_horizon_ready: boolean;
  existing_settlement_id: string | null;
  blocked_reasons: string[];
  ready: boolean;
}

export interface PostmatchSettlementDraft {
  match_review_id: string;
  settled_by: string | null;
  settlement_note: string | null;
}

export interface PostmatchSettlementRecord {
  id: string;
  match_id: string;
  match_review_id: string;
  model_run_id: string;
  feature_snapshot_id: string;
  competition_id: string;
  competition_name: string;
  competition_profile_id: string;
  model_version_id: string;
  model_version: string;
  parameter_set_id: string;
  parameter_version: string;
  rule_package_id: string;
  horizon: string;
  match_key: string;
  home_team_name: string;
  away_team_name: string;
  home_goals_90: number;
  away_goals_90: number;
  result_finalized_at: string;
  result_fingerprint: string;
  settlement_key: string;
  settlement_version: string;
  status: string;
  evidence_item_count: number;
  scored_evidence_count: number;
  drift_status: string | null;
  settled_by: string | null;
  settlement_note: string | null;
  metadata: Record<string, unknown>;
  settled_at: string;
}

export type EvidenceVerdict = "correct" | "partial" | "incorrect" | "not_verifiable";

export interface EvidenceScoringDecisionDraft {
  item_id: string;
  verdict: EvidenceVerdict;
  decided_by: string | null;
  decision_note: string;
}

export interface EvidenceScoringItemRecord {
  id: string;
  settlement_id: string;
  evidence_id: string;
  provider_id: string | null;
  provider_name: string | null;
  source_document_id: string | null;
  field_key: string;
  verification_state: string;
  source_tier: string;
  source_title: string | null;
  source_domain: string | null;
  published_at: string | null;
  retrieved_at: string;
  data_cutoff_at: string;
  timeliness_score: number;
  decision_id: string | null;
  verdict: EvidenceVerdict | null;
  accuracy_score: number | null;
  reliability_score: number | null;
  decided_by: string | null;
  decision_note: string | null;
  decided_at: string | null;
  status: "pending" | "scored" | "not_verifiable";
  created_at: string;
}

export interface ProviderScoreSnapshotRecord {
  id: string;
  provider_id: string;
  provider_name: string;
  scope_key: string;
  competition_id: string;
  competition_profile_id: string;
  model_version_id: string;
  parameter_set_id: string;
  horizon: string;
  sample_size: number;
  correct_count: number;
  partial_count: number;
  incorrect_count: number;
  not_verifiable_count: number;
  accuracy_mean: number;
  timeliness_mean: number;
  reliability_mean: number;
  weighted_score: number;
  decision_set_sha256: string;
  calculation_version: string;
  generated_at: string;
}

export interface PostmatchDriftFindingRecord {
  metric_name: string;
  baseline_mean: number;
  current_mean: number;
  absolute_delta: number;
  relative_delta: number | null;
  severity: "stable" | "warning" | "critical";
  direction: "up" | "down" | "flat";
}

export interface PostmatchDriftRunRecord {
  id: string;
  competition_id: string;
  competition_name: string;
  competition_profile_id: string;
  model_version_id: string;
  model_version: string;
  parameter_set_id: string;
  parameter_version: string;
  horizon: string;
  partition_key: string;
  baseline_size: number;
  current_size: number;
  baseline_window: Record<string, unknown>;
  current_window: Record<string, unknown>;
  status: "insufficient" | "stable" | "warning" | "critical";
  run_key: string;
  calculation_version: string;
  findings: PostmatchDriftFindingRecord[];
  generated_at: string;
}

export interface PostmatchMonitoringRequest {
  competition_id: string;
  horizon: string;
  baseline_size: number;
  current_size: number;
}

export interface PostmatchOverview {
  settlement_count: number;
  pending_evidence_count: number;
  scored_evidence_count: number;
  settlements: PostmatchSettlementRecord[];
  evidence_queue: EvidenceScoringItemRecord[];
  provider_scores: ProviderScoreSnapshotRecord[];
  drift_runs: PostmatchDriftRunRecord[];
}

export type ReleaseAcceptanceStatus = "pass" | "warning" | "blocked";

export interface ReleaseAcceptanceRequest {
  performance_window_days: number;
  cost_window_days: number;
  daily_cost_budget_usd: number | null;
  monthly_cost_budget_usd: number | null;
  requested_by: string | null;
}

export interface ReleaseAcceptanceCheck {
  id: string;
  run_id: string;
  sequence_no: number;
  category: "chain" | "performance" | "security" | "cost" | "release";
  check_code: string;
  title: string;
  status: ReleaseAcceptanceStatus;
  summary: string;
  remediation: string | null;
  evidence: Record<string, unknown> | unknown[] | null;
  duration_ms: number;
}

export interface ReleaseAcceptanceCategorySummary {
  category: string;
  passed: number;
  warnings: number;
  blocked: number;
}

export interface ReleaseAcceptancePerformanceSummary {
  database_latency_ms: number;
  recent_model_run_count: number;
  recent_model_run_p95_ms: number | null;
  recent_model_failure_count: number;
  query_warning_count: number;
}

export interface ReleaseAcceptanceCostSummary {
  window_days: number;
  completed_requests: number;
  failed_requests: number;
  search_calls: number;
  estimated_cost_usd: number;
  latest_day_cost_usd: number;
  daily_budget_usd: number | null;
  monthly_budget_usd: number | null;
}

export interface ReleaseAcceptanceRun {
  id: string;
  app_version: string;
  contract_version: string;
  fixture_version: string;
  overall_status: ReleaseAcceptanceStatus;
  started_at: string;
  completed_at: string;
  requested_by: string | null;
  report_sha256: string;
  passed_count: number;
  warning_count: number;
  blocked_count: number;
  category_summaries: ReleaseAcceptanceCategorySummary[];
  performance: ReleaseAcceptancePerformanceSummary;
  cost: ReleaseAcceptanceCostSummary;
  checks: ReleaseAcceptanceCheck[];
}

export interface ReleaseAcceptanceRunSummary {
  id: string;
  app_version: string;
  overall_status: ReleaseAcceptanceStatus;
  completed_at: string;
  requested_by: string | null;
  passed_count: number;
  warning_count: number;
  blocked_count: number;
  report_sha256: string;
}
