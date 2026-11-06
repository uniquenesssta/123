# R17-01 实际验证覆盖矩阵

基线ad743482f25aa44217422187925335e7edce84f1；分支rewrite/r17-verification-system。此矩阵索引当前生产路径、原测试及证据层级，不是新的运行时配置或持续runner。生成证据根：O:/Project/FOODBALL/logs/r17-01-20261107。报告生成本身不代表功能通过。

## 执行入口与边界

| 入口 | 当前实际内容 | 本批结果/界限 |
| --- | --- | --- |
| verify:architecture | 35个原架构/状态/保护/服务/持久化守卫 | 当前正向通过；不能代替业务或PG |
| verify:frontend / build | 96个Node项 + TypeScript + Vite，共98步 | 当前完整通过；包含两套命令契约和原fixture/真实生产工作流/完整挂载 |
| verify:all | architecture → release（依赖+frontend）→ rust | 已补架构入口；失败时真实npm链停止，不调用后续frontend/Rust。正向组件分别核验，未另重复完整调用verify:all |
| verify:rust | 锁文件/目标目录、fmt、clippy、workspace tests | 当前Rust树/Cargo配置/锁与正式c1574990相同；复用CI42125458762的768通过/40 PG ignored，不写成本批重新执行 |
| Windows Automated | setup、frontend、Rust、Tauri release、startup | R16精确c1574990官方证据已接受；R17自身CI/包在推送后另验 |
| Windows Full | 2个PG target + 实际人工/客户端链 + full日志 | 当前未执行；全部真实环境义务仍R17-06，额外18个target不得漏验 |
| tests/ui截图 | 17个fixture视口，原PNG比较 | 当前通过；不是17生产页，也不是Figma全状态或真实写入 |

几何矩阵G为1440×900、960×720、1280×720、1920×1080 × light/dark；各browser源的显式循环均已核对。S02目录几何与S08/双边编辑的明暗范围按原132项脚本分别解释。下面的交互/错误/恢复状态按原脚本实际视口执行，不宣称每个状态与G全排列。仅对既有批准记录和已测同状态复用，不更新PNG/Figma或扩大成逐像素全状态PASS。真实数据/服务状态均有R17-06归属。

## 测试组与真实生产调用

