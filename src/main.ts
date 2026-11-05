import {CandidateActions} from "./features/analytics/candidates/candidateActions";import {CandidateControls} from "./features/analytics/candidates/candidateControls";
import {AnalyticsFeature,AnalyticsActions,analyticsStorage} from "./features/analytics";
import {PostmatchFeature,postmatchRecovery,settlementTarget,PostmatchActions,PostmatchControls} from "./features/postmatch";
import { ResearchFeature, researchRecovery, researchSnapshotView } from "./features/prediction";
import {PredictionFeature,predictionRecovery,predictionPage,inlinePredictionResult,predictionDetailBody,routeSourceLabel,modelDisplayName,syncPredictionControls,syncPredictionSelectIcons} from "./features/prediction";
import {
api,
issueWasLogged,
recordUiOperation,
recordUiOperationFailure,
} from "./api/client";
import { ContextMenuAdapter } from "./app/context-menu";
import { EventScope,MountHandle } from "./app/lifecycle";
import { matchReviewWorkflowAllows } from "./app/matchReviewWorkflow";
import { ModalController,type ModalConfirmationOptions } from "./app/modal";
import { NavigationContext,PRIMARY_NAVIGATION,Router,isRoute,type NavigationRequest } from "./app/router";
import { AppSession } from "./app/session";
import { ApplicationShellController,pageTitle,renderShell } from "./app/shell";
import { WarningCenterController } from "./app/warnings";
import { WorkspaceCoordinator,type WorkspaceLayoutMode,type WorkspaceTabState } from "./app/workspace";
import { bulkArchiveSummary } from "./components/entityDeletion";
import { escapeHtml } from "./components/format";
import { enhanceSearchableSelects,refreshSearchableSelects } from "./components/searchableSelect";
import {

fetchLineups,




type LineupsLoadResult,


} from "./controllers/pageLoaders";
import { bindSearchableSelectDiagnostics } from "./diagnostics/searchableSelectDiagnostics";
import { activateSectionPanel } from "./ui/navigation";
import { architecturePage } from "./features/architecture";
import { CompetitionFeature,fitCatalogueRegions,selectCatalogueRegion,clearRulesCatalogueFilters,filterRulesCompetitionCatalogue,selectRulesDirectoryLevel } from "./features/competitions";
import { dashboardPage } from "./features/dashboard";
import { DatabaseFeature,databasePage,databaseTechnical } from "./features/database";
import { createLineupsFeature } from "./features/lineups";
import type { LineupSide } from "./features/lineups/lineupContext";
import { emptyPairedLineupSide } from "./features/lineups/lineupState";
import { PlayersFeature,playersPage } from "./features/players";
import { RulesFeature,routeDetailBody,rulesPage } from "./features/rules";
import { TeamsFeature,teamsPage } from "./features/teams";
import { analyticsPage } from "./pages/analytics";
import {AiWorkspaceFeature,AiWorkspaceActions,AiWorkspaceControls,aiWorkspaceStorage,aiWorkspacePage} from "./features/ai-workspace";
import { lineupPresetsPage } from "./pages/lineupPresets";
import { lineupsPage } from "./pages/lineups";
import {LogsFeature,LogActions,logsPage,logStorage} from "./features/logs";
import {createClientIssueRecorder,userFacingError} from "./platform/diagnostics";
import {ProfilesFeature,ProfileActions,ProfileControls,profilePage,profileStorage} from "./features/openai-profiles";
import {ReleaseDetails} from "./features/release/releaseDetails";
import {ReleaseControls} from "./features/release/releaseControls";
import {ReleaseFeature,ReleaseActions,releaseStorage,releaseFeaturePage} from "./features/release";
import {ReviewFeature,ReviewFacts,ReviewControls,reviewRecovery,reviewDatabaseIdentity,reviewPage,type ReviewIntent} from "./features/review";
import {RunsFeature,runRecovery,runsPage,runTechnicalBody,runHistoryMarkup} from "./features/runs";
import { WorkbooksFeature,workbooksPage } from "./features/workbooks";
import { importRecovery } from "./app/imports/importRecovery";
import { clipboard } from "./platform/clipboard";
import { localPreferences,preferenceKey,sessionPreferences } from "./platform/storage";
import { stopTauriTransport } from "./platform/tauri";
import "./styles/index.css";
import type {
CoachListItem,
CompetitionKind,
FormationRecord,
LineupSnapshotType,
MatchLineupChain,
Page,
PlayerListQuery,
PlayerNavigationContext,
PredictionCommand,
PredictionExecution,
PredictionModelFamily,

RouteDecision,
RoutePreviewCommand,
TeamListQuery
} from "./types";
import { KeyboardShortcutController,motionScrollBehavior } from "./ui/accessibility";
import { focusValidationTarget } from "./ui/forms/validation-summary";
import { CommandMenuController } from "./ui/overlays/command-menu";

const browserLifecycleController = new EventScope();

const DATABASE_RESET_COMPLETE_KEY = preferenceKey("database-reset-complete");
const TEAM_QUERY_KEY = preferenceKey("team-query");
const PLAYER_QUERY_KEY = preferenceKey("player-query");
const PLAYER_NAV_CONTEXT_KEY = preferenceKey("player-navigation-context");

let app: HTMLDivElement;

const recordClientIssue=createClientIssueRecorder({wasLogged:issueWasLogged,record:draft=>api.recordIssue(draft),critical:(operation,message)=>warningCenter?.record({key:'critical:'+operation,source:'客户端',severity:'critical',title:operation,message,target:'logs'})});

const session = new AppSession({
  loadBootstrap: () => api.bootstrap(),
  loadRecentRuns: () => api.listRecentRuns(100),
  applyTheme(theme) {
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
  },
});
const databaseFeature = new DatabaseFeature({
  snapshot: () => session.bootstrap,
  configure: options => api.configureDatabase(options),
  disconnect: () => api.disconnectDatabase(),
  reset: confirmation => api.resetDatabase(confirmation),
  refresh: () => session.refresh(),
  busy: operation => runBusy(operation),
  render: () => render(),
  notify: message => toast(message, "success"),
  confirm: (...args) => modal.confirm(...args),
  invalidateConfirmation: () => modal.close(),
  async completeReset() {
    await workspaceState.clearAll();
    workspaceState.stop();
    localPreferences.removeItem(TEAM_QUERY_KEY);
    localPreferences.removeItem(PLAYER_QUERY_KEY);
    localPreferences.removeItem(PLAYER_NAV_CONTEXT_KEY);
    sessionPreferences.setItem(DATABASE_RESET_COMPLETE_KEY, "1");
    window.location.reload();
  },
});
const rulesFeature = new RulesFeature({
  snapshot: () => session.bootstrap,
  registerRulePackage: draft => api.registerRulePackage(draft),
  createBinding: draft => api.createCompetitionBinding(draft),
  previewRoute: command => api.previewRoute(command),
  refresh: () => session.refresh(),
  busy: operation => runBusy(operation),
  render: () => render({ preserveForm: true }),
  active: () => !browserLifecycleController.signal.aborted && navigation.page === "rules",
  notify: message => toast(message, "success"),
});
const competitionFeature = new CompetitionFeature({
  snapshot: () => session.bootstrap,
  createCompetition: draft => api.createCompetition(draft),
  createSeason: draft => api.createSeason(draft),
  createStage: draft => api.createStage(draft),
  createRound: draft => api.createRound(draft),
  deleteCompetition: id => api.deleteCompetition(id),
  refresh: () => session.refresh(),
  busy: operation => runBusy(operation),
  render: () => render({ preserveForm: true }),
  active: () => !browserLifecycleController.signal.aborted && navigation.page === "rules",
  invalidateRoutes: () => rulesFeature.invalidatePreview(),
  notify: message => toast(message, "success"),
  confirm: (...args) => modal.confirm(...args),
  showPath: body => showHtmlModal("赛事规则与模型路径", "目录级说明", body),
});

const teamsFeature = new TeamsFeature({
  importRecovery: importRecovery(localPreferences,preferenceKey("team-import-recovery"),()=>session.bootstrap),
  captureImportView: ()=>captureImportView(),
  captureImportData: ()=>captureImportData(),

  api,
  initialQuery: localPreferences.readObject<TeamListQuery>(TEAM_QUERY_KEY),
  snapshot: () => session.bootstrap,
  page: () => navigation.page,
  root: () => app,
  busy: operation => runBusy(operation),
  persistQuery: query => localPreferences.writeJson(TEAM_QUERY_KEY, query),
  publishOptions(coaches, formations) { coachList = coaches; formationCatalog = formations; },
  publishCoaches: coaches => { coachList = coaches; },
  refreshPlayerReferences: async () => { const snapshot = session.bootstrap; const references = await api.playerCatalogReferenceData(); if (snapshot === session.bootstrap) playersFeature.catalog.setReferences(references); },
  refreshAll: () => refreshBootstrapAndCatalog(),
  clearPlayerInspector: () => { playersFeature.catalog.adopt(null); },
  render: () => render({ preserveForm: true }),
  notify: (message, kind) => toast(message, kind),
  setSection(section, inspectorCollapsed) { workspaceState.patchModule("teams", { active_section: section, ...(inspectorCollapsed !== undefined ? { inspector_collapsed: inspectorCollapsed } : {}), ...(section === "directory" && !teamsFeature.catalog.detail ? { active_tab_id: null } : {}) }); },
  appendTab: (id, label) => appendWorkspaceTab("teams", { id, label }),
  activeTab: page => workspaceState.module(page).active_tab_id,
  setPresetTab: id => workspaceState.patchModule("lineup_presets", { active_tab_id: id }),
  navigateTeams: () => navigateTo("teams"),
  confirm: (...args) => modal.confirm(...args),
  showHtml: (...args) => showHtmlModal(...args),
  closePanel: () => closeModal(),
  async afterDelete(result, missing) {
    const page = navigation.page, activeId = removeWorkspaceObjects("teams", [...missing, ...result.deleted_ids]), revision = teamsFeature.catalog.revision;
    await session.refresh();
    if (page !== navigation.page || revision !== teamsFeature.catalog.revision) return false;
    if (!await teamsFeature.catalog.load({ resetCursor: true, retainId: activeId, isCurrent: () => navigation.page === page })) return false;
    const current = teamsFeature.catalog.capture(), references = await api.playerCatalogReferenceData();
    if (!current() || page !== navigation.page) return false;
    playersFeature.catalog.setReferences(references); persistWorkspaceSelection("teams"); render({ preserveForm: true }); return true;
  },
  async removeMissing(ids) {
    const page = navigation.page, activeId = removeWorkspaceObjects("teams", ids), revision = teamsFeature.catalog.revision;
    await session.refresh(); if (page !== navigation.page || revision !== teamsFeature.catalog.revision) return;
    if (await teamsFeature.catalog.load({ resetCursor: true, retainId: activeId, isCurrent: () => navigation.page === page })) { persistWorkspaceSelection("teams"); render({ preserveForm: true }); }
  },
  async afterForceDelete(result) {
    const page = navigation.page, activeId = removeWorkspaceObjects("teams", [result.team_id]);
    removeWorkspaceObjects("players", result.deleted_player_ids); playersFeature.imports.clear(); lineupsFeature.imports.clear();
    const revision = teamsFeature.catalog.revision;
    await session.refresh(); if (page !== navigation.page || revision !== teamsFeature.catalog.revision) return false;
    if (!await teamsFeature.catalog.load({ resetCursor: true, retainId: activeId, isCurrent: () => navigation.page === page })) return false;
    const current = teamsFeature.catalog.capture();
    if (!await playersFeature.load(true, () => current() && page === navigation.page)) return false;
    persistWorkspaceSelection("teams"); persistWorkspaceSelection("players"); render({ preserveForm: true }); return true;
  },
  async afterArchive(result) {
    const page = navigation.page, activeId = removeWorkspaceObjects("teams", [...result.archived_ids, ...result.already_archived_ids]), revision = teamsFeature.catalog.revision;
    await session.refresh(); if (page !== navigation.page || revision !== teamsFeature.catalog.revision) return;
    if (!await teamsFeature.catalog.load({ resetCursor: true, retainId: activeId, isCurrent: () => navigation.page === page })) return;
    const current = teamsFeature.catalog.capture(), references = await api.playerCatalogReferenceData();
    if (!current() || page !== navigation.page) return;
    playersFeature.catalog.setReferences(references); persistWorkspaceSelection("teams"); render({ preserveForm: true });
    showHtmlModal("球队批量归档结果", "保留历史关系", bulkArchiveSummary(result, "支球队"));
  },
});


const playersFeature = new PlayersFeature({
  importRecovery: importRecovery(localPreferences,preferenceKey("player-import-recovery"),()=>session.bootstrap),
  captureImportView: ()=>captureImportView(),
  captureImportData: ()=>captureImportData(),

 api,initialQuery:localPreferences.readObject<PlayerListQuery>(PLAYER_QUERY_KEY),initialSource:localPreferences.readObject<PlayerNavigationContext>(PLAYER_NAV_CONTEXT_KEY),snapshot:()=>session.bootstrap,page:()=>navigation.page,root:()=>app,
 busy:operation=>runBusy(operation),persistQuery:query=>localPreferences.writeJson(PLAYER_QUERY_KEY,query),persistSource:source=>{if(source)localPreferences.writeJson(PLAYER_NAV_CONTEXT_KEY,source);else localPreferences.removeItem(PLAYER_NAV_CONTEXT_KEY);},
 render:()=>render({preserveForm:true}),notify:(message,kind)=>toast(message,kind),section:(active_section,inspector_collapsed)=>workspaceState.patchModule("players",{active_section,...(inspector_collapsed!==undefined?{inspector_collapsed}:{})}),appendTab:(id,label)=>appendWorkspaceTab("players",{id,label}),refreshAll:()=>refreshBootstrapAndCatalog(),confirm:(...args)=>modal.confirm(...args),showHtml:(...args)=>showHtmlModal(...args),showResult:(title,result)=>showModal(title,result),
 async afterMutation(ids){const page=navigation.page,active=removeWorkspaceObjects("players",ids),revision=playersFeature.catalog.revision;await session.refresh();if(page!==navigation.page||revision!==playersFeature.catalog.revision)return false;
 if(!await playersFeature.load(true,()=>page===navigation.page))return false;const current=playersFeature.catalog.capture();
 if(teamsFeature.catalog.list!==null){await teamsFeature.load(true);if(!current()||page!==navigation.page)return false;}
 persistWorkspaceSelection("players");if(active&&playersFeature.catalog.detail?.player.id===active)appendWorkspaceTab("players",{id:active,label:playersFeature.catalog.detail.player.canonical_name});render({preserveForm:true});return true;}
});

const navigation = new Router();
const workflowNavigation = new NavigationContext();
let coachList: CoachListItem[] = [];
let formationCatalog: FormationRecord[] = [];
const postmatchFeature=new PostmatchFeature({
 api,scope:()=>session.captureDatabase(),view:()=>{const page=navigation.page;return()=>page===navigation.page&&!browserLifecycleController.signal.aborted;},
 recovery:postmatchRecovery(localPreferences,preferenceKey("postmatch-request-recovery"),()=>reviewDatabaseIdentity(session.bootstrap)),formatError:userFacingError,
 changed:()=>{if((renderedPage==="review"||renderedPage==="analytics")&&session.bootstrap&&!browserLifecycleController.signal.aborted)render({preserveForm:true});},
});
const postmatchControls=new PostmatchControls();
const analyticsFeature=new AnalyticsFeature({
 api,scope:()=>session.captureDatabase(),view:()=>{const page=navigation.page;return()=>page===navigation.page&&!browserLifecycleController.signal.aborted;},
 storage:analyticsStorage(localPreferences,preferenceKey("analytics-request-state"),()=>reviewDatabaseIdentity(session.bootstrap)),formatError:userFacingError,
 historyCount:()=>postmatchFeature.overview?.settlement_count??null,initialStep:()=>workspaceState.module("analytics").active_section,
 saveStep:step=>workspaceState.patchModule("analytics",{active_section:"analysis-"+step+"-step"}),
 changed:()=>{if(renderedPage==="analytics"&&session.bootstrap&&!browserLifecycleController.signal.aborted)render({preserveForm:true});},
});
const analyticsActions=new AnalyticsActions(analyticsFeature,{confirm:(...args)=>showConfirmation(...args,{tone:"neutral"}),show:(title,value)=>showModal(title,value),notify:message=>toast(message,"success")});
const candidateControls=new CandidateControls();
const candidateActions=new CandidateActions(analyticsFeature.candidates,{confirm:(...args)=>showConfirmation(...args,{tone:"neutral"}),show:(...args)=>showModal(...args),notify:(message,kind)=>toast(message,kind??"success")});
const releaseFeature=new ReleaseFeature({api,scope:()=>session.captureDatabase(),view:()=>{const page=navigation.page;return()=>page===navigation.page&&!browserLifecycleController.signal.aborted;},storage:releaseStorage(localPreferences,preferenceKey("release-request-state"),()=>reviewDatabaseIdentity(session.bootstrap)),formatError:userFacingError,changed:()=>{if(renderedPage==="release"&&session.bootstrap&&!browserLifecycleController.signal.aborted)render({preserveForm:true});}});
const releaseControls=new ReleaseControls();
const aiWorkspaceFeature=new AiWorkspaceFeature({api,scope:()=>session.captureDatabase(),view:()=>{const page=navigation.page;return()=>page===navigation.page&&!browserLifecycleController.signal.aborted;},storage:aiWorkspaceStorage(localPreferences,preferenceKey("api-workspace-state"),()=>reviewDatabaseIdentity(session.bootstrap)),formatError:userFacingError,changed:()=>{if(renderedPage==="api_workspace"&&session.bootstrap&&!browserLifecycleController.signal.aborted)render({preserveForm:true});}});
const aiWorkspaceControls=new AiWorkspaceControls();
const profilesFeature=new ProfilesFeature({api,storage:profileStorage(localPreferences,preferenceKey("openai-profiles-drafts")),view:()=>{const page=navigation.page;return()=>page===navigation.page&&!browserLifecycleController.signal.aborted;},changed:()=>{if(renderedPage==="openai"&&session.bootstrap&&!browserLifecycleController.signal.aborted)render({preserveForm:true});},formatError:userFacingError});
const profileControls=new ProfileControls(profilesFeature);
const profileActions=new ProfileActions(profilesFeature,profileControls,{confirm:(...args)=>showConfirmation(...args),show:(...args)=>showModal(...args)});
const aiWorkspaceActions=new AiWorkspaceActions(aiWorkspaceFeature,{confirm:(title,description,facts,label,run)=>showConfirmation(title,description,facts.map(([a,b])=>[a,b]),label,run),show:(title,message)=>showModal(title,message),copy:value=>clipboard.writeText(value),notify:message=>toast(message,"success"),section:active_section=>workspaceState.patchModule("api_workspace",{active_section}),render:()=>render({preserveForm:true}),navigate:()=>navigateTo("api_workspace"),source:{readTeam:id=>api.readTeam(id),readPlayer:id=>api.readPlayer(id),team:()=>teamsFeature.catalog.detail,player:()=>playersFeature.catalog.detail,captureTeam:()=>{const current=teamsFeature.catalog.capture(),page=navigation.page;return()=>current()&&navigation.page===page;},capturePlayer:()=>{const current=playersFeature.catalog.capture(),page=navigation.page;return()=>current()&&navigation.page===page;}}});

