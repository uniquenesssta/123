use football_domain::{postmatch, review};
fn assert_same_type<T>(_: Option<T>, _: Option<T>) {}

#[test]
fn r2_06_review_module_paths_preserve_root_type_identity() {
    assert_same_type(None::<football_domain::AbilityCandidateStatus>, None::<review::AbilityCandidateStatus>);
    assert_same_type(None::<football_domain::AbilityCandidateDecision>, None::<review::AbilityCandidateDecision>);
    assert_same_type(None::<football_domain::MatchResultDraft>, None::<review::MatchResultDraft>);
    assert_same_type(None::<football_domain::MatchResultRecord>, None::<review::MatchResultRecord>);
    assert_same_type(None::<football_domain::SubstitutionDraft>, None::<review::SubstitutionDraft>);
    assert_same_type(None::<football_domain::SubstitutionRecord>, None::<review::SubstitutionRecord>);
    assert_same_type(None::<football_domain::PlayerPerformanceMetrics>, None::<review::PlayerPerformanceMetrics>);
    assert_same_type(None::<football_domain::PlayerMatchObservationDraft>, None::<review::PlayerMatchObservationDraft>);
    assert_same_type(None::<football_domain::PlayerMatchObservationRecord>, None::<review::PlayerMatchObservationRecord>);
    assert_same_type(None::<football_domain::MatchReviewDraft>, None::<review::MatchReviewDraft>);
    assert_same_type(None::<football_domain::PlayerMatchReviewRecord>, None::<review::PlayerMatchReviewRecord>);
    assert_same_type(None::<football_domain::TeamMatchReviewRecord>, None::<review::TeamMatchReviewRecord>);
    assert_same_type(None::<football_domain::MatchReviewSummary>, None::<review::MatchReviewSummary>);
    assert_same_type(None::<football_domain::AbilityUpdateCandidateRecord>, None::<review::AbilityUpdateCandidateRecord>);
    assert_same_type(None::<football_domain::AbilityCandidateDecisionDraft>, None::<review::AbilityCandidateDecisionDraft>);
    assert_same_type(None::<football_domain::MatchReviewEventRecord>, None::<review::MatchReviewEventRecord>);
    assert_same_type(None::<football_domain::MatchReviewDetail>, None::<review::MatchReviewDetail>);
    assert_same_type(None::<football_domain::ReviewableMatch>, None::<review::ReviewableMatch>);
    assert_same_type(None::<football_domain::ReviewPlayerBaseline>, None::<review::ReviewPlayerBaseline>);
    assert_same_type(None::<football_domain::ReviewTeamContext>, None::<review::ReviewTeamContext>);
    assert_same_type(None::<football_domain::PredictionReviewContext>, None::<review::PredictionReviewContext>);
    assert_same_type(None::<football_domain::ReviewPreparationData>, None::<review::ReviewPreparationData>);
    assert_same_type(None::<football_domain::AbilityCandidateProposal>, None::<review::AbilityCandidateProposal>);
    assert_same_type(None::<football_domain::CalculatedPlayerReview>, None::<review::CalculatedPlayerReview>);
    assert_same_type(None::<football_domain::CalculatedTeamReview>, None::<review::CalculatedTeamReview>);
    assert_same_type(None::<football_domain::CalculatedMatchReview>, None::<review::CalculatedMatchReview>);
    assert_same_type(None::<football_domain::MatchEventType>, None::<review::MatchEventType>);
    assert_same_type(None::<football_domain::MatchEventVerificationStatus>, None::<review::MatchEventVerificationStatus>);
    assert_same_type(None::<football_domain::MatchEventRevisionStatus>, None::<review::MatchEventRevisionStatus>);
    assert_same_type(None::<football_domain::MatchEventSummary>, None::<review::MatchEventSummary>);
    assert_same_type(None::<football_domain::MatchReviewPackageSnapshotSummary>, None::<review::MatchReviewPackageSnapshotSummary>);
    assert_same_type(None::<football_domain::MatchReviewPackageIdentityCheck>, None::<review::MatchReviewPackageIdentityCheck>);
    assert_same_type(None::<football_domain::MatchReviewPackageComparison>, None::<review::MatchReviewPackageComparison>);
    assert_same_type(None::<football_domain::MatchReviewPackageWorkflowRecord>, None::<review::MatchReviewPackageWorkflowRecord>);
    assert_same_type(None::<football_domain::MatchReviewPackageConfirmationRequest>, None::<review::MatchReviewPackageConfirmationRequest>);
    assert_same_type(None::<football_domain::MatchReviewPackageFactsCommitResult>, None::<review::MatchReviewPackageFactsCommitResult>);
    assert_same_type(None::<football_domain::MatchReviewPackageReviewResult>, None::<review::MatchReviewPackageReviewResult>);
    assert_same_type(None::<football_domain::MatchReviewPackageData>, None::<review::MatchReviewPackageData>);
    assert_same_type(None::<football_domain::MatchReviewPackageSummary>, None::<review::MatchReviewPackageSummary>);
    assert_same_type(None::<football_domain::MatchReviewEventDraft>, None::<review::MatchReviewEventDraft>);
    assert_same_type(None::<football_domain::MatchReviewPackageDiffSummary>, None::<review::MatchReviewPackageDiffSummary>);
    assert_same_type(None::<football_domain::MatchReviewPackagePreview>, None::<review::MatchReviewPackagePreview>);
    assert_same_type(None::<football_domain::MatchReviewPackageCommitRequest>, None::<review::MatchReviewPackageCommitRequest>);
    assert_same_type(None::<football_domain::MatchReviewPackageCommitResult>, None::<review::MatchReviewPackageCommitResult>);
    assert_same_type(None::<football_domain::MatchReviewPackageWorkflowStatus>, None::<review::MatchReviewPackageWorkflowStatus>);
    assert_same_type(None::<football_domain::MatchReviewPackageWorkflowStep>, None::<review::MatchReviewPackageWorkflowStep>);
    assert_same_type(None::<football_domain::MatchReviewPackageWorkflowAction>, None::<review::MatchReviewPackageWorkflowAction>);
    assert_same_type(None::<football_domain::MatchReviewPackageActionBlocker>, None::<review::MatchReviewPackageActionBlocker>);
}