| 组 | 生产工作流测试 | 完整挂载入口 | 已关闭范围的原记录 |
| --- | --- | --- | --- |
| readonly | [scripts/verify-browser-bootstrap.mjs](../../../scripts/verify-browser-bootstrap.mjs) | [scripts/design-refresh/verifyMountedDesign.mjs](../../../scripts/design-refresh/verifyMountedDesign.mjs) | [R13-core-ui-features/R13-01-read-only-entry-pilot.md](../R13-core-ui-features/R13-01-read-only-entry-pilot.md) |
| database | [scripts/features/verifyDatabaseWorkflow.mjs](../../../scripts/features/verifyDatabaseWorkflow.mjs) | [scripts/design-refresh/verifyMountedDesign.mjs](../../../scripts/design-refresh/verifyMountedDesign.mjs) | [R13-core-ui-features/R13-02-database-connection-and-reset.md](../R13-core-ui-features/R13-02-database-connection-and-reset.md) |
| rules | [scripts/features/verifyCompetitionRulesWorkflow.mjs](../../../scripts/features/verifyCompetitionRulesWorkflow.mjs) | [scripts/design-refresh/verifyMountedDesign.mjs](../../../scripts/design-refresh/verifyMountedDesign.mjs) | [R13-core-ui-features/R13-03-competition-rules-and-routing.md](../R13-core-ui-features/R13-03-competition-rules-and-routing.md) |
| teams | [scripts/features/verifyTeamsWorkflow.mjs](../../../scripts/features/verifyTeamsWorkflow.mjs) | [scripts/design-refresh/verifyMountedDesign.mjs](../../../scripts/design-refresh/verifyMountedDesign.mjs) | [R13-core-ui-features/R13-05-team-catalog-profile-and-package.md](../R13-core-ui-features/R13-05-team-catalog-profile-and-package.md) |
| players | [scripts/features/verifyPlayersWorkflow.mjs](../../../scripts/features/verifyPlayersWorkflow.mjs) | [scripts/design-refresh/verifyMountedDesign.mjs](../../../scripts/design-refresh/verifyMountedDesign.mjs) | [R13-core-ui-features/R13-06-player-catalog-history-and-observations.md](../R13-core-ui-features/R13-06-player-catalog-history-and-observations.md) |
| lineups | [scripts/features/verifyLineupsWorkflow.mjs](../../../scripts/features/verifyLineupsWorkflow.mjs) + [scripts/features/verifyMatchImports.mjs](../../../scripts/features/verifyMatchImports.mjs) | [scripts/features/lineups-browser/verifyLineupsBrowser.mjs](../../../scripts/features/lineups-browser/verifyLineupsBrowser.mjs) | [R13-core-ui-features/R13-07-presets-and-paired-lineups.md](../R13-core-ui-features/R13-07-presets-and-paired-lineups.md) |
| workbooks | [scripts/features/verifyWorkbooksWorkflow.mjs](../../../scripts/features/verifyWorkbooksWorkflow.mjs) | [scripts/features/verifyWorkbooksBrowser.mjs](../../../scripts/features/verifyWorkbooksBrowser.mjs) | [R13-core-ui-features/R13-09-workbook-workflows.md](../R13-core-ui-features/R13-09-workbook-workflows.md) |
| prediction | [scripts/features/verifyPredictionWorkflow.mjs](../../../scripts/features/verifyPredictionWorkflow.mjs) | [scripts/features/verifyPredictionBrowser.mjs](../../../scripts/features/verifyPredictionBrowser.mjs) | [R14-prediction-review-analytics-ui/R14-01-context-readiness-and-execution.md](../R14-prediction-review-analytics-ui/R14-01-context-readiness-and-execution.md) |
| research | [scripts/features/verifyResearchWorkflow.mjs](../../../scripts/features/verifyResearchWorkflow.mjs) | [scripts/features/verifyResearchBrowser.mjs](../../../scripts/features/verifyResearchBrowser.mjs) | [R14-prediction-review-analytics-ui/R14-06-research-evidence-and-freeze.md](../R14-prediction-review-analytics-ui/R14-06-research-evidence-and-freeze.md) |
| runs | [scripts/features/verifyRunsWorkflow.mjs](../../../scripts/features/verifyRunsWorkflow.mjs) | [scripts/features/verifyRunsBrowser.mjs](../../../scripts/features/verifyRunsBrowser.mjs) | [R14-prediction-review-analytics-ui/R14-07-run-history-and-lineage.md](../R14-prediction-review-analytics-ui/R14-07-run-history-and-lineage.md) |
| review | [scripts/features/verifyReviewWorkflow.mjs](../../../scripts/features/verifyReviewWorkflow.mjs) | [scripts/features/verifyReviewBrowser.mjs](../../../scripts/features/verifyReviewBrowser.mjs) | [R14-prediction-review-analytics-ui/R14-08-review-facts-and-package.md](../R14-prediction-review-analytics-ui/R14-08-review-facts-and-package.md) |
| postmatch | [scripts/features/verifyPostmatchWorkflow.mjs](../../../scripts/features/verifyPostmatchWorkflow.mjs) | [scripts/features/verifyPostmatchBrowser.mjs](../../../scripts/features/verifyPostmatchBrowser.mjs) | [R14-prediction-review-analytics-ui/R14-12-postmatch-settlement-and-monitoring.md](../R14-prediction-review-analytics-ui/R14-12-postmatch-settlement-and-monitoring.md) |
| analytics | [scripts/features/verifyAnalyticsWorkflow.mjs](../../../scripts/features/verifyAnalyticsWorkflow.mjs) | [scripts/features/verifyAnalyticsBrowser.mjs](../../../scripts/features/verifyAnalyticsBrowser.mjs) | [R14-prediction-review-analytics-ui/R14-13-analysis-jobs-and-packages.md](../R14-prediction-review-analytics-ui/R14-13-analysis-jobs-and-packages.md) |
| candidates | [scripts/features/verifyCandidatesWorkflow.mjs](../../../scripts/features/verifyCandidatesWorkflow.mjs) | [scripts/features/verifyCandidatesBrowser.mjs](../../../scripts/features/verifyCandidatesBrowser.mjs) | [R14-prediction-review-analytics-ui/R14-16-candidate-decisions-and-lifecycle.md](../R14-prediction-review-analytics-ui/R14-16-candidate-decisions-and-lifecycle.md) |
| release | [scripts/features/verifyReleaseWorkflow.mjs](../../../scripts/features/verifyReleaseWorkflow.mjs) | [scripts/features/verifyReleaseBrowser.mjs](../../../scripts/features/verifyReleaseBrowser.mjs) | [R14-prediction-review-analytics-ui/R14-17-acceptance-and-reports.md](../R14-prediction-review-analytics-ui/R14-17-acceptance-and-reports.md) |
| ai | [scripts/features/verifyAiWorkspaceWorkflow.mjs](../../../scripts/features/verifyAiWorkspaceWorkflow.mjs) | [scripts/features/verifyAiWorkspaceBrowser.mjs](../../../scripts/features/verifyAiWorkspaceBrowser.mjs) | [R15-ai-profiles-logs-ui/R15-01-session-draft-request-lifecycle.md](../R15-ai-profiles-logs-ui/R15-01-session-draft-request-lifecycle.md) |
| profile | [scripts/features/verifyProfileWorkflow.mjs](../../../scripts/features/verifyProfileWorkflow.mjs) | [scripts/features/verifyProfileBrowser.mjs](../../../scripts/features/verifyProfileBrowser.mjs) | [R15-ai-profiles-logs-ui/R15-07-profiles-parser-credentials-tests.md](../R15-ai-profiles-logs-ui/R15-07-profiles-parser-credentials-tests.md) |
| logs | [scripts/features/verifyLogsWorkflow.mjs](../../../scripts/features/verifyLogsWorkflow.mjs) | [scripts/features/verifyLogsBrowser.mjs](../../../scripts/features/verifyLogsBrowser.mjs) | [R15-ai-profiles-logs-ui/R15-12-logs-diagnostics.md](../R15-ai-profiles-logs-ui/R15-12-logs-diagnostics.md) |