const releaseDetails=new ReleaseDetails();
const releaseActions=new ReleaseActions(releaseFeature,{confirm:(...args)=>showConfirmation(...args,{tone:"neutral"}),show:(...args)=>releaseDetails.show(...args),notify:(text,tone)=>toast(text,tone)});
const reviewFeature=new ReviewFeature({postmatch:postmatchFeature.settlement,
 api,scope:()=>session.captureDatabase(),view:()=>{const page=navigation.page;return()=>page===navigation.page&&!browserLifecycleController.signal.aborted;},
 changed:()=>{if(renderedPage==="review"&&session.bootstrap&&!browserLifecycleController.signal.aborted)render({preserveForm:true});},
 recovery:reviewRecovery(localPreferences,preferenceKey("review-request-recovery"),()=>session.bootstrap),snapshot:()=>session.bootstrap,formatError:userFacingError,
});
const reviewControls=new ReviewControls();
const reviewFacts=new ReviewFacts(localPreferences,preferenceKey("review-facts"),()=>reviewDatabaseIdentity(session.bootstrap));
const postmatchActions=new PostmatchActions(postmatchFeature,{
 review(id){const detail=reviewFeature.state.detail;if(!detail||id&&id!==detail.summary.id)throw Error("请先载入并核对指定复盘");const current=reviewFeature.captureConfirmation(),version=reviewFeature.version;return {target:settlementTarget(detail),current:()=>current()&&version===reviewFeature.version&&reviewFeature.state.detail?.summary.id===detail.summary.id};},
 openSettlement(){reviewFeature.viewChanged();workspaceState.patchModule("review",{active_section:"step-8"});render({preserveForm:true});},
 async refreshReview(){await reviewFeature.load();},
 confirm:(...args)=>showConfirmation(...args,{tone:"neutral"}),showHtml:(...args)=>showHtmlModal(...args),showResult:(title,value)=>showModal(title,value),close:()=>closeModal(),notify:message=>toast(message,"success"),
});

const lineupsFeature = createLineupsFeature({
  captureImportView: ()=>captureImportView(),
  captureImportData: ()=>captureImportData(),
  importRecovery: importRecovery(localPreferences,preferenceKey("match-import-recovery"),()=>session.bootstrap),

recordUiOperation,recordUiOperationFailure,pageTitle,
get references(){return playersFeature.catalog.references;},
setReferences:value=>playersFeature.catalog.setReferences(value),
get coachList(){return coachList;}, set coachList(v){coachList=v;},
localDateTimeInputValue:(...args)=>localDateTimeInputValue(...args),
api,files:api,
render:(...args)=>render(...args),
get formationCatalog(){return formationCatalog;}, set formationCatalog(v){formationCatalog=v;},
toast:(...args)=>toast(...args),
get teamDetail(){return teamsFeature.catalog.detail;},get teamPresets(){return teamsFeature.catalog.presets;},setTeamPresets:(id,presets)=>teamsFeature.catalog.setPresets(id,presets),refreshTeam:async()=>{await teamsFeature.catalog.refreshSelected();},
showHtmlModal:(...args)=>showHtmlModal(...args),
value:(...args)=>value(...args),
nullableValue:(...args)=>nullableValue(...args),
nullableNumber:(...args)=>nullableNumber(...args),
checked:(...args)=>checked(...args),
runBusy:(...args)=>runBusy(...args),
showConfirmation:(...args)=>showConfirmation(...args),
closeModal:(...args)=>closeModal(...args),
localDateTimeToIso:(...args)=>localDateTimeToIso(...args),
get lastPredictionResult(){return predictionFeature.lastResult;}, set lastPredictionResult(v){predictionFeature.lastResult=v;},
get workspaceState(){ return workspaceState; },
session,
filterSelectOptions:(...args)=>filterSelectOptions(...args),
get app(){ return app; },
refreshBootstrapAndCatalog:(...args)=>refreshBootstrapAndCatalog(...args),
navigation,
recordClientIssue:(...args)=>recordClientIssue(...args),
userFacingError:(...args)=>userFacingError(...args),
get selectedReviewMatchId(){return reviewFeature.state.selectedMatchId;},
loadReviewLineups:(id)=>reviewFeature.refreshLineups(id),
missingLineupTeam:(...args)=>missingLineupTeam(...args)
});

const workbooksFeature = new WorkbooksFeature({
  targets: {
    team: {label:"球队月度",batch:teamsFeature.imports.monthlyBatch,preview:()=>teamsFeature.previewMonthly(),commit:ticket=>teamsFeature.commitMonthly(ticket),exportFile:template=>teamsFeature.exportFile(template?"monthly-template":"monthly-data")},
    player: {label:"球员月度",batch:playersFeature.imports,preview:()=>playersFeature.previewImport(),commit:ticket=>playersFeature.commitImport(ticket),exportFile:template=>playersFeature.exportFile(template)},
    match: {label:"比赛与阵容",batch:lineupsFeature.imports,preview:()=>lineupsFeature.actions.previewMatchImport(false),commit:ticket=>lineupsFeature.actions.commitMatchImport(ticket),exportFile:template=>template?lineupsFeature.actions.exportMatchTemplate():lineupsFeature.actions.exportMatchData(),exportContext:()=>{
      const id=nullableValue("exchange-match-id"),match=playersFeature.catalog.references?.managed_matches.find(item=>item.id===id);
      if(!match)throw new Error("请先选择当前列表中的比赛");
      return {facts:[["比赛",match.home_team_name+" vs "+match.away_team_name],["比赛 ID",match.id]],current:()=>nullableValue("exchange-match-id")===id&&!!playersFeature.catalog.references?.managed_matches.some(item=>item.id===id)};
    }},
  },
  page:()=>navigation.page,section:()=>workspaceState.module("workbooks").active_section,
  render:()=>render({preserveForm:true}),confirm:(title,description,facts,label,action)=>showConfirmation(title,description,facts,label,action,{tone:"neutral"}),
});
function captureImportData():()=>boolean {
  const target=session.bootstrap?.data.database_url;
  return ()=>!!target && session.bootstrap?.data.database_configured===true && target===session.bootstrap.data.database_url && session.state.phase!=="stopped";
}
function captureImportView():()=>boolean {
  const page=navigation.page,section=workspaceState.module(page).active_section,current=workbooksFeature.capture();
  return ()=>page===navigation.page && section===workspaceState.module(page).active_section && current();
}

const predictionFeature=new PredictionFeature({
  api,scope:()=>captureImportData(),
  view:()=>{const page=navigation.page,section=workspaceState.module(page).active_section;return()=>page===navigation.page&&section===workspaceState.module(page).active_section&&!browserLifecycleController.signal.aborted;},
  changed:()=>{if(renderedPage==="prediction"&&session.bootstrap)render({preserveForm:true});},
  busy:operation=>runBusy(operation),
  recovery:predictionRecovery(localPreferences,preferenceKey("prediction-run-recovery"),()=>session.bootstrap),
});

const researchFeature=new ResearchFeature({
  api,context:predictionFeature.context,scope:()=>{const summary=session.bootstrap,current=captureImportData();return()=>current()&&summary===session.bootstrap;},view:()=>predictionFeature.capturePresentation(),
  changed:()=>{if(renderedPage==="prediction"&&session.bootstrap)render({preserveForm:true});},
  recovery:researchRecovery(localPreferences,preferenceKey("prediction-research-recovery"),()=>session.bootstrap),
});

const runsFeature=new RunsFeature({
  api,source:{snapshot:()=>session.bootstrap,refresh:()=>session.refreshRecentRuns(),capture:()=>session.captureDatabase(),hidden:id=>session.confirmRunHidden(id)},
  view:()=>{const page=navigation.page;return()=>page===navigation.page&&!browserLifecycleController.signal.aborted;},
  changed:()=>{if((renderedPage==="runs"||renderedPage==="prediction")&&session.bootstrap&&!browserLifecycleController.signal.aborted)render({preserveForm:true});},
  formatError:userFacingError,recovery:runRecovery(localPreferences,preferenceKey("run-history-recovery"),()=>session.bootstrap),
});

interface WorkflowContinuation {
  readonly reason: string;
  readonly matchId: string | null;
  readonly snapshotType: LineupSnapshotType | null;
}

let workflowContinuation: WorkflowContinuation | null = null;


const logsFeature=new LogsFeature({api,storage:logStorage(localPreferences,preferenceKey("logs-view-recovery")),view:()=>{const page=navigation.page;return()=>page===navigation.page&&!browserLifecycleController.signal.aborted;},busy:runBusy,changed:()=>{if(renderedPage==="logs"&&session.bootstrap&&!browserLifecycleController.signal.aborted)render({preserveForm:true});}});
const logActions=new LogActions(logsFeature,{confirm:(...args)=>showConfirmation(...args),show:(...args)=>showModal(...args)});
const workspaceState = new WorkspaceCoordinator<Page>({
  read: () => api.readWorkspaceState(),
  save: (document) => api.saveWorkspaceState(document),
  clear: () => api.clearWorkspaceState(),
}, {
  onError({ operation, error }) {
    recordClientIssue(error, `工作区 / ${operation === "read" ? "读取状态" : operation === "clear" ? "清除状态" : "保存状态"}`);
    warningCenter?.record({
      key: `workspace:${operation}`, source: "工作区", severity: "warning", target: "logs",
      title: operation === "read" ? "工作区恢复失败" : operation === "clear" ? "工作区未能清除" : "工作区尚未保存",
      message: operation === "read" ? "原状态文件已保留，本次暂用临时工作区。请查看问题日志，修复后重新启动。" : "当前操作未完成，本次工作区仍保留。请查看问题日志后重试。",
    });
  },
});
const modal = new ModalController(error => { recordClientIssue(error, "确认操作");return userFacingError(error); });
const contextMenu = new ContextMenuAdapter();
const commandMenu = new CommandMenuController(document,error => { recordClientIssue(error,"快速导航");return userFacingError(error); });
const globalShortcuts = new KeyboardShortcutController(document,[{key:"k",run:openCommandMenu,allowInEditable:true}]);
function openCommandMenu(): void {
  contextMenu.close();
  commandMenu.open(PRIMARY_NAVIGATION.flatMap(module => module.items.map(item => ({id:item.page,label:item.label,description:item.description,keywords:module.label+" "+item.page,run:async()=>{
    if(item.page===navigation.page)return;
    if(item.page==="players")playersFeature.directEntry();
    await navigateTo(item.page);
  }}))));
}
let shellController: ApplicationShellController | null = null;
let warningCenter: WarningCenterController | null = null;
let renderedPage: Page | null = null;

interface RenderOptions {
  readonly preserveForm?: boolean;
}

function renderWorkflowContinuationBanner(): string {
  if (!workflowContinuation || !workflowNavigation.target || navigation.page === workflowNavigation.target.page) return "";
  return `<section class="workflow-continuation-banner" role="status"><div><span>正在补齐前置资料</span><strong>${escapeHtml(workflowContinuation.reason)}</strong><small>完成当前资料后可返回原工作流，比赛和时间窗口不会丢失。</small></div><div class="button-row"><button class="secondary" data-action="workflow-return">返回原任务</button><button class="ghost" data-action="workflow-cancel">取消返回</button></div></section>`;
}

function workflowSection(pageName: Page): string | null {
  return workspaceState.module(pageName).active_section ?? null;
}

async function startWorkflowCompletion(button: HTMLElement): Promise<void> {
  const targetPage = button.dataset.targetPage as Page | undefined;
  if (!targetPage) throw new Error("缺少补录目标页面");
  workflowNavigation.set({ page: navigation.page, section: workflowSection(navigation.page) });
  workflowContinuation = {
    reason: button.dataset.returnReason ?? "补齐缺失资料后返回",
    matchId: lineupsFeature.state.pairedLineupBuilder.match_id || lineupsFeature.state.selectedManagedMatchId,
    snapshotType: lineupsFeature.state.pairedLineupBuilder.snapshot_type,
  };
  const targetSection = button.dataset.targetSection ?? null;
  if (targetSection) workspaceState.patchModule(targetPage, { active_section: targetSection });
  await navigateTo(targetPage);
}

async function returnToWorkflow(): Promise<void> {
  const continuation = workflowContinuation;
  const returnTarget = workflowNavigation.take();
  if (!continuation || !returnTarget) return;
  workflowContinuation = null;
  if (returnTarget.section) {
    workspaceState.patchModule(returnTarget.page, {
      active_section: returnTarget.section,
    });
  }
  if (continuation.matchId) {
    lineupsFeature.state.selectedManagedMatchId = continuation.matchId;
    predictionFeature.context.matchId = continuation.matchId;
    lineupsFeature.state.pairedLineupBuilder = {
      ...lineupsFeature.state.pairedLineupBuilder,
      match_id: continuation.matchId,
      snapshot_type: continuation.snapshotType ?? lineupsFeature.state.pairedLineupBuilder.snapshot_type,
    };
  }
  await navigateTo(returnTarget.page);
  if (returnTarget.page === "lineups" && continuation.matchId) {
    lineupsFeature.actions.resetPairedBuilderForMatch(continuation.matchId, false);
    await lineupsFeature.actions.loadBothPairedLineupSides();
    render({ preserveForm: true });
  }
}