#[test]
fn r2_06_postmatch_module_paths_preserve_root_type_identity() {
    assert_same_type(None::<football_domain::PostmatchSettlementReadiness>, None::<postmatch::PostmatchSettlementReadiness>);
    assert_same_type(None::<football_domain::PostmatchSettlementDraft>, None::<postmatch::PostmatchSettlementDraft>);
    assert_same_type(None::<football_domain::PostmatchSettlementRecord>, None::<postmatch::PostmatchSettlementRecord>);
    assert_same_type(None::<football_domain::EvidenceVerdict>, None::<postmatch::EvidenceVerdict>);
    assert_same_type(None::<football_domain::EvidenceScoringDecisionDraft>, None::<postmatch::EvidenceScoringDecisionDraft>);
    assert_same_type(None::<football_domain::EvidenceScoringItemRecord>, None::<postmatch::EvidenceScoringItemRecord>);
    assert_same_type(None::<football_domain::ProviderScoreSnapshotRecord>, None::<postmatch::ProviderScoreSnapshotRecord>);
    assert_same_type(None::<football_domain::PostmatchDriftFindingRecord>, None::<postmatch::PostmatchDriftFindingRecord>);
    assert_same_type(None::<football_domain::PostmatchDriftRunRecord>, None::<postmatch::PostmatchDriftRunRecord>);
    assert_same_type(None::<football_domain::PostmatchMonitoringRequest>, None::<postmatch::PostmatchMonitoringRequest>);
    assert_same_type(None::<football_domain::PostmatchOverview>, None::<postmatch::PostmatchOverview>);
}