这些测试通过loadTeamsProduction或原生产main/Shell/Router/Session/Workspace执行实际模块，传输/文件/API响应是明确QA边界；不是在测试里重写状态机。共有函数被多个入口调用时，不把日志重复输出相加成新的独立覆盖数。源码标签表达式也不是实际执行次数。

## 17页状态/尺寸/主题分配

| 页面 | 当前生产owner | 测试组 | 必需状态与实际范围 | 真实链路/设计范围边界 |
| --- | --- | --- | --- | --- |
| dashboard | [src/features/dashboard/dashboardView.ts](../../../src/features/dashboard/dashboardView.ts) | readonly | G几何；5 primary actions, service metrics, refresh; business unknown/empty remains explicit | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| database | [src/features/database/databaseView.ts](../../../src/features/database/databaseView.ts) | database | G几何；overview, identity, connection/reset failure and stale confirmation; no real reset | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| openai | [src/features/openai-profiles/ProfilesFeature.ts](../../../src/features/openai-profiles/ProfilesFeature.ts) | profile | G几何；directory/editor/example, credential intent, stale/unknown/partial failure and test versions | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| api_workspace | [src/features/ai-workspace/aiWorkspaceFeature.ts](../../../src/features/ai-workspace/aiWorkspaceFeature.ts) | ai | G几何；session/draft/context/composer, loading/empty/error, send/unknown/cancel/archive/ABA | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| workbooks | [src/pages/workbooks.ts](../../../src/pages/workbooks.ts) | workbooks | G几何；three categories, native-file transport boundary, preflight/conflict/unknown/recovery | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| lineup_presets | [src/pages/lineupPresets.ts](../../../src/pages/lineupPresets.ts) | lineups | G几何；empty/catalogue/editor, 6-column members, archive/apply/copy isolation and failure | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| rules | [src/features/rules/rulesView.ts](../../../src/features/rules/rulesView.ts) | rules | G几何；scope/region/competition grid, fixed filters, result-only scroll, structure/routing/rules | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| players | [src/features/players/playerDirectoryView.ts](../../../src/features/players/playerDirectoryView.ts) | players | G几何；18-row directory, paging/loading focus, selected profile, source context and drafts | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| teams | [src/features/teams/teamsView.ts](../../../src/features/teams/teamsView.ts) | teams | G几何；directory, filter, pagination, selection, roster/quick-view/profile, source return/focus | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| lineups | [src/pages/lineups.ts](../../../src/pages/lineups.ts) | lineups | G几何；match selection/editor, paired members, snapshots/locks/history, preview and restoration | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| prediction | [src/pages/prediction.ts](../../../src/pages/prediction.ts) | prediction / research / runs | G几何；context/permission, formal/shadow, unknown/readback, actual zero and missing fields; P4 task/evidence/conflict/freeze and stored-history state | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| review | [src/features/review/reviewPage.ts](../../../src/features/review/reviewPage.ts) | review / postmatch | G几何；loaded 9-step workflow, facts, package/preflight/confirm, unknown and result readback; settlement/H-monitor readiness, exact request recovery and stale confirmation | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| analytics | [src/features/analytics/analyticsFeature.ts](../../../src/features/analytics/analyticsFeature.ts) | analytics / candidates | G几何；overview/jobs/retry/cancel, packages/suggestions, candidate decision/shadow/promote/rollback | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| runs | [src/features/runs/runWorkspaceView.ts](../../../src/features/runs/runWorkspaceView.ts) | runs | G几何；populated/missing/loading/failure, exact detail, cancel/hide/unknown/readback and lineage | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| logs | [src/features/logs/LogsFeature.ts](../../../src/features/logs/LogsFeature.ts) | logs | G几何；loaded/levels/repeat/summary/detail, loading/empty/read failure, export/clear unknown and redaction | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| architecture | [src/features/architecture/architectureView.ts](../../../src/features/architecture/architectureView.ts) | readonly | G几何；four layers, tab/anchor navigation, capability boundary, no installed-provider claim | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |
| release | [src/features/release/releaseFeature.ts](../../../src/features/release/releaseFeature.ts) | release | G几何；request draft/running/warning/history/missing/unknown, exact report identity and readback | 原记录限定状态；外部数据/API/原生命令专项仍R17-06，无未分配的新增设计项 |