function render(options: RenderOptions = {}): void {
  if (browserLifecycleController.signal.aborted) return;
  rulesFeature.invalidatePreview();
  const shellFocus = shellController?.captureFocus() ?? null;
  if(renderedPage==="api_workspace")aiWorkspaceControls.capture(app);
  if(renderedPage==="openai")profileControls.capture(app);
  teamsFeature.drafts.capture(app);
  playersFeature.drafts.capture(app);
  if(renderedPage==="review")reviewFacts.capture(app,reviewFeature.state.selectedMatchId);
  modal.capture();
  if (options.preserveForm === true && renderedPage === "lineups") {
    lineupsFeature.actions.capturePairedLineupFromDom();
  }
  if (options.preserveForm === true && renderedPage === "review") {
    lineupsFeature.actions.captureLineupFormFromDom();
  }
  if (!session.bootstrap) {
    app.innerHTML = `<div class="boot-screen"><div class="spinner"></div><strong>正在启动平台</strong></div>`;
    renderedPage = null;
    return;
  }
  if (renderedPage) {
    const previousPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
    workspaceState.capture(renderedPage, previousPageRoot, options.preserveForm === true);
  }

  let content = "";
  switch (navigation.page) {
    case "dashboard":
      content = dashboardPage(session.bootstrap);
      break;
    case "database":
      content = databasePage(session.bootstrap, workspaceState.module("database").active_section);
      break;
    case "rules": {
      const workspace = workspaceState.module("rules");
      content = rulesPage(session.bootstrap, rulesFeature.packages.pending, workspace.active_section ?? "catalog", rulesFeature.packages.reading);
      break;
    }
    case "players": {
      const workspace = workspaceState.module("players");
      content = playersPage(
        session.bootstrap, playersFeature.catalog.references, playersFeature.catalog.list, playersFeature.catalog.detail, playersFeature.catalog.query, playersFeature.imports.preview, playersFeature.catalog.selectedIds,
        workspace.tabs, workspace.active_tab_id, workspace.layout_mode, workspace.module_sidebar_collapsed, workspace.inspector_collapsed,
        workspace.active_section ?? "directory", playersFeature.catalog.pageNumber, playersFeature.catalog.source,
      );
      break;
    }
    case "teams": {
      const workspace = workspaceState.module("teams");
      content = teamsPage(
        session.bootstrap, teamsFeature.catalog.list, teamsFeature.catalog.detail, playersFeature.catalog.detail, teamsFeature.catalog.query, teamsFeature.catalog.selectedIds, coachList, formationCatalog,
        teamsFeature.imports.package, teamsFeature.catalog.lineups, teamsFeature.catalog.presets, workspace.tabs, workspace.active_tab_id,
        workspace.layout_mode, workspace.module_sidebar_collapsed, workspace.inspector_collapsed,
        workspace.active_section ?? "directory", teamsFeature.catalog.pageNumber,
      );
      break;
    }
    case "lineup_presets": {
      content = lineupPresetsPage(
        session.bootstrap, teamsFeature.catalog.list, teamsFeature.catalog.detail, teamsFeature.catalog.presets, teamsFeature.catalog.query, teamsFeature.catalog.pageNumber,
      );
      break;
    }
    case "workbooks": {
      const workspace = workspaceState.module("workbooks");
      content = workbooksPage(session.bootstrap, teamsFeature.imports.monthly, playersFeature.imports.preview, lineupsFeature.imports.preview, workspace.module_sidebar_collapsed, workspace.inspector_collapsed, workspace.active_section ?? "team",workbooksFeature.state,playersFeature.catalog.references?.managed_matches??[],workspace.selected_object_ids[0]??"");
      break;
    }
    case "lineups": {
      const workspace = workspaceState.module("lineups");
      content = lineupsPage(
        session.bootstrap,
        playersFeature.catalog.references,
        lineupsFeature.state.lineupRecords,
        lineupsFeature.imports.preview,
        lineupsFeature.state.pairedLineupBuilder,
        lineupsFeature.state.pairedLineupPresets,
        coachList,
        lineupsFeature.state.selectedMatchLineupChain,
        lineupsFeature.state.selectedManagedMatchId,
        workspace.module_sidebar_collapsed,
        workspace.inspector_collapsed,
        workspace.active_section ?? "matches",
      );
      break;
    }
    case "prediction": {
      const workspace = workspaceState.module("prediction");
      const history = runsFeature.view;
      content = predictionPage(
        session.bootstrap,
        playersFeature.catalog.references,
        predictionFeature.context.matchId,
        researchFeature.matchWorkspace,
        researchFeature.taskWorkspace,
        lineupsFeature.state.selectedMatchLineupChain,
        predictionFeature.readiness,
        predictionFeature.context.snapshot,
        predictionFeature.context.model,
        workspace.module_sidebar_collapsed,
        workspace.inspector_collapsed,
        workspace.active_section ?? "formal",
        runHistoryMarkup(history.rows, true, [...history.locked, ...history.pending.map(receipt => receipt.runId)]),
        predictionFeature.view,
        researchFeature.view,
        history,
      );
      break;
    }
    case "review": {
      content=reviewPage(session.bootstrap,reviewFeature,reviewFacts,workspaceState.module("review").active_section,{references:playersFeature.catalog.references,players:lineupsFeature.state.lineupPlayerCandidates,builderPlayers:lineupsFeature.state.lineupBuilderPlayers,builderForm:lineupsFeature.state.lineupBuilderForm},postmatchFeature);
      break;
    }
    case "analytics":
      content = analyticsPage(session.bootstrap,analyticsFeature,postmatchFeature);
      break;
    case "runs":
      content = runsPage(session.bootstrap,runsFeature.view);
      break;
    case "logs":
      content = logsPage(logsFeature.state);
      break;
    case "openai":
      content = profilePage(profilesFeature);
      break;
    case "api_workspace":
      content = aiWorkspacePage(session.bootstrap,aiWorkspaceFeature,workspaceState.module("api_workspace").active_section ?? "chat",workspaceState.module("api_workspace").inspector_collapsed);
      break;
    case "release": {
      const workspace = workspaceState.module("release");
      content = releaseFeaturePage(session.bootstrap,releaseFeature,workspace.active_section??"overview",workspace.inspector_collapsed);
      break;
    }
    case "architecture":
      content = architecturePage(workspaceState.module("architecture").active_section);
      break;
  }
  content = renderWorkflowContinuationBanner() + content;
  reviewControls.destroy();postmatchControls.destroy();candidateControls.destroy();releaseControls.destroy();
  app.innerHTML = renderShell({
    state: session.bootstrap,
    page: navigation.page,
    theme: session.theme,
    content,
    navigationPending: navigation.pendingPage === navigation.page,
    navigationFailed: shellController?.navigationFailed ?? false,
    sidebarCollapsed: workspaceState.sidebarCollapsed(),
  });
  modal.restore();postmatchActions.sync();
  warningCenter?.observeConnection(session.bootstrap.connection_error);
  renderedPage = navigation.page;
  const currentPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
  if(navigation.page==="api_workspace"||navigation.page==="openai")currentPageRoot.dataset.workspaceCapture="suspended";
  workspaceState.restore(navigation.page, currentPageRoot);
  if(navigation.page==="analytics")candidateControls.mount(currentPageRoot);
  if(navigation.page==="release")releaseControls.mount(currentPageRoot);
  if(navigation.page==="review")reviewControls.mount(currentPageRoot);
  if(navigation.page==="review"||navigation.page==="analytics")postmatchControls.mount(currentPageRoot);
  if(navigation.page==="prediction"){
    syncPredictionControls(currentPageRoot,predictionFeature.context.command);
    enhanceSearchableSelects(currentPageRoot);
    syncPredictionSelectIcons(currentPageRoot);
  }
  if(navigation.page==="api_workspace")aiWorkspaceControls.restore(currentPageRoot);
  if(navigation.page==="openai")profileControls.restore(currentPageRoot);
  rulesFeature.syncHierarchy(currentPageRoot);
  if (navigation.page === "rules") fitCatalogueRegions(currentPageRoot);
  teamsFeature.drafts.restore(currentPageRoot, session.bootstrap);
  playersFeature.drafts.restore(currentPageRoot, session.bootstrap);
  shellController?.restoreFocus(shellFocus);
  if (navigation.page === "lineups") {
  queueMicrotask(() => {
    const matchCompetition = currentPageRoot.querySelector<HTMLSelectElement>("#new-match-competition");
    if (matchCompetition) {
      lineupsFeature.actions.updateCompetitionHierarchy("init");
      lineupsFeature.actions.autoSelectMatchSeason();
      lineupsFeature.actions.filterMatchTeamOptions();
      lineupsFeature.actions.updateFormationHierarchy("home", "init");
      lineupsFeature.actions.updateFormationHierarchy("away", "init");
    }
    enhanceSearchableSelects(currentPageRoot);
    refreshSearchableSelects(currentPageRoot);
  });
}
}

function appendWorkspaceTab(targetPage: "teams" | "players", tab: WorkspaceTabState): void {
  const current = workspaceState.module(targetPage);
  const existing = current.tabs.filter((item) => item.id !== tab.id);
  const tabs = [...existing, tab].slice(-6);
  workspaceState.patchModule(targetPage, { tabs, active_tab_id: tab.id });
}

function persistWorkspaceSelection(targetPage: "teams" | "players"): void {
  const ids = targetPage === "teams" ? [...teamsFeature.catalog.selectedIds] : [...playersFeature.catalog.selectedIds];
  workspaceState.patchModule(targetPage, { selected_object_ids: ids });
}

function removeWorkspaceObjects(
  targetPage: "teams" | "players",
  objectIds: Iterable<string>,
): string | null {
  const removed = new Set(objectIds);
  if (removed.size === 0) return workspaceState.module(targetPage).active_tab_id;
  const current = workspaceState.module(targetPage);
  const tabs = current.tabs.filter((item) => !removed.has(item.id));
  const activeId = current.active_tab_id && !removed.has(current.active_tab_id)
    ? current.active_tab_id
    : (tabs.at(-1)?.id ?? null);
  const selectedIds = current.selected_object_ids.filter((id) => !removed.has(id));
  workspaceState.patchModule(targetPage, {
    tabs,
    active_tab_id: activeId,
    selected_object_ids: selectedIds,
  });
  if (targetPage === "teams") {
    teamsFeature.catalog.remove(removed);
  } else {
    for (const id of removed) playersFeature.catalog.toggleSelection(id, false);
    if (playersFeature.catalog.detail && removed.has(playersFeature.catalog.detail.player.id)) playersFeature.catalog.adopt(null);
  }
  return activeId;
}

function isMissingWorkspaceObjectError(
  targetPage: "teams" | "players",
  error: unknown,
): boolean {
  const message = userFacingError(error);
  return targetPage === "teams"
    ? message.includes("球队不存在")
    : message.includes("球员不存在");
}

async function openAvailableWorkspaceTab(
  targetPage: "teams" | "players",
  preferredId: string | null,
): Promise<void> {
  const current = workspaceState.module(targetPage);
  const candidates = [
    preferredId,
    ...current.tabs.map((item) => item.id).reverse(),
  ].filter((id, index, values): id is string => Boolean(id) && values.indexOf(id) === index);
  for (const candidate of candidates) {
    try {
      if (targetPage === "teams") await teamsFeature.open(candidate);
      else await playersFeature.openDirectory(candidate);
      return;
    } catch (error) {
      if (!isMissingWorkspaceObjectError(targetPage, error)) throw error;
      removeWorkspaceObjects(targetPage, [candidate]);
    }
  }
  if (targetPage === "teams") {
    teamsFeature.catalog.clearDetail();
  } else {
    playersFeature.catalog.adopt(null);
  }
  render({ preserveForm: true });
}

async function activateWorkspaceTab(targetPage: "teams" | "players", objectId: string): Promise<void> {
  workspaceState.patchModule(targetPage, { active_tab_id: objectId });
  if (targetPage === "teams") await teamsFeature.open(objectId);
  else await playersFeature.openDirectory(objectId);
}

async function closeWorkspaceTab(targetPage: "teams" | "players", objectId: string): Promise<void> {
  const current = workspaceState.module(targetPage);
  const wasActive = current.active_tab_id === objectId;
  const activeId = removeWorkspaceObjects(targetPage, [objectId]);
  if (!wasActive) {
    render({ preserveForm: true });
    return;
  }
  await openAvailableWorkspaceTab(targetPage, activeId);
}

async function openSelectedWorkspaceObjects(targetPage: "teams" | "players"): Promise<void> {
  const ids = targetPage === "teams" ? [...teamsFeature.catalog.selectedIds] : [...playersFeature.catalog.selectedIds];
  const items = targetPage === "teams" ? (teamsFeature.catalog.list?.items ?? []) : (playersFeature.catalog.list?.items ?? []);
  for (const id of ids.slice(-6)) {
    const item = items.find((candidate) => candidate.id === id);
    appendWorkspaceTab(targetPage, { id, label: item?.canonical_name ?? id });
  }
  const active = ids.at(-1);
  if (!active) return;
  if (targetPage === "teams") await teamsFeature.open(active);
  else await playersFeature.openDirectory(active);
}

function setWorkspaceMode(targetPage: "teams" | "players", mode: WorkspaceLayoutMode): void {
  workspaceState.patchModule(targetPage, { layout_mode: mode });
  render({ preserveForm: true });
}

function toggleWorkspacePane(pane: "module-sidebar" | "inspector"): void {
  const current = workspaceState.module(navigation.page);
  const patch = pane === "module-sidebar"
    ? { module_sidebar_collapsed: !current.module_sidebar_collapsed }
    : { inspector_collapsed: !current.inspector_collapsed };
  workspaceState.patchModule(navigation.page, patch);
  render({ preserveForm: true });
}

function resetCurrentWorkspace(): void {
  if (navigation.pendingPage) return;
  const page = navigation.page;
  shellController?.confirmWorkspaceReset(pageTitle(page), () => {
    if (navigation.page !== page || browserLifecycleController.signal.aborted) return;
    resetWorkspaceContents();
  });
}

function resetWorkspaceContents(): void {
  modal.reset();releaseDetails.destroy();
  workspaceState.clear(navigation.page);
  if (navigation.page === "teams") { teamsFeature.reset(); playersFeature.catalog.adopt(null); }
  if (navigation.page === "players") playersFeature.reset();
  if (navigation.page === "lineup_presets") { teamsFeature.reset(); }
  // Do not capture the discarded DOM back into the just-cleared workspace.
  renderedPage = null;
  render();
  toast("当前工作区已重置", "success");
}

function toast(
  message: string,
  kind: "normal" | "success" | "error" = "normal",
): void {
  warningCenter?.notify(message, kind === "error" ? "danger" : kind === "success" ? "success" : "info", "客户端");
}

async function runBusy<T>(operation: () => Promise<T>): Promise<T> {
  if (!warningCenter) throw new Error("应用告警模块尚未启动");
  return warningCenter.runTask(operation);
}

function value(id: string): string {
  const input = document.querySelector<
    HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
  >(`#${id}`);
  if (!input) throw new Error(`缺少输入控件：${id}`);
  return input.value;
}

function nullableValue(id: string): string | null {
  const result = value(id).trim();
  return result.length > 0 ? result : null;
}

function closeModal(): void {
  modal.close();
}

function showHtmlModal(
  title: string,
  subtitle: string,
  body: string,
  footer = "",
  modalClass = "",
): void {
  modal.showHtml(title, subtitle, body, footer, modalClass);
}

function showModal(title: string, payload: unknown, footer = ""): void {
  modal.show(title, payload, footer);
}

function showConfirmation(
  title: string,
  description: string,
  facts: Array<[string, string]>,
  confirmLabel: string,
  action: () => Promise<void>,
  options: ModalConfirmationOptions = {},
): void {
  modal.confirm(title, description, facts, confirmLabel, action, options);
}