## 原40业务ID逐项映射

每行保留原独立义务；相同交付owner不等于该行真实环境全部通过。行为/挂载证据由上表组展开，实际写入/服务结果按BA单列。

| 原ID / 能力 | 页面 | 业务交付owner | 实际测试组 | 当前判定 |
| --- | --- | --- | --- | --- |
| R13-01 Dashboard | dashboard | R13-01 | readonly | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R13-02 Database | database | R13-02 | database | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R13-03 Competitions | rules | R13-03 | rules | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R13-04 Rules | rules | R13-03 | rules | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R13-05 Teams | teams | R13-05 | teams | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R13-06 Players | players | R13-06 | players | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R13-07 Lineup Presets | lineup_presets | R13-07 | lineups | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R13-08 Lineups | lineups | R13-07 | lineups | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R13-09 Workbooks | workbooks | R13-09 | workbooks | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R13-10 Architecture Page | architecture | R13-01 | readonly | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-01 Prediction Context | prediction | R14-01 | prediction | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-02 Readiness 与 Fingerprint | prediction | R14-01 | prediction | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-03 Formal Run | prediction | R14-01 | prediction | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-04 Shadow Run | prediction | R14-01 | prediction | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-05 Prediction Result | prediction | R14-01 | prediction | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-06 P4 Research UI | prediction | R14-06 | research | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-07 Runs | runs | R14-07 | runs | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-08 Review Selection 与 Workflow | review | R14-08 | review | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-09 Review Facts | review | R14-08 | review | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-10 Review Package | review | R14-08 | review | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-11 Review Results | review | R14-08 | review | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-12 Postmatch | review | R14-12 | postmatch | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-13 Analytics Overview | analytics | R14-13 | analytics | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-14 Analytics Jobs | analytics | R14-13 | analytics | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-15 Analysis Packages 与 Suggestions | analytics | R14-13 | analytics | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-16 Ability 与 Parameter Candidates | analytics | R14-16 | candidates | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R14-17 Release | release | R14-17 | release | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-01 AI Session Directory | api_workspace | R15-01 | ai | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-02 Conversation | api_workspace | R15-01 | ai | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-03 Composer | api_workspace | R15-01 | ai | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-04 Pending Request | api_workspace | R15-01 | ai | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-05 Request Cancellation | api_workspace | R15-01 | ai | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-06 Context Picker | api_workspace | R15-01 | ai | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-07 OpenAI Profile Directory | openai | R15-07 | profile | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-08 OpenAI Profile Editor | openai | R15-07 | profile | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-09 API Example Parser | openai | R15-07 | profile | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-10 Credential Actions | openai | R15-07 | profile | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-11 Profile Test | openai | R15-07 | profile | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-12 Logs | logs | R15-12 | logs | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |
| R15-13 Diagnostics Bridge | logs | R15-12 | logs | 保留原交付，当前生产行为/挂载通过；外部边界不核销 |

## 全部16条BA承接

| BA | 当前本地覆盖 | 必须承接的真实结果/最终候选 | target/入口与接收者 |
| --- | --- | --- | --- |
| BA-01 | readonly/database/rules [scripts/architecture/verifyModuleBoundaries.mjs](../../../scripts/architecture/verifyModuleBoundaries.mjs) [scripts/verify-command-contract.mjs](../../../scripts/verify-command-contract.mjs) [scripts/verify_command_contract.mjs](../../../scripts/verify_command_contract.mjs) [scripts/verify_database_baseline.mjs](../../../scripts/verify_database_baseline.mjs) | current static applicable checks; not DB acceptance | 实际API/系统/最终候选专项；as scoped above; final R18 candidate separately |
| BA-02 | lineups/research/review  | OPEN four named ignored tests require real authorized DB | postgres_integration；R17-06 |
| BA-03 | lineups  | implementation/controlled behavior retained; protected deletion and rollback OPEN | match_catalog_repository_contract, model_run_identity_repository_contract；R17-06 |
| BA-04 | lineups/workbooks  | A8 12 insert/1 end, batch audit and rollback OPEN | postgres_integration；R17-06 |
| BA-05 | teams/players  | real identity/reference/concurrent deletion OPEN | entity_matching_references_repository_contract, entity_deletion_repository_contract, entity_permanent_delete_repository_contract, team_force_delete_repository_contract；R17-06 |
| BA-06 | lineups  | real lock competition, history cutoff and rollback OPEN | match_catalog_repository_contract, postgres_integration；R17-06 |
| BA-07 | teams/players/workbooks/review  | valid XLSX to DB readback/batch recovery OPEN | postgres_integration；R17-06 |
| BA-08 | prediction/research/runs  | real run/snapshot transaction, concurrency and fault readback OPEN | postgres_integration, model_run_identity_repository_contract；R17-06 |
| BA-09 | ai  | production ABA/draft bug fixed in R15-01; current controlled regression; real scenarios belong BA-10/11 | 实际API/系统/最终候选专项；as scoped above; final R18 candidate separately |
| BA-10 | ai  | real messages/attempts/API/cancellation and partial writes OPEN | postgres_integration；R17-06 |
| BA-11 | ai/profile/logs  | authorized API, Windows credential save/delete/fault and real secret boundary OPEN | 实际API/系统/最终候选专项；R17-06 |
| BA-12 | review/postmatch/analytics/candidates/release  | real ledgers/concurrency/immutable reports and known missing B acceptance contract OPEN | postgres_integration；R17-06 |
| BA-13 | prediction/research/candidates  | private P4/P7 authorized assets and original fixed regression OPEN; public stub not evidence | 实际API/系统/最终候选专项；R17-06 |
| BA-14 | readonly/database/rules/teams/players/lineups/workbooks/prediction/research/runs/review/postmatch/analytics/candidates/release/ai/profile/logs  | all mandatory targets and actual Windows Full + install OPEN; 2-target runner is incomplete for this ledger | 全部20 target见下表；R17-06 |
| BA-15 | prediction/runs [scripts/architecture/verifyModuleBoundaries.mjs](../../../scripts/architecture/verifyModuleBoundaries.mjs) [scripts/architecture/verifyStateOwnership.mjs](../../../scripts/architecture/verifyStateOwnership.mjs) | actual graph and zero transition edges verified locally, not real external evidence | 实际API/系统/最终候选专项；as scoped above; final R18 candidate separately |
| BA-16 | readonly/database/rules/teams/players/lineups/workbooks/prediction/research/runs/review/postmatch/analytics/candidates/release/ai/profile/logs  | R12 remains ended; final candidate exact-source revalidation is R18, current artifacts are not final release | 实际API/系统/最终候选专项；as scoped above; final R18 candidate separately |

BA-02不得整组关闭，以下4个postgres_integration测试均仍为ignored，必须分别记录授权库、实际命令/退出码、写入/回滚读回：

- match_lineup_chain_versions_model_selection_and_freeze_gate_are_consistent
- match_scope_inference_and_lineup_pair_transaction_are_atomic
- structured_match_events_are_queryable_and_revision_aware
- p4_stage_c_writes_are_idempotent_and_frozen_history_is_immutable