function objectValue(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function showPredictionDetail(title: string, payload: unknown): void {
  showHtmlModal(
    title,
    "比赛预测结果",
    predictionDetailBody(payload),
    "",
    "prediction-result-modal",
  );
}

function showRouteDetail(route: RouteDecision): void {
  showHtmlModal("赛事规则与模型判定", "清晰模型链路", routeDetailBody(route, { source: routeSourceLabel(route.source), model: modelDisplayName(route.model_id) }));
}

type PredictionResultMode = "formal" | "shadow" | "temporary";

function showPredictionResult(execution:PredictionExecution|Record<string,unknown>,mode:PredictionResultMode):void {
  predictionFeature.present(execution,mode);
  const target=document.querySelector<HTMLElement>("#prediction-result");
  if(target)target.innerHTML=inlinePredictionResult(predictionFeature.lastResult,mode);
  else showPredictionDetail(mode==="shadow"?"影子推演结果":mode==="formal"?"正式推演结果":"临时演练结果",predictionFeature.lastResult);
}

function syncSimpleMatchInput(): Record<string, unknown> {
  const base = session.bootstrap?.data.default_match;
  const input =
    base && typeof base === "object"
      ? (JSON.parse(JSON.stringify(base)) as Record<string, unknown>)
      : {};
  delete input.match_id;
  delete input.snapshot;
  delete input.database_match_id;
  delete input.feature_snapshot_id;
  delete input.feature_quality_score;
  delete input.preparation_version;
  delete input.data_quality;
  const kickoffRaw = nullableValue("simple-kickoff");
  const kickoff = localDateTimeToIso(kickoffRaw);
  if (!kickoff) throw new Error("请选择开球时间");
  input.kickoff_time = kickoff;
  const teamA =
    typeof input.team_a === "object" && input.team_a !== null
      ? (input.team_a as Record<string, unknown>)
      : {};
  const teamB =
    typeof input.team_b === "object" && input.team_b !== null
      ? (input.team_b as Record<string, unknown>)
      : {};
  const homeName = nullableValue("simple-home-name");
  const awayName = nullableValue("simple-away-name");
  if (!homeName || !awayName) throw new Error("请输入主队和客队名称");
  teamA.name = homeName;
  teamB.name = awayName;
  input.team_a = teamA;
  input.team_b = teamB;
  return input;
}

function matchKickoff(): string {
  const input = syncSimpleMatchInput();
  const kickoff = input.kickoff_time;
  if (typeof kickoff !== "string" || kickoff.trim().length === 0) {
    throw new Error("请选择比赛开球时间");
  }
  return kickoff;
}

function routeSelection(): Omit<RoutePreviewCommand, "kickoff_time"> {
  return {
    competition_id: nullableValue("competition-id"),
    season_id: nullableValue("season-id"),
    stage_id: nullableValue("stage-id"),
    competition_kind: value("competition-kind") as CompetitionKind,
    explicit_rule_package_id: null,
    model_family: value("simulation-model-family") as PredictionModelFamily,
  };
}

async function previewRoute(): Promise<void> {
  const command: RoutePreviewCommand = {
    kickoff_time: matchKickoff(),
    ...routeSelection(),
  };
  const routeRoot = document.querySelector("#competition-id");
  const formIdentity = () => JSON.stringify([...app.querySelectorAll<HTMLInputElement | HTMLSelectElement>("#competition-id, #season-id, #stage-id, #competition-kind, #simulation-model-family, #simple-kickoff")].map(input => [input.id, input.value]));
  const identity = formIdentity(), page = navigation.page;
  const result = await rulesFeature.resolve(command, () => routeRoot?.isConnected === true && navigation.page === page && formIdentity() === identity);
  if (result) { showRouteDetail(result); toast("模型路由预览完成；正式运行仍会重新判定", "success"); }
}

async function executePrediction(): Promise<void> {
  const command: PredictionCommand = {
    match_input: syncSimpleMatchInput(),
    snapshot_type: value("snapshot-type"),
    ...routeSelection(),
  };
  const current = predictionFeature.capturePresentation(), scope = captureImportData();
  const inputs = () => JSON.stringify([...app.querySelectorAll<HTMLInputElement | HTMLSelectElement>('[data-workspace-section="simulation"] input, [data-workspace-section="simulation"] select')].map(input => [input.id, input.value]));
  const identity = inputs();
  const result = await runBusy(() => api.executePrediction(command));
  if (!scope() || !current() || inputs() !== identity) return;
  await session.refreshRecentRuns();
  if (!scope() || !current() || inputs() !== identity) return;
  render();
  showPredictionDetail("正式推演结果", { ...result, execution_mode: "formal" });
  toast("推演完成，规则选择和结果已保存到数据库", "success");
}

async function dryRun(): Promise<void> {
  const current = predictionFeature.capturePresentation();
  const result = await runBusy(() => api.dryRunDefaultFixture());
  if (!current()) return;
  showPredictionDetail("模型自检结果", { ...result, execution_mode: "temporary" });
  toast("模型自检通过", "success");
}

function checked(id: string): boolean {
  const input = document.querySelector<HTMLInputElement>(`#${id}`);
  if (!input) throw new Error(`缺少复选框：${id}`);
  return input.checked;
}

function nullableNumber(id: string): number | null {
  const raw = value(id).trim();
  if (!raw) return null;
  const parsed = Number(raw);
  if (!Number.isFinite(parsed)) throw new Error(`${id} 必须是有效数字`);
  return parsed;
}

function localDateTimeToIso(
  raw: string | null,
  fallbackNow = false,
): string | null {
  if (!raw) return fallbackNow ? new Date().toISOString() : null;
  const parsed = new Date(raw);
  if (Number.isNaN(parsed.getTime())) throw new Error(`时间格式无效：${raw}`);
  return parsed.toISOString();
}

function localDateTimeInputValue(raw: string): string {
  const parsed = new Date(raw);
  if (Number.isNaN(parsed.getTime())) return "";
  const local = new Date(parsed.getTime() - parsed.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
}

function applyLineups(result: LineupsLoadResult): void {
  playersFeature.catalog.setReferences(result.references);
  lineupsFeature.state.lineupRecords = result.records;
  coachList = result.coaches;
  const matches = result.references.managed_matches;
  if (lineupsFeature.state.selectedManagedMatchId && !matches.some((item) => item.id === lineupsFeature.state.selectedManagedMatchId)) {
    lineupsFeature.state.selectedManagedMatchId = null;
  }
  if (lineupsFeature.state.pairedLineupBuilder.match_id && !matches.some((item) => item.id === lineupsFeature.state.pairedLineupBuilder.match_id)) {
    lineupsFeature.state.pairedLineupBuilder = {
      ...lineupsFeature.state.pairedLineupBuilder,
      match_id: "",
      home: emptyPairedLineupSide(),
      away: emptyPairedLineupSide(),
    };
  }
  lineupsFeature.state.selectedManagedMatchId ??= matches[0]?.id ?? null;
}

async function loadLineups(): Promise<void> {
  if (!session.bootstrap?.data.database_configured) return;
  applyLineups(await fetchLineups());
}
async function loadAnalysisCenter():Promise<void>{if(session.bootstrap?.data.database_configured)await Promise.all([analyticsFeature.refresh(),postmatchFeature.refresh()]);}




async function navigateTo(nextPage: Page): Promise<void> {
  logsFeature.navigationChanged();
  aiWorkspaceFeature.navigationChanged();
  profilesFeature.navigationChanged();profileControls.navigationChanged();
  workbooksFeature.changedCategory();
  predictionFeature.navigationChanged();
  runsFeature.navigationChanged();
  reviewFeature.navigationChanged();
  postmatchFeature.navigationChanged();analyticsFeature.navigationChanged();releaseFeature.navigationChanged();
  await navigation.navigate(nextPage, async request => {
    try {
      await loadNavigationPage(request);
      if (navigation.isCurrent(request)) shellController?.clearNavigationFailure();
    } catch (error) {
      if (navigation.isCurrent(request)) shellController?.showNavigationFailure();
      throw error;
    }
  }, {
    started() {
      modal.reset();releaseDetails.destroy();
      shellController?.clearNavigationFailure();
      render({ preserveForm: true });
    },
    settled() { render({ preserveForm: true }); },
  });
}

async function loadNavigationPage(request: NavigationRequest): Promise<void> {
  const nextPage = request.page;
  if (nextPage === "workbooks") {
    if (session.bootstrap?.data.database_configured) await playersFeature.catalog.refreshReferences();
    return;
  }
  // Read-only Features consume Session/current static explanation; no page-owned loading state.
  if (nextPage === "dashboard" || nextPage === "architecture") return;
  if (nextPage === "logs") {
    await logsFeature.refresh(request.isCurrent);
    return;
  }
  if (nextPage === "openai") {
    const module=workspaceState.module("openai");if(Object.hasOwn(module.controls,"openai-api-example")){const controls={...module.controls};delete controls["openai-api-example"];workspaceState.patchModule("openai",{controls});}
    await profilesFeature.load(request.isCurrent);return;
  }
  if (nextPage === "api_workspace") {
    aiWorkspaceFeature.restoreLegacyDraft(workspaceState.module("api_workspace").controls["api-workspace-message"]?.value);
    await aiWorkspaceFeature.load(request.isCurrent);
    return;
  }
  if (!session.bootstrap?.data.database_configured) return;
  if (nextPage === "release") {await releaseFeature.refresh();return;}

  if (nextPage === "players") {
    await playersFeature.load(playersFeature.catalog.list === null, () => navigation.isCurrent(request));
  } else if (nextPage === "teams" || nextPage === "lineup_presets") {
    await teamsFeature.loadForRoute(nextPage, () => navigation.isCurrent(request));
  } else if (nextPage === "lineups") {
    const result = await fetchLineups();
    if (navigation.isCurrent(request)) applyLineups(result);
  } else if (nextPage === "prediction") {
    const references =
      playersFeature.catalog.references ?? (await api.playerCatalogReferenceData());
    const matchId =
      predictionFeature.context.matchId &&
      references.managed_matches.some((item) => item.id === predictionFeature.context.matchId)
        ? predictionFeature.context.matchId
        : (references.upcoming_matches[0]?.id ??
          references.managed_matches[0]?.id ??
          null);
    if (navigation.isCurrent(request)) {
      playersFeature.catalog.setReferences(references);
      const saved=workspaceState.module("prediction").controls??{};
      const model=saved["prediction-model-family"]?.value;
      const snapshot=saved["prediction-stored-snapshot"]?.value;
      const rule=saved["explicit-rule-package-id"]?.value;
      predictionFeature.initializeContext({match_id:matchId??"",model_family:model&&session.bootstrap?.data.models.some(m=>m.model_id===model)?model as PredictionModelFamily:predictionFeature.context.model,snapshot_type:snapshot&&["T-N","T-24h","T-6h","T-1h"].includes(snapshot)?snapshot:predictionFeature.context.snapshot,explicit_rule_package_id:rule&&session.bootstrap?.data.rule_packages.some(r=>r.id===rule&&r.status==="active")?rule:null});
      await researchFeature.load(matchId);
      if (!navigation.isCurrent(request)) return;
    }
  } else if (nextPage === "runs") {
    await runsFeature.refresh();
  } else if (nextPage === "review") {
    await reviewFeature.load();
  } else if (nextPage === "analytics") {
    await loadAnalysisCenter();
  }
}

async function refreshBootstrapAndCatalog(): Promise<void> {
  await session.refresh();
  await playersFeature.load(true);
  if (teamsFeature.catalog.list !== null || navigation.page === "teams" || navigation.page === "lineup_presets") await teamsFeature.load(true);
}

async function previewPlayerFromTeam(playerId: string): Promise<void> {
  const source = await teamsFeature.readPlayerFromTeam(playerId); if (!source || !source.current()) return;
  playersFeature.catalog.adopt(source.player); workspaceState.patchModule("teams", { inspector_collapsed: false }); render({ preserveForm: true });
}

async function refreshLineupPresetPage(): Promise<void> {
  await runBusy(() => teamsFeature.load(false));
  if (teamsFeature.catalog.detail) workspaceState.patchModule("lineup_presets", { active_tab_id: teamsFeature.catalog.detail.team.id });
  render({ preserveForm: true });
}

async function bulkArchiveEntities(entityType: "team" | "player"): Promise<void> { if(entityType === "team") await teamsFeature.archive(); else await playersFeature.archive(); }

async function openPlayerFromTeam(playerId: string): Promise<void> {
  const source=await teamsFeature.readPlayerFromTeam(playerId); if(!source||!source.current())return;
  const now=new Date().toISOString();playersFeature.enterSource({source:"team_roster",team_id:source.teamId,team_name:source.teamName,player_id:playerId,origin_page:"teams",return_section:null,created_at:now,updated_at:now},source.player);

  workspaceState.patchModule("players", { active_section: "directory", inspector_collapsed: false });await navigateTo("players");
}

async function openPlayerProfileFromTeam(playerId: string): Promise<void> {
  const source=await teamsFeature.readPlayerFromTeam(playerId); if(!source||!source.current())return;
  const now=new Date().toISOString();playersFeature.enterSource({source:"team_roster",team_id:source.teamId,team_name:source.teamName,player_id:playerId,origin_page:"teams",return_section:null,created_at:now,updated_at:now},source.player);
  appendWorkspaceTab("players", {id:playerId,label:source.player.player.canonical_name});
  workspaceState.patchModule("players", { active_section: "profile", inspector_collapsed: true });await navigateTo("players");
}


async function returnToSourceTeamProfile(teamId: string): Promise<void> {
  const sourceTeamId = teamId || playersFeature.catalog.source?.team_id;
  if (!sourceTeamId) { workspaceState.patchModule("players", { active_section: "directory", inspector_collapsed: false }); render(); return; }
  await teamsFeature.open(sourceTeamId, "profile", true);
}

async function openPlayerFromLineup(playerId:string,teamId:string,teamName:string,returnSection:"builder"|"chain"="chain"):Promise<void>{
 if(!playerId||!teamId)throw new Error("阵容球员或球队身份缺失，请刷新后重试");const page=navigation.page;
 if(!await playersFeature.catalog.open(playerId,()=>page===navigation.page)||!playersFeature.catalog.detail)return;
 const now=new Date().toISOString();playersFeature.enterSource({source:"match_lineup",team_id:teamId,team_name:teamName||"来源球队",player_id:playerId,origin_page:"lineups",return_section:returnSection,created_at:now,updated_at:now},playersFeature.catalog.detail);
 workspaceState.patchModule("players",{active_section:"profile",inspector_collapsed:true});await navigateTo("players");
}

async function returnToLineupWorkspace(section: "builder" | "chain"): Promise<void> {
  playersFeature.catalog.setSource(null);
  workspaceState.patchModule("lineups", { active_section: section });
  await navigateTo("lineups");
}

async function createLineupPlayerQuick(): Promise<void> {
  const playerName = value("quick-lineup-player-name").trim();
  const teamId = nullableValue("quick-lineup-player-team");
  const positionCode = nullableValue("quick-lineup-player-position");
  if (!playerName) throw new Error("请输入球员姓名");
  if (!teamId) throw new Error("请选择球员所属球队");
  const today = new Date().toISOString().slice(0, 10);
  await runBusy(async () => {
    const created = await api.createPlayer({
      canonical_name: playerName,
      date_of_birth: null,
      nationality_code: nullableValue("quick-lineup-player-nationality"),
      preferred_foot: "unknown",
      height_cm: null,
      status: "active",
      metadata: { entry_mode: "lineup_quick_create" },
    });
    await api.addPlayerTeamPeriod({
      player_id: created.id,
      team_id: teamId,
      season_id: null,
      squad_number: null,
      valid_from: today,
      valid_to: null,
      registration_status: "registered",
      source_document_id: null,
    });
    if (positionCode) {
      await api.assignPlayerPosition({
        player_id: created.id,
        position_code: positionCode,
        proficiency: 0.8,
        default_role_code: null,
        is_primary: true,
        valid_from: today,
        valid_to: null,
        source_document_id: null,
      });
    }
    await refreshBootstrapAndCatalog();
    lineupsFeature.state.lineupBuilderForm = { ...lineupsFeature.state.lineupBuilderForm, team_id: teamId };
    const list = await api.listPlayers({
      search: null,
      team_id: teamId,
      position_code: null,
      availability_status: null,
      player_status: "active",
      limit: 200,
      cursor_name: null,
      cursor_id: null,
    });
    lineupsFeature.state.lineupPlayerCandidates = list.items;
  });
  render();
  toast("球员已创建并加入球队，可继续编辑阵容", "success");
}

async function calculateStoredMatch():Promise<void>{
  predictionFeature.context.update({match_id:lineupsFeature.actions.selectedExchangeMatchId(),snapshot_type:value("stored-match-snapshot"),explicit_rule_package_id:null});
  const current=predictionFeature.capturePresentation(),scope=captureImportData();
  if(!await predictionFeature.execute("formal"))return;
  const result=predictionFeature.lastResult;
  if(session.bootstrap)await session.refreshRecentRuns();
  if(!current()||!scope()||predictionFeature.lastResult!==result)return;
  render({preserveForm:true});showPredictionDetail("比赛推演结果",result);
}

function selectedPredictionMatchId(): string {
  const matchId = nullableValue("prediction-match-id");
  if (!matchId) throw new Error("请先选择比赛");
  return matchId;
}

function storedMatchRouteCommand(matchId: string): RoutePreviewCommand {
  const match = (
    playersFeature.catalog.references?.managed_matches ?? playersFeature.catalog.references?.upcoming_matches ?? []
  ).find((item) => item.id === matchId);
  if (!match) throw new Error("所选比赛不存在，请刷新比赛列表");
  const stage = match.stage_id
    ? session.bootstrap?.data.stages.find((item) => item.id === match.stage_id)
    : null;
  const competition = match.competition_id
    ? session.bootstrap?.data.competitions.find((item) => item.id === match.competition_id)
    : null;
  return {
    kickoff_time: match.kickoff_time,
    competition_id: match.competition_id,
    season_id: match.season_id,
    stage_id: match.stage_id,
    competition_kind:
      stage?.stage_kind ?? competition?.competition_kind ?? "custom",
    explicit_rule_package_id: nullableValue("explicit-rule-package-id"),
    model_family: predictionFeature.context.model,
  };
}

async function previewStoredRoute():Promise<void>{
  const matchId=selectedPredictionMatchId(),command=storedMatchRouteCommand(matchId);
  const route=await predictionFeature.inspectRoute(()=>api.previewRoute(command));
  if(route)toast("模型判定完成","success");
}

function selectedPredictionSnapshotType(): LineupSnapshotType {
  return predictionFeature.context.snapshot;
}

async function readSelectedPredictionLineupChain():Promise<MatchLineupChain>{return predictionFeature.readChain();}

function missingLineupTeam(chain: MatchLineupChain) {
  const teams = [chain.home, chain.away];
  return (
    teams.find((team) => team.versions.length === 0) ??
    teams.find((team) => !team.selected_lineup_id) ??
    teams.find((team) => team.blocking_issues.length > 0) ??
    chain.home
  );
}

async function openMissingPredictionLineup(
  chain: MatchLineupChain,
): Promise<void> {
  const target = missingLineupTeam(chain);
  lineupsFeature.state.pairedLineupBuilder = {
    match_id: chain.match_record.id,
    lineup_type: "expected",
    snapshot_type: chain.snapshot_type as LineupSnapshotType,
    captured_at: localDateTimeInputValue(chain.data_cutoff_time),
    source_urls: "",
    home: {
      ...emptyPairedLineupSide(),
      team_id: chain.home.team_id,
      team_name: chain.home.team_name,
      coach_id: lineupsFeature.actions.preferredCoachForTeam(chain.home.team_id),
    },
    away: {
      ...emptyPairedLineupSide(),
      team_id: chain.away.team_id,
      team_name: chain.away.team_name,
      coach_id: lineupsFeature.actions.preferredCoachForTeam(chain.away.team_id),
    },
  };
  lineupsFeature.state.selectedManagedMatchId = chain.match_record.id;
  lineupsFeature.state.selectedMatchLineupChain = chain;
  workflowNavigation.set({ page: "prediction", section: workspaceState.module("prediction").active_section ?? "formal" });
  workflowContinuation = {
    reason: `补齐${chain.match_record.home_team_name}与${chain.match_record.away_team_name}双方阵容`,
    matchId: chain.match_record.id,
    snapshotType: chain.snapshot_type as LineupSnapshotType,
  };
  workspaceState.patchModule("lineups", {
    active_section: "builder",
    controls: {},
  });
  await navigateTo("lineups");
  const [homeCount, awayCount] = await runBusy(lineupsFeature.actions.loadBothPairedLineupSides);
  render({ preserveForm: true });
  toast(
    `已进入双方阵容编排：${chain.home.team_name} ${homeCount} 人，${chain.away.team_name} ${awayCount} 人`,
    "success",
  );
  if (target.blocking_issues.length > 0) {
    toast(`${target.team_name}当前阻断：${target.blocking_issues.join("；")}`, "normal");
  }
}

async function checkPredictionLineupChain():Promise<void>{
  if(await predictionFeature.check())toast(predictionFeature.readiness?.can_run_formal?"当前输入允许正式推演":predictionFeature.readiness?.can_run_shadow?"当前输入仅允许影子推演":"当前输入存在阻断","normal");
}

async function preparePredictionLineups(): Promise<void> {
  const chain =
    predictionFeature.chain?.match_record.id === selectedPredictionMatchId() &&
    predictionFeature.chain.snapshot_type === selectedPredictionSnapshotType()
      ? predictionFeature.chain
      : await runBusy(readSelectedPredictionLineupChain);
  if (chain.ready_for_model) {
    render({ preserveForm: true });
    showModal("当前双方阵容",chain);
    return;
  }
  await openMissingPredictionLineup(chain);
}

async function continueLineupPrediction(): Promise<void> {
  const chain = lineupsFeature.state.selectedMatchLineupChain;
  if (!chain?.ready_for_model) throw new Error("双方阵容尚未通过模型门禁");
  workflowContinuation = null;
  workflowNavigation.clear();
  predictionFeature.context.matchId = chain.match_record.id;
  predictionFeature.context.snapshot = chain.snapshot_type as LineupSnapshotType;
  workspaceState.patchModule("prediction", { active_section: "formal" });
  await navigateTo("prediction");
  render({ preserveForm: true });
  toast("双方阵容已就绪，可以开始正式推演", "success");
}

async function runShadowPredictionMatch():Promise<void>{
  closeModal();
  if(await predictionFeature.execute("shadow")){render({preserveForm:true});showPredictionResult(predictionFeature.lastResult as Record<string,unknown>,"shadow");toast("影子推演完成；未写入正式历史","success");}
}

async function calculatePredictionMatch():Promise<void>{
  const current=predictionFeature.capturePresentation(),scope=captureImportData();
  if(!await predictionFeature.execute("formal"))return;
  const result=predictionFeature.lastResult;
  if(session.bootstrap)await session.refreshRecentRuns();
  if(!current()||!scope()||predictionFeature.lastResult!==result)return;
  render({preserveForm:true});showPredictionResult(result as Record<string,unknown>,"formal");toast("正式推演及原运行读回完成","success");
}

function resolveP4Conflict(conflictId:string,acceptUnknown:boolean):void {
  const selection=()=>document.querySelector<HTMLInputElement>(`input[name="p4-conflict-${CSS.escape(conflictId)}"]:checked`)?.value??null;
  const note=()=>nullableValue(`p4-conflict-note-${conflictId}`);
  const intent=researchFeature.prepareDecision(conflictId,acceptUnknown,selection(),note());
  const current=researchFeature.captureDraft();
  showConfirmation("确认冲突处理","确认后重新核验任务版本、证据和截止时间；条件变化时需重新确认。",
    [["比赛",researchFeature.matchWorkspace?.match_key??intent.matchId],["原任务",intent.taskId],["处理",acceptUnknown?"接受当前未知":intent.command.selected_evidence_ids.join("、")],["说明",intent.command.note??"未填写"]],
    "确认追加",async()=>{await researchFeature.decide(intent,current);},{tone:"neutral"});
}

function requestHideRunHistory(runId:string,_runLabel:string):void {
  const intent=runsFeature.prepareHide(runId),current=runsFeature.captureView();
  showConfirmation("从历史列表移除","该操作只隐藏列表中的记录，不删除模型运行、概率矩阵、快照、复盘或收敛血缘。",
    [["推演",intent.label],["运行 ID",intent.runId],["底层数据","完整保留，可继续用于审计、复盘与收敛"]],"确认移除",async()=>{await runsFeature.hide(intent,current);},{tone:"neutral"});
}

async function openHistoryRun(runId:string):Promise<void> {
  if(navigation.page!=="runs"){
    const pending=navigateTo("runs"),current=runsFeature.captureView();await pending;if(!current())return;
  }
  if(await runsFeature.open(runId))document.querySelector<HTMLElement>("#runs-title")?.focus({preventScroll:true});
}

async function loadSelectedReviewMatch():Promise<void> {
 const id=reviewFeature.state.choiceId;if(!id)throw Error("请先选择需要复盘的比赛");
 reviewFacts.capture(app,reviewFeature.state.selectedMatchId);
 lineupsFeature.state.lineupBuilderForm={...lineupsFeature.state.lineupBuilderForm,match_id:id,team_id:"",lineup_type:"actual",formation_id:"",formation:"",coach_id:"",source_urls:"",captured_at:""};
 lineupsFeature.state.lineupBuilderPlayers=[];lineupsFeature.state.lineupPlayerCandidates=[];
 reviewFeature.viewChanged();workspaceState.patchModule("review",{active_section:null});
 await reviewFeature.select(id);
}
function exportMatchReviewPackage():Promise<boolean>|void {
 if(!reviewFeature.state.workflow)return reviewFeature.file("export");
 const current=reviewFeature.captureConfirmation(),version=reviewFeature.version,workflow=reviewFeature.state.workflow;
 showConfirmation("重新导出赛后复盘资料包","新一轮导出会替代当前包工作流。此前预检和人工确认不能用于新文件；已写入事实不会由此撤回。",[["比赛",workflow.match_key],["当前资料包",workflow.package_id],["原文件 SHA256",workflow.import_sha256??workflow.export_sha256??"未记录"]],"重新导出",async()=>{if(!current()||version!==reviewFeature.version)throw Error("原确认已失效，请重新核对当前资料包");await reviewFeature.file("export");},{tone:"neutral"});
}
function requestReviewWrite(kind:ReviewIntent["kind"],candidateId?:string,decision?:"accept"|"reject"):void {
 reviewFacts.capture(app,reviewFeature.state.selectedMatchId);
 const match=reviewFeature.selected;if(!match)throw Error("请先载入当前比赛");
 const fields=reviewFacts.values(match,reviewFeature.state.lineups,reviewFeature.state.detail).fields;
 const intent=reviewFeature.prepare(kind,{confirmedBy:fields.confirmedBy.trim()||null,note:kind==="candidate"?(decision==="accept"?"桌面端人工接受":"桌面端人工拒绝"):fields.confirmationNote.trim()||null,draft:kind==="manual"?reviewFacts.parse(match,reviewFeature.state.lineups,reviewFeature.state.detail):null,candidateId:candidateId??null,decision:decision??null});
 const capture=reviewFeature.captureConfirmation(),draftVersion=reviewFacts.version,current=()=>capture()&&draftVersion===reviewFacts.version;
 const labels={confirm:["人工确认本轮资料包","确认仅冻结本轮导入，后端会复查原文件 SHA256；不会立即写入事实。","确认本轮资料包"],facts:["写入真实赛后事实","写入实际阵容、正式赛果、结构化事件及球员观察；生成复盘需单独执行。","写入真实赛后事实"],generate:["生成正式复盘","只使用已写入事实生成本轮复盘；不会自动结算或应用能力候选。","生成正式复盘"],manual:["生成手工复盘","按本场实际阵容和已录入事实生成复盘；空白不补0，已有结构化事件与换人保留。","生成复盘"],candidate:[decision==="accept"?"接受能力更新候选":"拒绝能力更新候选",decision==="accept"?"此操作将应用当前候选，写入正式能力历史。":"只拒绝当前候选，不修改其他待审核项。",decision==="accept"?"确认接受":"确认拒绝"]};
 const [title,description,label]=labels[kind],workflow=reviewFeature.state.workflow;
 const facts:[string,string][]=[["比赛",match.match_record.home_team_name+" vs "+match.match_record.away_team_name],["比赛 ID",intent.matchId]];
 if(kind==="manual")facts.push(["复盘版本",intent.draft!.review_version!],["90分钟比分",intent.draft!.result.home_goals_90+"-"+intent.draft!.result.away_goals_90],["球员观察",String(intent.draft!.player_observations.length)]);
 else if(kind==="candidate")facts.push(["复盘",reviewFeature.state.detail!.summary.id],["候选",candidateId!]);
 else facts.push(["资料包",workflow?.package_id??"未记录"],["原文件 SHA256",workflow?.import_sha256??workflow?.export_sha256??"未记录"]);
 showConfirmation(title,description,facts,label,async()=>{await reviewFeature.submit(intent,current);},{tone:"neutral"});
}
function syncReviewManualValidation():void {
 reviewFacts.capture(app,reviewFeature.state.selectedMatchId);
 const result=reviewFacts.validate(reviewFeature.selected,reviewFeature.state.lineups,reviewFeature.state.detail),allowed=reviewFeature.allows("manual");
 const message=app.querySelector<HTMLElement>("#review-manual-validation"),submit=app.querySelector<HTMLButtonElement>('[data-action="generate-match-review"]');
 if(message)message.textContent=result.error??(!allowed?"当前请求或资料包状态尚不允许人工生成。":"所需事实已录入；确认后生成本次人工复盘。");
 if(submit)submit.disabled=!!result.error||!allowed;
}

async function handleAction(
  action: string,
  button: HTMLElement,
): Promise<void> {
  const shellAction = shellController?.handleAction(action);
  if (shellAction === true || (shellAction && await shellAction)) return;
  if (action.includes("api-workspace") && await aiWorkspaceActions.handle(button)) return;
  if(action.includes("openai")&&await profileActions.handle(button))return;
  if((action.includes("issue-")||action.startsWith("set-logs")||action==="issue-log-scope")&&await logActions.handle(button))return;
  switch (action) {
    case "toggle-workspace-pane":
      toggleWorkspacePane(button.dataset.pane === "inspector" ? "inspector" : "module-sidebar");
      break;
    case "complete-workflow":
      await startWorkflowCompletion(button);
      break;
    case "workflow-return":
      await returnToWorkflow();
      break;
    case "workflow-cancel":
      workflowContinuation = null;
      workflowNavigation.clear();
      render({ preserveForm: true });
      break;
    case "select-workspace-section": {
      const sectionId = button.dataset.sectionId;
      if (!sectionId) break;
      if (navigation.page === "workbooks") workbooksFeature.changedCategory();
      if (navigation.page === "prediction") predictionFeature.viewChanged();
      if (sectionId === "directory" && navigation.page === "teams") {
        const id = teamsFeature.catalog.detail?.team.id ?? null;
        teamsFeature.directory(); if (id) workspaceState.restoreFocus("teams", app.querySelector(".page-container") ?? app, `team-directory-${id}`); break;
      }
      workspaceState.patchModule(navigation.page, { active_section: sectionId });
      render({ preserveForm: true });
      if (navigation.page === "prediction" && sectionId === "p4") await researchFeature.load(predictionFeature.context.matchId);
      if (sectionId === "directory" && navigation.page === "players" && playersFeature.catalog.detail) workspaceState.restoreFocus("players", app.querySelector(".page-container") ?? app, `player-directory-${playersFeature.catalog.detail.player.id}`);
      break;
    }
    case "refresh-service-state":
      await runBusy(() => session.refresh());
      render({ preserveForm: true });
      break;
    case "show-database-technical":
      if (session.bootstrap) showHtmlModal("技术连接信息", "凭据已脱敏，不显示密码。", databaseTechnical(session.bootstrap));
      break;
    case "show-database-scope":
      showModal("数据服务使用边界", "已连接不等于所有业务已就绪。连接与保存可能执行迁移与内置资料注册；清除连接不删除业务数据。危险重置须核对当前数据库身份并输入名称确认，无法撤销。统计、模型提供器与业务许可分别校验。");
      break;
    case "show-architecture-scope":
      showModal("架构说明范围", "本页说明职责、历史和路由原则，不代表实时健康、模型可执行性或发布验收结果。连接状态、输入条件、提供器与许可由对应业务流程校验。");
      break;
    case "show-model-runtime-boundary":
      showModal("模型接口与运行能力", "统一接口与规则登记不等于已安装模型运行时。当前公共仓库不捆绑外部模型运行时，实际执行前仍须验证提供器、模型许可和本次输入条件。");
      break;
    case "jump-workspace-anchor": {
      const anchorId = button.dataset.anchorId;
      if (!anchorId) break;
      const target = document.getElementById(anchorId);
      if (!target) break;
      if (activateSectionPanel(app, anchorId)) {
        workspaceState.patchModule(navigation.page, { active_section: anchorId });
        break;
      }
      if (target instanceof HTMLDetailsElement) target.open = true;
      const nav = button.closest<HTMLElement>(".workspace-anchor-nav, .workspace-task-anchor-nav");
      nav?.querySelectorAll<HTMLElement>("button[data-anchor-id]").forEach((item) => item.classList.toggle("active", item === button));
      target.scrollIntoView({ behavior: motionScrollBehavior("smooth"), block: "start" });
      const anchorMenu = button.closest<HTMLDetailsElement>("details[data-workspace-anchor-menu]");
      if (anchorMenu) {
        anchorMenu.open = false;
        target.tabIndex = -1;
        target.focus({ preventScroll: true });
      }
      break;
    }
    case "review-adopt-file": {
      const current=reviewFeature.captureConfirmation(),version=reviewFeature.version,w=reviewFeature.state.workflow;
      if(!w)throw Error("请先刷新当前登记文件");
      showConfirmation("核对当前登记文件","只采用当前登记状态；原请求仍不能据此认定为成功，不会重发或写入事实。",[["比赛",w.match_key],["资料包",w.package_id],["文件",w.import_path??w.export_path],["SHA256",w.import_sha256??w.export_sha256??"未记录"]],"采用当前登记状态",async()=>{await reviewFeature.adoptCurrentFile(version,current);},{tone:"neutral"});break;
    }
    case "review-open-analytics":
      if(!matchReviewWorkflowAllows(reviewFeature.state.workflow,"open_analytics"))throw Error("后端尚未允许进入分析");
      await navigateTo("analytics");break;
    case "review-view": {
      const view=button.dataset.reviewView;if(!view||!["workflow","manual","results","history"].includes(view))break;
      reviewFacts.capture(app,reviewFeature.state.selectedMatchId);reviewFeature.viewChanged();
      workspaceState.patchModule("review",{active_section:view});render({preserveForm:true});break;
    }
    case "toggle-review-metrics": {
      const row=button.closest<HTMLElement>("[data-review-player]"),details=row?.querySelector("details");if(!details)break;
      details.open=!details.open;button.setAttribute("aria-expanded",String(details.open));reviewFacts.capture(app,reviewFeature.state.selectedMatchId);break;
    }
    case "select-review-workflow-step": {
      const step = Number(button.dataset.reviewStep);
      if (!Number.isInteger(step) || step < 1 || step > 9) break;
      reviewFacts.capture(app,reviewFeature.state.selectedMatchId);
      reviewFeature.viewChanged();
      workspaceState.patchModule("review", { active_section: `step-${step}` });
      render({ preserveForm: true });
      break;
    }
    case "run-release-acceptance":releaseActions.run();break;
    case "refresh-release-acceptance":await releaseFeature.refresh();break;
    case "open-release-acceptance-run":workspaceState.patchModule("release",{active_section:"overview"});await releaseFeature.select(button.dataset.runId??"");break;
    case "recover-release-acceptance":await releaseFeature.recover();break;
    case "show-release-check-evidence":releaseActions.evidence(button.dataset.checkId??"",button.dataset.runId);break;
    case "show-release-acceptance-json":releaseActions.report(button.dataset.runId);break;
    case "release-scope":releaseActions.scope();break;
    case "open-selected-teams":
      await openSelectedWorkspaceObjects("teams");
      break;
    case "open-selected-players":
      await openSelectedWorkspaceObjects("players");
      break;
    case "activate-teams-tab":
      await activateWorkspaceTab("teams", button.dataset.objectId ?? "");
      break;
    case "activate-players-tab":
      await activateWorkspaceTab("players", button.dataset.objectId ?? "");
      break;
    case "close-teams-tab":
      await closeWorkspaceTab("teams", button.dataset.objectId ?? "");
      break;
    case "close-players-tab":
      await closeWorkspaceTab("players", button.dataset.objectId ?? "");
      break;
    case "set-teams-workspace-mode":
      setWorkspaceMode("teams", (button.dataset.mode ?? "detail") as WorkspaceLayoutMode);
      break;
    case "set-players-workspace-mode":
      setWorkspaceMode("players", (button.dataset.mode ?? "detail") as WorkspaceLayoutMode);
      break;
    case "connect-database":
      await databaseFeature.connect(app);
      break;
    case "disconnect-database":
      await databaseFeature.disconnect();
      break;
    case "open-command-menu":
      openCommandMenu();
      break;
    case "request-reset-database":
      databaseFeature.requestReset(document);
      break;
    case "execute-database-reset":
      await modal.runPendingAction();
      break;
    case "dry-run":
      await dryRun();
      break;
    case "preview-route":
      await previewRoute();
      break;
    case "execute-prediction":
      await executePrediction();
      break;
    case "preview-stored-route":
      await previewStoredRoute();
      break;
    case "show-prediction-readiness":
      if(predictionFeature.readiness)showModal("当前输入完整度检查",predictionFeature.readiness);
      break;
    case "prediction-readback":
      await predictionFeature.readback();break;
    case "prediction-open-candidate": {
      const record=predictionFeature.candidates.find(r=>r.id===button.dataset.runId);
      if(record)showPredictionDetail("相关运行记录（尚未认定原请求成功）",{...record,execution_mode:"formal"});
      break;
    }
    case "prediction-adopt-candidate": {
      const id=button.dataset.runId??"",receipt=predictionFeature.receipt,record=predictionFeature.candidates.find(r=>r.id===id);
      if(!receipt||!record)throw Error("原请求或候选运行已失效");
      const identity=JSON.stringify(receipt),audit=objectValue(record.input_audit);
      showConfirmation("采用已核对的运行","只采用真实已有结果，不发送运行请求；请核对原请求与实际运行的身份。",[["原比赛",receipt.command.match_id],["窗口",receipt.command.snapshot_type],["原指纹",receipt.manifest],["运行 ID",id],["实际指纹",String(audit.manifest_sha256??"未记录")]],"采用已有结果",async()=>{
        if(JSON.stringify(predictionFeature.receipt)!==identity)throw Error("原请求已变化，请重新核查");
        await predictionFeature.adoptExisting(id);
      });break;
    }
    case "prediction-retry-shadow": {
      const identity=JSON.stringify(predictionFeature.receipt);
      showConfirmation("重新检查影子请求","影子请求不写入正式历史。此动作只解除原只读请求的未知状态，随后必须重新检查当前影子准入。",[],"重新检查",async()=>{
        if(JSON.stringify(predictionFeature.receipt)!==identity)throw Error("原请求已变化");
        predictionFeature.retryReadonlyShadow();
      });break;
    }
    case "calculate-prediction-match":
      await calculatePredictionMatch();
      break;
    case "run-shadow-prediction-match":
      await runShadowPredictionMatch();
      break;
    case "check-prediction-lineup-chain":
      await checkPredictionLineupChain();
      break;
    case "prepare-prediction-lineups":
      await preparePredictionLineups();
      break;
    case "continue-lineup-prediction":
      await continueLineupPrediction();
      break;
    case "plan-p4-horizons":
      await researchFeature.plan(nullableValue("p4-plan-rule-package")??"");
      break;
    case "refresh-p4-workbench":
      await researchFeature.load(predictionFeature.context.matchId);
      break;
    case "open-p4-task":
      await researchFeature.openTask(button.dataset.taskId ?? "");
      break;
    case "resolve-p4-conflict-select":
      await resolveP4Conflict(button.dataset.conflictId ?? "", false);
      break;
    case "resolve-p4-conflict-unknown":
      await resolveP4Conflict(button.dataset.conflictId ?? "", true);
      break;
    case "recover-p4-research":
      await researchFeature.recover();
      break;
    case "show-p4-snapshot":
      showHtmlModal("P4 冻结快照","只读 · 实际冻结结果",researchSnapshotView(researchFeature.snapshot()));
      break;
    case "create-competition":
      await competitionFeature.createCompetition(app);
      break;
    case "clear-rules-catalogue-filters":
      clearRulesCatalogueFilters(filterRulesCompetitionCatalogue);
      break;
    case "show-competition-path":
      competitionFeature.showPath(button.dataset.competitionId ?? "");
      break;
    case "request-delete-competition":
      competitionFeature.requestDelete(button.dataset.competitionId ?? "");
      break;
    case "create-season":
      await competitionFeature.createSeason(app);
      break;
    case "create-stage":
      await competitionFeature.createStage(app);
      break;
    case "create-round":
      await competitionFeature.createRound(app);
      break;
    case "register-rule-package":
      await rulesFeature.registerPackage();
      break;
    case "clear-rule-package":
      rulesFeature.clearPackage();
      break;
    case "show-pending-rule-package":
      if (rulesFeature.packages.pending) showModal("规则包只读详情", rulesFeature.packages.pending);
      break;
    case "create-binding":
      await rulesFeature.createBinding(app);
      break;
    case "refresh-player-catalog":
      await runBusy(() => playersFeature.load(true));
      render();
      break;
    case "refresh-team-catalog":
      await runBusy(() => teamsFeature.load(true));
      render();
      break;
    case "show-lineup-preset-rules":
      showHtmlModal("预设套用规则", "本场副本与球队预设独立", "<p>先选择比赛与球队，预览最新名单、阵型与角色。确认仅替换目标一侧草稿；比赛或草稿变化后需重新预览。双方阵容仍需各11名首发并通过独立保存与模型准入检查。</p>");
      break;
    case "refresh-lineup-preset-page":
      await refreshLineupPresetPage();
      break;
    case "select-lineup-preset-team":
      await teamsFeature.selectPresetTeam(button.dataset.teamId ?? "");
      break;
    case "search-teams":
      await teamsFeature.search();
      break;
    case "search-teams-from-detail":
      await teamsFeature.search(true);
      break;
    case "clear-team-filters":
      await teamsFeature.clearFilters();
      break;
    case "clear-team-filters-from-detail":
      await teamsFeature.clearFilters(true);
      break;
    case "next-team-page":
      await teamsFeature.next();
      break;
    case "previous-team-page":
      await teamsFeature.previous();
      break;
    case "open-team":
      await teamsFeature.open(button.dataset.teamId ?? "");
      break;
    case "return-team-directory": {
      const id = teamsFeature.catalog.detail?.team.id ?? null;
      teamsFeature.directory(); if (id) workspaceState.restoreFocus("teams", app.querySelector(".page-container") ?? app, `team-directory-${id}`);
      break;
    }
    case "update-team":
      await teamsFeature.editor.update(button.dataset.teamId ?? "");
      break;
    case "add-team-name":
      await teamsFeature.editor.addName(button.dataset.teamId ?? "");
      break;
    case "save-team-profile":
      await teamsFeature.editor.profile(button.dataset.teamId ?? "");
      break;
    case "save-formation-usage":
      await teamsFeature.editor.formation(button.dataset.teamId ?? "");
      break;
    case "export-team-package-template":
      await teamsFeature.exportFile("package-template");
      break;
    case "preview-team-package-import":
      await teamsFeature.previewPackage();
      break;
    case "resolve-team-package-conflict":
      await teamsFeature.resolvePackage(
        button.dataset.packageScope === "player" ? "player" : "team",
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-team-package-conflict":
      await teamsFeature.resolvePackage(
        button.dataset.packageScope === "player" ? "player" : "team",
        button.dataset.rowId ?? "",
        null,
        true,
      );
      break;
    case "commit-team-package-import":
      await teamsFeature.commitPackage();
      break;
    case "show-team-package-preview-json":
      if (teamsFeature.imports.package)
        showModal(
          "球队完整资料包预检结果",
          teamsFeature.imports.package,
          `<button type="button" class="secondary" data-action="close-workspace-detail">关闭</button><button type="button" class="primary" data-action="export-team-package-preview-json">导出完整预检 JSON</button>`,
        );
      break;
    case "export-team-package-preview-json":
      await teamsFeature.exportPackagePreview();
      break;
    case "export-team-template":
      await teamsFeature.exportFile("monthly-template");
      break;
    case "export-team-data":
      await teamsFeature.exportFile("monthly-data");
      break;
    case "workbook-commit":
      workbooksFeature.requestCommit();
      break;
    case "workbook-readback":
      await workbooksFeature.readback();
      break;
    case "workbook-export-template":
      workbooksFeature.requestExport(true);
      break;
    case "workbook-export-data":
      workbooksFeature.requestExport(false);
      break;
    case "preview-team-import":
      await teamsFeature.previewMonthly();
      break;
    case "resolve-team-import-conflict":
      await teamsFeature.resolveMonthly(
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-team-import-conflict":
      await teamsFeature.resolveMonthly(button.dataset.rowId ?? "", null, true);
      break;
    case "commit-team-import":
      await teamsFeature.commitMonthly();
      break;
    case "show-team-import-preview-json":
      if (teamsFeature.imports.monthly)
        showModal("球队月度工作簿预检结果", teamsFeature.imports.monthly);
      break;
    case "create-coach":
      await teamsFeature.editor.createCoach();
      break;
    case "add-team-coach-period":
      await teamsFeature.editor.coachPeriod(button.dataset.teamId ?? "");
      break;
    case "bulk-archive-teams":
      await bulkArchiveEntities("team");
      break;
    case "bulk-archive-players":
      await bulkArchiveEntities("player");
      break;
    case "bulk-delete-teams":
      await teamsFeature.requestDelete();
      break;
    case "request-force-delete-team":
      await teamsFeature.requestForceDeleteTeam(button.dataset.teamId ?? "");
      break;
    case "confirm-force-delete-team":
      await teamsFeature.confirmForceDeleteTeam();
      break;
    case "bulk-delete-players":
      playersFeature.requestBulkDelete();
      break;
    case "preview-player-from-team":
      await previewPlayerFromTeam(button.dataset.playerId ?? "");
      break;
    case "open-player-from-team":
      await openPlayerFromTeam(button.dataset.playerId ?? "");
      break;
    case "open-player-profile-from-team":
      await openPlayerProfileFromTeam(button.dataset.playerId ?? "");
      break;
    case "return-to-source-team-profile":
      await returnToSourceTeamProfile(button.dataset.teamId ?? "");
      break;
    case "open-player-from-lineup":
      await openPlayerFromLineup(
        button.dataset.playerId ?? "",
        button.dataset.teamId ?? "",
        button.dataset.teamName ?? "",
        button.dataset.returnSection === "builder" ? "builder" : "chain",
      );
      break;
    case "return-to-lineup-workspace":
      await returnToLineupWorkspace(button.dataset.returnSection === "builder" ? "builder" : "chain");
      break;
    case "open-team-api-workspace":
      await aiWorkspaceActions.openTeam(button.dataset.teamId ?? null);
      break;
    case "open-player-api-workspace":
      await aiWorkspaceActions.openPlayer(button.dataset.playerId ?? "");
      break;
    case "search-team-options":
      await playersFeature.searchTeamOptions();
      break;
    case "search-players":
      await playersFeature.search();
      break;
    case "clear-player-filters":
      await playersFeature.clearFilters();
      break;
    case "next-player-page":
      await playersFeature.next();
      break;
    case "previous-player-page":
      await playersFeature.previous();
      break;
    case "open-player":
      await playersFeature.openDirectory(button.dataset.playerId ?? "");
      break;
    case "open-player-profile":
      await playersFeature.openProfile(button.dataset.playerId ?? "");
      break;
    case "open-team-profile":
      await teamsFeature.openProfile(button.dataset.teamId ?? "");
      break;
    case "show-player-json":
      if (playersFeature.catalog.detail) showModal("球员完整档案", playersFeature.catalog.detail);
      break;
    case "create-team":
      await teamsFeature.create();
      break;
    case "create-provider":
      await playersFeature.editor.createProvider();
      break;
    case "create-player":
      await playersFeature.editor.createPlayer();
      break;
    case "create-lineup-player":
      await createLineupPlayerQuick();
      break;
    case "update-player":
      await playersFeature.editor.updatePlayer(button.dataset.playerId ?? "");
      break;
    case "request-delete-player":
      await playersFeature.requestDelete(
        button.dataset.playerId ?? "",
        button.dataset.playerName ?? "未命名球员",
      );
      break;
    case "export-player-template":
      await playersFeature.exportTemplate();
      break;
    case "export-player-data":
      await playersFeature.exportData();
      break;
    case "preview-player-import":
      await playersFeature.previewImport();
      break;
    case "resolve-import-conflict":
      await playersFeature.resolveImport(
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-import-conflict":
      await playersFeature.resolveImport(button.dataset.rowId ?? "", null, true);
      break;
    case "commit-player-import":
      await playersFeature.commitImport();
      break;
    case "show-import-preview-json":
      if (playersFeature.imports.preview) showModal("表格导入检查结果", playersFeature.imports.preview);
      break;
    case "add-player-name":
      await playersFeature.editor.addPlayerName(button.dataset.playerId ?? "");
      break;
    case "assign-player-position":
      await playersFeature.editor.assignPlayerPosition(button.dataset.playerId ?? "");
      break;
    case "add-player-team-period":
      await playersFeature.editor.addPlayerTeamPeriod(button.dataset.playerId ?? "");
      break;
    case "add-player-availability":
      await playersFeature.editor.addPlayerAvailability(button.dataset.playerId ?? "");
      break;
    case "add-player-ability":
      await playersFeature.editor.addPlayerAbility(button.dataset.playerId ?? "");
      break;
    case "add-player-external-id":
      await playersFeature.editor.addPlayerExternalId(button.dataset.playerId ?? "");
      break;
    case "add-player-dynamic-tag":
      await playersFeature.editor.addPlayerDynamicTag(button.dataset.playerId ?? "");
      break;
    case "calculate-player-contribution":
      await playersFeature.editor.calculatePlayerContribution(button.dataset.playerId ?? "");
      break;
    case "export-match-template":
      await lineupsFeature.actions.exportMatchTemplate();
      break;
    case "export-match-data":
      await lineupsFeature.actions.exportMatchData();
      break;
    case "export-ai-match-package":
      await lineupsFeature.actions.exportAiMatchPackage();
      break;
    case "calculate-stored-match":
      await calculateStoredMatch();
      break;
    case "preview-match-import":
      await lineupsFeature.actions.previewMatchImport(false);
      break;
    case "preview-ai-match-import":
      await lineupsFeature.actions.previewMatchImport(true);
      break;
    case "resolve-match-import-conflict":
      await lineupsFeature.actions.resolveMatchImportConflict(
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-match-import-conflict":
      await lineupsFeature.actions.resolveMatchImportConflict(button.dataset.rowId ?? "", null, true);
      break;
    case "commit-match-import":
      await lineupsFeature.actions.commitMatchImport();
      break;
    case "show-match-import-json":
      if (lineupsFeature.imports.preview) {
        await lineupsFeature.imports.readback();
        if(lineupsFeature.imports.preview)showModal("比赛与阵容导入预检", lineupsFeature.imports.preview);
      }
      break;
    case "select-managed-match": {
      const matchId = button.dataset.matchId ?? "";
      lineupsFeature.state.selectedManagedMatchId = matchId || null;
      render({ preserveForm: true });
      queueMicrotask(() => {
        lineupsFeature.actions.autoSelectMatchSeason();
        lineupsFeature.actions.filterMatchTeamOptions();
      });
      break;
    }
    case "new-managed-match":
      lineupsFeature.state.selectedManagedMatchId = null;
      render({ preserveForm: true });
      break;
    case "open-match-lineups": {
      const matchId = button.dataset.matchId ?? "";
      lineupsFeature.actions.resetPairedBuilderForMatch(matchId, false);
      workspaceState.patchModule("lineups", { active_section: "builder" });
      await runBusy(lineupsFeature.actions.loadBothPairedLineupSides);
      render({ preserveForm: true });
      break;
    }
    case "open-match-prediction": {
      const matchId = button.dataset.matchId ?? "";
      predictionFeature.context.matchId = matchId;
      predictionFeature.context.snapshot = lineupsFeature.state.pairedLineupBuilder.match_id === matchId
        ? lineupsFeature.state.pairedLineupBuilder.snapshot_type
        : "T-N";
      workspaceState.patchModule("prediction", { active_section: "formal" });
      await navigateTo("prediction");
      break;
    }
    case "create-match":
      await lineupsFeature.actions.createMatch();
      break;
    case "request-delete-match":
      lineupsFeature.actions.requestDeleteMatch(
        button.dataset.matchId ?? "",
        button.dataset.matchLabel ?? "未命名比赛",
      );
      break;
    case "reload-paired-lineup-side": {
      const side = button.dataset.lineupSide as LineupSide;
      const count = await runBusy(() => lineupsFeature.actions.loadPairedLineupSide(side));
      render();
      toast(`${lineupsFeature.actions.pairedSide(side).team_name}已加载 ${count} 名球员`, count > 0 ? "success" : "normal");
      break;
    }
    case "clear-paired-lineup-side":
      lineupsFeature.actions.clearPairedLineupSide(button.dataset.lineupSide as LineupSide);
      break;
    case "open-team-lineup-preset-manager":
      await lineupsFeature.actions.openTeamLineupPresetManager(
        button.dataset.teamId ?? teamsFeature.catalog.detail?.team.id ?? "",
        button.dataset.teamName ?? teamsFeature.catalog.detail?.team.canonical_name ?? "球队",
      );
      break;
    case "open-lineup-preset-manager":
      await lineupsFeature.actions.openLineupPresetManagerForSide(button.dataset.lineupSide as LineupSide);
      break;
    case "request-delete-team-lineup-preset":
      lineupsFeature.actions.requestDeleteTeamLineupPreset(
        button.dataset.presetId ?? "",
        button.dataset.presetName ?? "阵容预设",
        button.dataset.teamId ?? "",
        button.dataset.teamName ?? "球队",
        button.dataset.presetStatus ?? "active",
        Number(button.dataset.memberCount ?? 0),
      );
      break;
    case "open-team-lineup-preset-editor":
      lineupsFeature.actions.openTeamLineupPresetEditor(button.dataset.presetId || null);
      break;
    case "auto-assign-preset-formation":
      lineupsFeature.actions.assignPresetFormationSlots();
      break;
    case "save-team-lineup-preset":
      await lineupsFeature.actions.saveTeamLineupPreset(button.dataset.presetId || null);
      break;
    case "duplicate-team-lineup-preset":
      lineupsFeature.actions.openDuplicateLineupPreset(
        button.dataset.presetId ?? "",
        button.dataset.presetName ?? "阵容预设",
      );
      break;
    case "confirm-duplicate-lineup-preset":
      await lineupsFeature.actions.duplicateLineupPreset(button.dataset.presetId ?? "");
      break;
    case "archive-team-lineup-preset":
      lineupsFeature.actions.requestArchiveLineupPreset(
        button.dataset.presetId ?? "",
        button.dataset.presetName ?? "阵容预设",
      );
      break;
    case "save-current-lineup-as-preset":
      lineupsFeature.actions.openSaveCurrentLineupPreset(button.dataset.lineupSide as LineupSide);
      break;
    case "confirm-save-current-lineup-preset":
      await lineupsFeature.actions.saveCurrentLineupAsPreset(button.dataset.lineupSide as LineupSide);
      break;
    case "preview-apply-lineup-preset":
      await lineupsFeature.actions.previewApplyLineupPreset(button.dataset.lineupSide as LineupSide);
      break;
    case "confirm-apply-lineup-preset":
      await lineupsFeature.actions.applyLineupPreset(
        button.dataset.lineupSide as LineupSide,
        button.dataset.presetId ?? "",
      );
      break;
    case "create-lineup-pair":
      await lineupsFeature.actions.createPairedLineups();
      break;
    case "inspect-paired-lineup-chain":
      await lineupsFeature.actions.inspectPairedLineupChain();
      break;
    case "load-lineup-players":
      await lineupsFeature.actions.loadLineupPlayers();
      break;
    case "add-selected-lineup-player":
      lineupsFeature.actions.addSelectedPairedLineupPlayer(button.dataset.lineupSide as LineupSide);
      break;
    case "open-lineup-player-settings":
      lineupsFeature.actions.openPairedLineupPlayerSettings(
        button.dataset.lineupSide as LineupSide,
        button.dataset.playerId ?? "",
      );
      break;
    case "save-lineup-player-settings":
      lineupsFeature.actions.savePairedLineupPlayerSettings(
        button.dataset.lineupSide as LineupSide,
        button.dataset.playerId ?? "",
      );
      break;
    case "add-lineup-player": {
      const side = button.dataset.lineupSide as LineupSide | undefined;
      if (side) {
        lineupsFeature.actions.addPairedLineupPlayer(
          side,
          button.dataset.playerId ?? "",
          button.dataset.role === "starter",
        );
      } else {
        lineupsFeature.actions.addLineupPlayer(
          button.dataset.playerId ?? "",
          button.dataset.role === "starter",
        );
      }
      break;
    }
    case "remove-lineup-player": {
      const side = button.dataset.lineupSide as LineupSide | undefined;
      if (side) lineupsFeature.actions.removePairedLineupPlayer(side, button.dataset.playerId ?? "");
      else lineupsFeature.actions.removeLineupPlayer(button.dataset.playerId ?? "");
      break;
    }
    case "clear-lineup-builder":
      lineupsFeature.actions.captureLineupFormFromDom();
      lineupsFeature.state.lineupBuilderPlayers = [];
      render();
      break;
    case "inspect-lineup-chain":
      await lineupsFeature.actions.inspectLineupChain();
      break;
    case "create-lineup":
      await lineupsFeature.actions.createLineup();
      break;
    case "refresh-lineups":
      await runBusy(loadLineups);
      render();
      break;
    case "refresh-review":
      await reviewFeature.load();
      break;
    case "load-review-match":
      await loadSelectedReviewMatch();
      break;
    case "export-match-review-package":
      await exportMatchReviewPackage();
      break;
    case "preview-match-review-package":
      await reviewFeature.file("preview");
      break;
    case "confirm-match-review-package":
      requestReviewWrite("confirm");
      break;
    case "commit-match-review-package-facts":
      requestReviewWrite("facts");
      break;
    case "generate-match-review-from-package":
      requestReviewWrite("generate");
      break;
    case "show-match-review-package-json":
      if (reviewFeature.preview) showModal("赛后复盘资料包预检", reviewFeature.preview);
      break;
    case "generate-match-review":
      requestReviewWrite("manual");
      break;
    case "open-match-review":
      reviewFacts.capture(app,reviewFeature.state.selectedMatchId);
      reviewFeature.viewChanged();
      workspaceState.patchModule("review",{active_section:"results"});
      await reviewFeature.openReview(button.dataset.reviewId ?? "");
      break;
    case "decide-ability-candidate":if(navigation.page==="review")requestReviewWrite("candidate",button.dataset.candidateId??"",button.dataset.decision==="accept"?"accept":"reject");else candidateActions.ability(button.dataset.candidateId??"",button.dataset.decision==="accept"?"accept":"reject");break;
    case "show-review-json":
      if (reviewFeature.state.detail) showModal("复盘完整详情", reviewFeature.state.detail);
      break;
    case "inspect-postmatch-readiness":
      await postmatchActions.inspect(button.dataset.reviewId ?? "");
      break;
    case "settle-postmatch-review":
      postmatchActions.settle(button.dataset.reviewId ?? "");
      break;
    case "show-candidate-json": {
      if(navigation.page==="analytics"){candidateActions.abilityDiagnostic(button.dataset.candidateId??"");break;}
      const candidate =
        reviewFeature.state.detail?.ability_candidates.find(
          (item) => item.id === button.dataset.candidateId,
        ) ??
        analyticsFeature.state.abilityCandidates.find(
          (item) => item.id === button.dataset.candidateId,
        );
      if (candidate) showModal("能力更新证据", candidate);
      break;
    }
    case "resume-postmatch-settlement":postmatchActions.settle(button.dataset.reviewId??"",true);break;
    case "recover-postmatch-settlement":await postmatchActions.recoverSettlement();break;
    case "recover-postmatch-monitoring":await postmatchFeature.monitoring.recover();break;
    case "refresh-postmatch-overview":await postmatchFeature.refresh();break;
    case "postmatch-open-review":await navigateTo("review");break;
    case "postmatch-evidence-verdict":postmatchActions.evidenceVerdict(button.dataset.evidenceItemId??"",button.dataset.verdict as import("./types").EvidenceVerdict);break;
    case "recover-evidence-decision":await postmatchActions.recoverEvidence(button.dataset.evidenceItemId??"");break;
    case "show-postmatch-evidence":postmatchActions.showEvidence(button.dataset.evidenceItemId??"");break;
    case "refresh-analysis-page":
      await loadAnalysisCenter();
      break;
    case "refresh-postmatch-monitoring":
      postmatchActions.monitor();
      break;
    case "prepare-evidence-decision":
      postmatchActions.evidenceChoice(button.dataset.evidenceItemId ?? "");
      break;
    case "submit-evidence-decision":
      await postmatchActions.submitEvidence(button.dataset.evidenceItemId ?? "",button.dataset.postmatchConfirm ?? "");
      break;
    case "generate-parameter-tuning":candidateActions.provider();break;
    case "candidate-provider-boundary":candidateActions.provider();break;
    case "check-parameter-lifecycle-readiness":await analyticsFeature.candidates.context.inspect();break;
    case "run-parameter-shadow-validation":candidateActions.parameter(button.dataset.candidateId??"","shadow");break;
    case "promote-parameter-candidate":candidateActions.parameter(button.dataset.candidateId??"","promote");break;
    case "rollback-parameter-candidate":candidateActions.parameter(button.dataset.candidateId??"","rollback");break;
    case "show-parameter-lifecycle-history":await candidateActions.history(button.dataset.candidateId??"");break;
    case "decide-parameter-tuning":candidateActions.parameter(button.dataset.candidateId??"",button.dataset.decision==="reject"?"reject":"accept_for_backtest");break;
    case "reject-parameter-candidate":candidateActions.parameter(button.dataset.candidateId??"","reject");break;
    case "show-parameter-tuning":candidateActions.diagnostic(button.dataset.candidateId??"");break;
    case "recover-candidate-operation":await analyticsFeature.candidates.recover(button.dataset.candidateId??"");break;
    case "analysis-step":analyticsFeature.setStep(button.dataset.analyticsStep as import("./features/analytics").AnalysisStep);break;
    case "run-full-analysis":analyticsActions.enqueue("full_analysis_refresh");break;
    case "run-model-analysis":analyticsActions.enqueue("refresh_analytics");break;
    case "run-quality-scan":analyticsActions.enqueue("data_quality_scan");break;
    case "run-query-performance-scan":analyticsActions.enqueue("query_performance_scan");break;
    case "refresh-analysis-jobs":await analyticsFeature.refresh();break;
    case "cancel-analysis-job":analyticsActions.job(button.dataset.jobId??"","cancel");break;
    case "retry-analysis-job":analyticsActions.job(button.dataset.jobId??"","retry");break;
    case "recover-analysis-job":await analyticsFeature.jobs.recover(button.dataset.jobId??"");break;
    case "recover-analysis-enqueue":analyticsActions.recoverEnqueue();break;
    case "select-analysis-job":await analyticsFeature.selectJob(button.dataset.jobId||null);break;
    case "show-job-json":analyticsActions.showJob(button.dataset.jobId??"");break;
    case "analysis-jobs":analyticsFeature.setStep("model");app.querySelector("[data-analysis-panel=jobs]")?.scrollIntoView({block:"start"});break;
    case "analysis-model":analyticsFeature.setStep("model");break;
    case "analysis-quality":analyticsFeature.setStep("quality");break;
    case "analysis-exchange":analyticsFeature.setStep("exchange");break;
    case "analysis-advanced":analyticsFeature.setStep("advanced");break;
    case "show-analysis-scope":analyticsActions.showScope();break;
    case "show-analysis-quality":showModal("分析质量来源",{source:analyticsFeature.overview?.source_snapshot,quality:analyticsFeature.overview?.data_quality,gates:analyticsFeature.gates});break;
    case "export-ai-analysis-package":analyticsActions.exportPackage();break;
    case "export-ai-response-template":analyticsActions.template();break;
    case "preview-ai-analysis-response":await analyticsFeature.exchange.chooseResponse();break;
    case "import-ai-analysis-response":analyticsActions.importResponse();break;
    case "recover-analysis-import":await analyticsFeature.exchange.recoverImport(button.dataset.receiptKey);break;
    case "show-ai-response-json":showModal("分析建议检查结果",analyticsFeature.exchange.state.preview);break;
    case "decide-ai-suggestion":analyticsActions.suggestion(button.dataset.suggestionId??"",button.dataset.decision as "accept"|"reject");break;
    case "recover-analysis-decision":await analyticsFeature.reviews.recover(button.dataset.id??"");break;
    case "show-ai-suggestion-json":showModal("智能分析建议依据",analyticsFeature.state.suggestions.find(r=>r.id===button.dataset.suggestionId));break;
    case "decide-quality-finding":analyticsActions.quality(button.dataset.findingId??"",button.dataset.decision as "resolve"|"ignore");break;
    case "show-quality-json":showModal("数据质量详情",analyticsFeature.overview?.data_quality.findings.find(r=>r.id===button.dataset.findingId));break;
    case "open-lineup":
      showModal(
        "阵容详情",
        await runBusy(() => api.readLineup(button.dataset.lineupId ?? "")),
      );
      break;
    case "refresh-runs":
      await runsFeature.refresh();
      break;
    case "runs-back": {
      const id=runsFeature.selectedId;runsFeature.back();
      if(id)workspaceState.restoreFocus("runs",app.querySelector(".page-container")??app,`run-open-${id}`);
      break;
    }
    case "runs-retry-detail":
      await openHistoryRun(button.dataset.runId??runsFeature.selectedId??"");
      break;
    case "runs-technical": {
      const doc=runsFeature.detail;
      if(!doc||runsFeature.selectedId!==button.dataset.runId)throw Error("当前记录已变化，请重新打开技术追踪");
      showHtmlModal("运行技术追踪","只读 · 原运行结果、矩阵与血缘",runTechnicalBody(doc),"","prediction-result-modal");
      break;
    }
    case "request-remove-lineup-history":
      lineupsFeature.actions.requestRemoveLineupHistory(
        button.dataset.lineupId ?? "",
        button.dataset.lineupLabel ?? "未命名阵容",
      );
      break;
    case "request-hide-run-history":
      requestHideRunHistory(
        button.dataset.runId ?? "",
        button.dataset.runLabel ?? "未命名推演",
      );
      break;
    case "open-run":
      await openHistoryRun(button.dataset.runId??"");
      break;
    case "show-route-json": {
      if(navigation.page==="prediction"&&workspaceState.module("prediction").active_section==="formal"&&predictionFeature.view.route){showRouteDetail(predictionFeature.view.route);break;}
      if (rulesFeature.route) { showRouteDetail(rulesFeature.route); break; }
      if (predictionFeature.lastResult === null) break;
      const root = objectValue(predictionFeature.lastResult);
      const route =
        Object.keys(objectValue(root.route)).length > 0
          ? objectValue(root.route)
          : root;
      if (
        typeof route.source === "string" &&
        typeof route.package_display_name === "string"
      ) {
        showRouteDetail(route as unknown as RouteDecision);
      }
      break;
    }
    case "show-last-result":
      if (predictionFeature.lastResult !== null)
        showPredictionDetail("推演完整结果", predictionFeature.lastResult);
      break;
    case "confirm-workspace-action":
      await modal.runPendingAction();
      break;
    case "close-workspace-detail":
      teamsFeature.deletion.cancelForce();
      closeModal();
      break;
  }
}

function closeAppContextMenu(): void { contextMenu.close(); }
function showAppContextMenu(event: MouseEvent,target: HTMLElement): void { contextMenu.open(event,target); }

document.addEventListener("contextmenu", (event) => {
  const target = (event.target as HTMLElement).closest<HTMLElement>("[data-context-kind]");
  if (!target) return;
  showAppContextMenu(event, target);
}, { signal: browserLifecycleController.signal });

document.addEventListener("keydown", (event) => {
  if (event.key !== "Enter" || !(event.target instanceof HTMLInputElement)) return;
  const action =
    event.target.id === "team-search"
      ? "search-teams"
      : event.target.id === "player-search"
        ? "search-players"
        : null;
  if (!action) return;
  event.preventDefault();
  document.querySelector<HTMLButtonElement>(`[data-action="${action}"]`)?.click();
}, { signal: browserLifecycleController.signal });


document.addEventListener("click", (event) => {
  const target = event.target as HTMLElement;
  if (shellController?.hasConfirmation) return;
  if (target.closest("[data-shell-controls]")) return;
  if (target.closest("button:disabled, [inert], [aria-disabled='true']")) return;
  const summary = target.closest<HTMLElement>("[data-validation-summary]");
  const formRegion = summary?.closest<HTMLElement>(".inline-setup-panel");
  if (summary && formRegion && focusValidationTarget(event, summary, formRegion)) return;
  if (!target.closest("#app-context-menu")) closeAppContextMenu();
  if (selectRulesDirectoryLevel(target)) return;
  const pageButton = target.closest<HTMLElement>("[data-page]");
  if (pageButton?.dataset.page) {
    const targetPage = pageButton.dataset.page;
    if (isRoute(targetPage)) activatePage(targetPage);
    return;
  }
  const actionButton = target.closest<HTMLElement>("[data-action]");
  if (!actionButton?.dataset.action) return;
  dispatchAction(actionButton.dataset.action, actionButton);
}, { signal: browserLifecycleController.signal });

function activatePage(targetPage: Page): void {
  const resetPlayerSource = targetPage === "players" && playersFeature.directEntry();
  if (targetPage === navigation.page && !resetPlayerSource) return;
  const navigationTraceId = recordUiOperation("navigation", `page:${targetPage}`, { fromPage: navigation.page, toPage: targetPage });
  void navigateTo(targetPage).catch((error: unknown) => {
    recordUiOperationFailure(`page:${targetPage}`, error, navigationTraceId, { fromPage: navigation.page, toPage: targetPage });
    recordClientIssue(error, `打开页面：${pageTitle(targetPage)}`,"error",navigationTraceId);
    toast(userFacingError(error), "error");
  });
}

function dispatchAction(action: string, actionButton: HTMLElement): void {
  const actionTraceId = recordUiOperation("ui_action", `action:${action}`, {
    page: navigation.page,
    dataset: { ...actionButton.dataset },
  });
  void handleAction(action, actionButton).catch((error: unknown) => {
    recordUiOperationFailure(`action:${action}`, error, actionTraceId, { page: navigation.page });
    recordClientIssue(error, `${pageTitle(navigation.page)} / ${action}`,"error",actionTraceId);
    toast(userFacingError(error), "error");
  });
}

function filterSelectOptions(
  selectId: string,
  dataKey: string,
  expected: string | null,
): void {
  const select = document.querySelector<HTMLSelectElement>(`#${selectId}`);
  if (!select) return;
  for (const option of Array.from(select.options)) {
    if (!option.value) {
      option.hidden = false;
      continue;
    }
    option.hidden = expected !== null && option.dataset[dataKey] !== expected;
  }
  if (select.selectedOptions[0]?.hidden) select.value = "";
}

document.addEventListener("input", (event) => {
  const releaseTarget=event.target as HTMLInputElement;if(navigation.page==="release"&&releaseTarget.dataset.releaseField){releaseFeature.update(releaseTarget.dataset.releaseField,releaseTarget.value);return;}
  const candidateTarget=event.target as HTMLInputElement|HTMLSelectElement;
  if(navigation.page==="analytics"&&candidateTarget.dataset.candidateNote){analyticsFeature.candidates.context.note(candidateTarget.dataset.candidateNote,candidateTarget.value);return;}

  const target = event.target as HTMLInputElement | HTMLTextAreaElement;
  if(target.dataset.postmatchNote)postmatchActions.captureNote(target.value,target.dataset.postmatchNote);
  if(target instanceof HTMLInputElement&&target.dataset.postmatchField)postmatchFeature.monitoring.update(target.dataset.postmatchField as import("./features/postmatch").MonitorFieldKey,target.value);
  if(navigation.page==="review"&&(target.matches("[data-review-fact]")||target.closest("[data-review-player]")))syncReviewManualValidation();
  if (target.id?.startsWith("p4-conflict-note-")||target.id?.startsWith("p4-evidence-")) researchFeature.draftChanged();
  if (["competition-id", "season-id", "stage-id", "competition-kind", "simulation-model-family", "simple-kickoff"].includes(target.id) || target.id.startsWith("binding-")) rulesFeature.invalidatePreview();
  if(navigation.page==="openai"&&profileControls.input(target))return;
  if(navigation.page==="api_workspace"&&aiWorkspaceActions.input(target))return;
  if (target.id === "managed-match-search" && target instanceof HTMLInputElement) {
    lineupsFeature.actions.filterManagedMatchList(target.value);
    return;
  }
  if (target.id === "rules-competition-search" && target instanceof HTMLInputElement) {
    filterRulesCompetitionCatalogue();
  }
}, { signal: browserLifecycleController.signal });

document.addEventListener("change", (event) => {
  const releaseTarget=event.target as HTMLInputElement;if(navigation.page==="release"&&releaseTarget.dataset.releaseField){releaseFeature.update(releaseTarget.dataset.releaseField,releaseTarget.value);render({preserveForm:true});return;}
  const candidateTarget=event.target as HTMLInputElement|HTMLSelectElement;
  if(navigation.page==="analytics"&&candidateTarget.dataset.candidateNote){analyticsFeature.candidates.context.note(candidateTarget.dataset.candidateNote,candidateTarget.value);return;}
  if(navigation.page==="analytics"&&candidateTarget.dataset.candidateField){analyticsFeature.candidates.context.update(candidateTarget.dataset.candidateField,candidateTarget.value);render({preserveForm:true});return;}
  const target = event.target as HTMLInputElement | HTMLSelectElement;
  if(target.dataset.postmatchField){if(target.dataset.postmatchField==="settlement")postmatchFeature.selectSettlement(target.value==="__all__"?"":target.value);else postmatchFeature.monitoring.update(target.dataset.postmatchField as import("./features/postmatch").MonitorFieldKey,target.value);return;}
  if(navigation.page==="review"&&target instanceof HTMLSelectElement&&target.closest("[data-review-picker]")){reviewFacts.capture(app,reviewFeature.state.selectedMatchId);reviewFeature.choose(target.value||null);return;}
  if(navigation.page==="review"&&(target.matches("[data-review-fact]")||target.closest("[data-review-player]")))syncReviewManualValidation();
  if (target.id?.startsWith("p4-conflict-note-")||target.id?.startsWith("p4-evidence-")) researchFeature.draftChanged();
  if (["competition-id", "season-id", "stage-id", "competition-kind", "simulation-model-family", "simple-kickoff"].includes(target.id) || target.id.startsWith("binding-")) rulesFeature.invalidatePreview();
  if (
    target instanceof HTMLInputElement &&
    target.classList.contains("player-select-checkbox")
  ) {
    const playerId = target.dataset.playerId;
    if (playerId)
      target.checked
        ? playersFeature.catalog.toggleSelection(playerId, true)
        : playersFeature.catalog.toggleSelection(playerId, false);
    persistWorkspaceSelection("players");
    render({ preserveForm: true });
    return;
  }
  if (target instanceof HTMLInputElement && target.id === "player-select-all") {
    for (const item of playersFeature.catalog.list?.items ?? [])
      target.checked
        ? playersFeature.catalog.toggleSelection(item.id, true)
        : playersFeature.catalog.toggleSelection(item.id, false);
    persistWorkspaceSelection("players");
    render({ preserveForm: true });
    return;
  }
  if (
    target instanceof HTMLInputElement &&
    target.classList.contains("team-select-checkbox")
  ) {
    const teamId = target.dataset.teamId;
    if (teamId)
      teamsFeature.catalog.toggleSelection(teamId, target.checked);
    persistWorkspaceSelection("teams");
    render({ preserveForm: true });
    return;
  }
  if (target instanceof HTMLInputElement && target.id === "team-select-all") {
    teamsFeature.catalog.selectPage(target.checked);
    persistWorkspaceSelection("teams");
    render({ preserveForm: true });
    return;
  }
  if(navigation.page==="openai"&&target instanceof HTMLSelectElement&&profileControls.input(target))return;
  if(navigation.page==="api_workspace"&&aiWorkspaceActions.change(target))return;
  if (target instanceof HTMLInputElement && target.classList.contains("preset-member-enabled")) {
    const row = target.closest<HTMLElement>("[data-preset-player-id]");
    if (row) lineupsFeature.actions.syncPresetMemberRowState(row);
    lineupsFeature.actions.updatePresetEditorSummary();
    return;
  }
  if (target instanceof HTMLSelectElement && target.id === "lineup-preset-formation") {
    lineupsFeature.actions.refreshPresetPositionOptions(true);
    return;
  }
  if (target instanceof HTMLSelectElement && target.classList.contains("preset-member-role")) {
    const row = target.closest<HTMLElement>("[data-preset-player-id]");
    if (row) lineupsFeature.actions.syncPresetMemberRowState(row);
    lineupsFeature.actions.updatePresetEditorSummary();
    return;
  }
  if (target instanceof HTMLSelectElement && target.classList.contains("preset-member-position")) {
    const row = target.closest<HTMLElement>("[data-preset-player-id]");
    if (row) lineupsFeature.actions.refreshPresetTacticalRoleSelect(row);
    lineupsFeature.actions.updatePresetEditorSummary();
    return;
  }
  if (target instanceof HTMLSelectElement && target.classList.contains("preset-member-tactical-role")) {
    lineupsFeature.actions.updatePresetEditorSummary();
    return;
  }
  if (target.id === "rule-package-file" && target instanceof HTMLInputElement) {
    const file = target.files?.[0];
    if (file) void rulesFeature.readPackage(file).catch((error: unknown) => {
      recordClientIssue(error, "赛事设置 / 读取规则包文件");
      toast(userFacingError(error), "error");
    });
    return;
  }
  if (target.id === "new-match-kickoff" && target instanceof HTMLInputElement) {
    lineupsFeature.actions.autoSelectMatchSeason();
    lineupsFeature.actions.filterMatchTeamOptions();
    return;
  }
  if (!(target instanceof HTMLSelectElement)) return;
  if(navigation.page==="workbooks"&&target.id==="exchange-match-id"){
    workspaceState.patchModule("workbooks",{selected_object_ids:target.value?[target.value]:[]});
    workspaceState.capture("workbooks",app.querySelector(".page-container")??app,true);
    return;
  }
  if (navigation.page === "workbooks" && ["match-import-mode","spreadsheet-import-mode","team-spreadsheet-import-mode"].includes(target.id)) workspaceState.capture("workbooks",app.querySelector(".page-container")??app,true);
  if (target.id === "match-import-mode") { lineupsFeature.imports.setMode(target.value as import("./types").SpreadsheetImportMode); render({preserveForm:true}); return; }
  if (target.id === "spreadsheet-import-mode") { playersFeature.changeImportMode(); return; }
  if (target.id === "team-package-import-mode" || target.id === "team-spreadsheet-import-mode") {
    teamsFeature.changeImportMode(target.id === "team-package-import-mode" ? "package" : "monthly");
    return;
  }
  if (target.matches("[data-rules-region-select]")) {
    selectCatalogueRegion(target);
    filterRulesCompetitionCatalogue();
    return;
  }
  if (target.id === "rules-competition-kind") {
    filterRulesCompetitionCatalogue();
    return;
  }
  if (target.id === "prediction-match-id") {
    const matchId = target.value;
    if (!matchId) return;
    predictionFeature.context.matchId = matchId;
    lineupsFeature.state.selectedMatchLineupChain = null;
    void runBusy(() => researchFeature.load(matchId))
      .then(() => {
        render({ preserveForm: true });
      })
      .catch((error: unknown) => {
        recordClientIssue(error, "赛事推演 / 加载单场研究工作台");
        toast(userFacingError(error), "error");
      });
    return;
  }
  if (target.id === "prediction-stored-snapshot") {
    predictionFeature.context.snapshot = target.value as LineupSnapshotType;
    lineupsFeature.state.selectedMatchLineupChain = null;
    render({ preserveForm: true });
    return;
  }
  if (target.id === "prediction-model-family") {
    predictionFeature.context.model = target.value as PredictionModelFamily;
    lineupsFeature.state.selectedMatchLineupChain = null;
    render({ preserveForm: true });
    return;
  }
  if (target.id === "explicit-rule-package-id") {
    predictionFeature.context.update({explicit_rule_package_id:target.value||null});
    render({ preserveForm: true });
    return;
  }
  if (target.id === "p4-workbench-match-id") {
    const matchId = target.value;
    if (!matchId) return;
    void runBusy(() => researchFeature.load(matchId))
      .then(() => {
        render({ preserveForm: true });
      })
      .catch((error: unknown) => {
        recordClientIssue(error, "赛事推演 / 切换单场研究历史");
        toast(userFacingError(error), "error");
      });
    return;
  }
  if (target.id === "competition-id") {
    const selected = target.selectedOptions[0];
    const kind = selected?.dataset.kind;
    const kindSelect =
      document.querySelector<HTMLSelectElement>("#competition-kind");
    if (kind && kindSelect) kindSelect.value = kind;
    rulesFeature.syncHierarchy(app, target.id);
    refreshSearchableSelects(app);
  }
  if (target.id === "season-id") {
    rulesFeature.syncHierarchy(app, target.id);
    refreshSearchableSelects(app);
  }
  if (target.id === "stage-id") {
    const kind = target.selectedOptions[0]?.dataset.kind;
    const kindSelect =
      document.querySelector<HTMLSelectElement>("#competition-kind");
    if (kind && kindSelect) kindSelect.value = kind;
  }
  if (target.id.startsWith("binding-")) {
    rulesFeature.invalidatePreview();
    rulesFeature.syncHierarchy(app, target.id);
    refreshSearchableSelects(app);
  }
  if (target.id === "new-match-competition-scope") {
    lineupsFeature.actions.updateCompetitionHierarchy("scope");
    return;
  }
  if (target.id === "new-match-competition-region") {
    lineupsFeature.actions.updateCompetitionHierarchy("region");
    return;
  }
  if (target.id === "new-match-competition") {
    lineupsFeature.actions.updateCompetitionHierarchy("competition");
    filterSelectOptions(
      "new-match-season",
      "competitionId",
      target.value || null,
    );
    filterSelectOptions(
      "new-match-stage",
      "competitionId",
      target.value || null,
    );
    const scope = document.querySelector<HTMLSelectElement>("#new-match-team-scope");
    if (scope?.value === "auto") scope.value = "auto";
    lineupsFeature.actions.autoSelectMatchSeason();
    lineupsFeature.actions.filterMatchTeamOptions();
    return;
  }
  if (target.id === "new-match-season") {
    filterSelectOptions("new-match-stage", "seasonId", target.value || null);
    lineupsFeature.actions.filterMatchTeamOptions();
    return;
  }
  if (target.id === "new-match-stage") {
    filterSelectOptions("new-match-round", "stageId", target.value || null);
    return;
  }
  if (target.id === "new-match-team-scope") {
    lineupsFeature.actions.filterMatchTeamOptions();
    return;
  }
  if (target.id === "paired-home-formation-level1" || target.id === "paired-away-formation-level1") {
    lineupsFeature.actions.updateFormationHierarchy(target.id.includes("home") ? "home" : "away", "level1");
    return;
  }
  if (target.id === "paired-home-formation-level2" || target.id === "paired-away-formation-level2") {
    lineupsFeature.actions.updateFormationHierarchy(target.id.includes("home") ? "home" : "away", "level2");
    return;
  }
  if (target.id === "paired-home-formation-id" || target.id === "paired-away-formation-id") {
    lineupsFeature.actions.updateFormationHierarchy(target.id.includes("home") ? "home" : "away", "formation");
    return;
  }
  if (target.id === "paired-lineup-match") {
    if (!target.value) return;
    lineupsFeature.actions.resetPairedBuilderForMatch(target.value, true);
    void runBusy(lineupsFeature.actions.loadBothPairedLineupSides)
      .then(([homeCount, awayCount]) => {
        render();
        toast(`双方名单已加载：主队 ${homeCount} 人，客队 ${awayCount} 人`, "success");
      })
      .catch((error: unknown) => {
        recordClientIssue(error, "阵容编排 / 加载双方名单");
        toast(userFacingError(error), "error");
      });
    return;
  }
  if (["paired-lineup-type", "paired-lineup-snapshot", "paired-home-coach", "paired-away-coach", "paired-home-quality", "paired-away-quality"].includes(target.id)) {
    lineupsFeature.actions.capturePairedLineupFromDom();
    lineupsFeature.state.selectedMatchLineupChain = null;
    return;
  }
  if (target.id === "new-lineup-match") {
    const selected = target.selectedOptions[0];
    lineupsFeature.actions.captureLineupFormFromDom({ allowBlankIdentity: true });
    lineupsFeature.state.lineupBuilderForm = {
      ...lineupsFeature.state.lineupBuilderForm,
      match_id: target.value,
      team_id: selected?.dataset.homeTeam ?? "",
    };
    lineupsFeature.state.lineupBuilderPlayers = [];
    lineupsFeature.state.lineupPlayerCandidates = [];
    lineupsFeature.state.lineupPlayerLoadSequence.invalidate();
    const teamId = lineupsFeature.state.lineupBuilderForm.team_id;
    render();
    if (teamId) void lineupsFeature.actions.autoLoadLineupPlayers(teamId);
    return;
  }
  if (["new-lineup-team", "new-lineup-type", "new-lineup-snapshot", "new-lineup-formation-id", "new-lineup-coach"].includes(target.id)) {
    const previousTeam = lineupsFeature.state.lineupBuilderForm.team_id;
    lineupsFeature.actions.captureLineupFormFromDom({
      allowBlankIdentity: target.id === "new-lineup-team",
    });
    if (
      target.id === "new-lineup-team" &&
      previousTeam !== lineupsFeature.state.lineupBuilderForm.team_id
    ) {
      lineupsFeature.state.lineupBuilderPlayers = [];
      lineupsFeature.state.lineupPlayerCandidates = [];
      lineupsFeature.state.lineupPlayerLoadSequence.invalidate();
      const teamId = lineupsFeature.state.lineupBuilderForm.team_id;
      render();
      if (teamId) void lineupsFeature.actions.autoLoadLineupPlayers(teamId);
      return;
    }
  }
}, { signal: browserLifecycleController.signal });

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") closeModal();
}, { signal: browserLifecycleController.signal });

window.addEventListener("error", (event) => {
  recordClientIssue(
    event.error ?? event.message,
    "客户端未捕获异常",
    "critical",
  );
}, { signal: browserLifecycleController.signal });

window.addEventListener("unhandledrejection", (event) => {
  recordClientIssue(event.reason, "客户端未处理的异步异常", "critical");
}, { signal: browserLifecycleController.signal });

async function initializeApplication(): Promise<void> {
  await workspaceState.initialize();
  if (browserLifecycleController.signal.aborted) return;
  teamsFeature.catalog.replaceSelection(workspaceState.module("teams").selected_object_ids);
  playersFeature.catalog.replaceSelection(new Set(workspaceState.module("players").selected_object_ids));
  await session.start();
  if (browserLifecycleController.signal.aborted) return;
  render();
  if (sessionPreferences.getItem(DATABASE_RESET_COMPLETE_KEY) === "1") {
    sessionPreferences.removeItem(DATABASE_RESET_COMPLETE_KEY);
    toast("数据库已彻底清空并重建为空白状态", "success");
  }
}

window.addEventListener("beforeunload", () => {
  if (renderedPage) {
    const currentPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
    workspaceState.capture(renderedPage, currentPageRoot, true);
  }
  void workspaceState.flush().catch(error => recordClientIssue(error, "工作区 / 退出保存"));
}, { signal: browserLifecycleController.signal });

export interface BrowserApplicationModule {
  readonly name: "browser-application";
  start(): Promise<void>;
  destroy(): Promise<void>;
  stop(): void;
}

let browserApplicationCreated = false;

export function createBrowserApplicationModule(
  root: HTMLDivElement,
): BrowserApplicationModule {
  if (browserApplicationCreated) {
    throw new Error("浏览器应用模块只能由组合根创建一次");
  }
  browserApplicationCreated = true;
  app = root;
  warningCenter = new WarningCenterController(root, { navigate: target => activatePage(target) });
  shellController = new ApplicationShellController(root, { navigate: activatePage, action: dispatchAction }, {
    sidebarCollapsed: () => workspaceState.sidebarCollapsed(),
    setSidebarCollapsed: collapsed => workspaceState.setSidebarCollapsed(collapsed),
    toggleTheme: () => session.toggleTheme(),
    renderPreservingWorkspace: () => render({ preserveForm: true }),
    historyBack: () => modal.back(),
    historyForward: () => modal.forward(),
    resetWorkspace: resetCurrentWorkspace,
    retryPageRead: () => navigateTo(navigation.page),
    openLogs: () => navigation.page === "logs" ? navigateTo("logs") : activatePage("logs"),
  });
  const lifecycle = new MountHandle({
    closedMessage: "浏览器应用生命周期已结束",
    stop: stopResources,
    async start() {
      warningCenter?.start();
      shellController?.start();
      globalShortcuts.start();
      workspaceState.start(root, () => renderedPage, () => app.querySelector<HTMLElement>(".page-container"));
      bindSearchableSelectDiagnostics(browserLifecycleController.signal);
      render();
      try {
        await initializeApplication();
      } catch (error: unknown) {
        recordClientIssue(error, "平台启动", "critical");
        throw new Error(userFacingError(error), { cause: error });
      }
    },
    async destroy() {
      try { await workspaceState.destroy(); }
      finally { try { await session.destroy(); } finally { stopTauriTransport(); } }
    },
  });

  function stopResources(): void {
    if (renderedPage) {
      const currentPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
      workspaceState.capture(renderedPage, currentPageRoot, true);
    }
    warningCenter?.destroy();
    shellController?.destroy();
    browserLifecycleController.destroy();
    databaseFeature.stop();
    rulesFeature.stop();
    teamsFeature.stop();
  playersFeature.stop();
    reviewControls.destroy();postmatchControls.destroy();candidateControls.destroy();releaseControls.destroy();
    session.stop();
    profilesFeature.stop();profileControls.stop();logsFeature.stop();
    aiWorkspaceFeature.stop();aiWorkspaceControls.stop();
    navigation.destroy();
    modal.reset();releaseDetails.destroy();
    contextMenu.destroy();
    globalShortcuts.destroy();
    commandMenu.destroy();
    workflowNavigation.clear();
    workspaceState.stop();
    lineupsFeature.stop();
  }

  return { name: "browser-application", start: () => lifecycle.start(), destroy: () => lifecycle.destroy(), stop: () => lifecycle.stop() };
}

window.addEventListener("resize", () => fitCatalogueRegions(app), { signal: browserLifecycleController.signal });