## Cargo确认的20个PostgreSQL target

普通cargo test只编译并报告这些40项ignored，CI绿色不表示真实PG通过。下列清单由实际Cargo metadata与源码测试名同时确认。执行前须按R17-06核验授权测试库；推荐逐target、单测试线程，避免并行清空/写入竞争。统一命令为 cargo test --locked -p football-persistence-postgres --test <target> -- --ignored --nocapture --test-threads=1。

| 实际target/源码 | ignored数量 | 现有Full自动选择 | 必需状态/接收者 |
| --- | --- | --- | --- |
| [crates/persistence-postgres/tests/coach_formation_repository_contract.rs](../../../crates/persistence-postgres/tests/coach_formation_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/competition_bindings_repository_contract.rs](../../../crates/persistence-postgres/tests/competition_bindings_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/competition_hierarchy_repository_contract.rs](../../../crates/persistence-postgres/tests/competition_hierarchy_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/competitions_repository_contract.rs](../../../crates/persistence-postgres/tests/competitions_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/entity_deletion_repository_contract.rs](../../../crates/persistence-postgres/tests/entity_deletion_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/entity_matching_references_repository_contract.rs](../../../crates/persistence-postgres/tests/entity_matching_references_repository_contract.rs) | 2 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/entity_permanent_delete_repository_contract.rs](../../../crates/persistence-postgres/tests/entity_permanent_delete_repository_contract.rs) | 2 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/global_name_search_repository_contract.rs](../../../crates/persistence-postgres/tests/global_name_search_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/match_catalog_repository_contract.rs](../../../crates/persistence-postgres/tests/match_catalog_repository_contract.rs) | 2 | 是 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/model_run_identity_repository_contract.rs](../../../crates/persistence-postgres/tests/model_run_identity_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/player_abilities_dynamic_tags_repository_contract.rs](../../../crates/persistence-postgres/tests/player_abilities_dynamic_tags_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/player_directory_detail_repository_contract.rs](../../../crates/persistence-postgres/tests/player_directory_detail_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/player_names_positions_repository_contract.rs](../../../crates/persistence-postgres/tests/player_names_positions_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/player_team_periods_availability_repository_contract.rs](../../../crates/persistence-postgres/tests/player_team_periods_availability_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/postgres_integration.rs](../../../crates/persistence-postgres/tests/postgres_integration.rs) | 18 | 是 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/route_resolution_repository_contract.rs](../../../crates/persistence-postgres/tests/route_resolution_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/rule_package_repository_contract.rs](../../../crates/persistence-postgres/tests/rule_package_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/team_directory_detail_repository_contract.rs](../../../crates/persistence-postgres/tests/team_directory_detail_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/team_force_delete_repository_contract.rs](../../../crates/persistence-postgres/tests/team_force_delete_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |
| [crates/persistence-postgres/tests/team_names_profiles_repository_contract.rs](../../../crates/persistence-postgres/tests/team_names_profiles_repository_contract.rs) | 1 | 否，需另行明确执行 | OPEN / R17-06 |

另有src-tauri/src/bootstrap/native_log_acceptance.rs宿主，受all(test, windows, football_native_log_acceptance)限定；它不在普通cargo test的40 PG ignored中。原R15/R16的隔离原生操作与startup有单独证据，不是Full或生产用户凭据验证。

## 171命令与旧路径审计

verify-command-contract比较真实前端/注册/定义三方集合和重复；verify_command_contract另外检查机器契约、唯一owner、分组/顺序/hash及允许动态参数。两者均保留且接入前端，不按文件名相似删检查。command-shapes.json索引171个真实Rust声明和前端参数表达式；它不等于171条命令的真实调用或所有JSON字段序列化实证，TS/Rust类型、命令静态与真实边界分别记录。

原apiWorkspace/openai路径在旧制品契约选择器中已转换为精确当前owner及真实文件断言，不是read失败后的空跳过；层级测试断言旧文件退出；本批仅在隔离坏契约中使用退役路径，要求原检查明确失败。明细见coverage-audit.json.sourceOwnerAudit。

原截图与已批准设计不改；不能从上述覆盖表继承“全部设计/真实业务通过”。R17-06未闭合前，R17阶段与R18发布继续受阻。
