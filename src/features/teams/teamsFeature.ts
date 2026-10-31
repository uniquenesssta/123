import type { ModalController } from "../../app/modal";
import type { BootstrapResponse, BulkArchiveResult, BulkDeleteResult, CoachListItem, FormationRecord, Page, TeamForceDeleteResult, TeamListQuery } from "../../types";
import { TeamCatalog, normalizeTeamQuery } from "./teamCatalog";
import { TeamDeletion } from "./teamDeletion";
import { forceDeleteBody, forceDeleteResultSummary, teamBulkDeleteSummary, teamDeletionCheckSummary } from "./teamDeletionView";
import { TeamDrafts } from "./teamDrafts";
import { TeamEditor, teamValue } from "./teamEditor";
import { TeamImports } from "./teamImports";
import type { TeamApi } from "./teamPorts";

export interface TeamsFeaturePorts {
  api: TeamApi; initialQuery: Partial<TeamListQuery> | null;
  snapshot(): BootstrapResponse | null; page(): Page; root(): ParentNode;
  busy<T>(operation: () => Promise<T>): Promise<T>;
  persistQuery(query: Omit<TeamListQuery, "cursor_name" | "cursor_id">): void;
  publishOptions(coaches: CoachListItem[], formations: FormationRecord[]): void;
  publishCoaches(coaches: CoachListItem[]): void;
  refreshPlayerReferences(): Promise<void>;
  refreshAll(): Promise<void>;
  clearPlayerInspector(): void;
  render(): void; notify(message: string, kind?: "normal" | "success" | "error"): void;
  setSection(section: "directory" | "profile", inspectorCollapsed?: boolean): void;
  appendTab(id: string, label: string): void;
  activeTab(page: "teams" | "lineup_presets"): string | null;
  setPresetTab(id: string): void;
  navigateTeams(): Promise<void>;
  confirm: ModalController["confirm"];
  showHtml(title: string, subtitle: string, body: string, actions?: string): void;
  closePanel(): void;
  afterDelete(result: BulkDeleteResult, missing: string[]): Promise<boolean>;
  afterForceDelete(result: TeamForceDeleteResult): Promise<boolean>;
  afterArchive(result: BulkArchiveResult): Promise<void>;
  removeMissing(ids: string[]): Promise<void>;
}

/** Composes team-specific workflows with the existing Router/Workspace/Modal and real transport. */
export class TeamsFeature {
  readonly catalog: TeamCatalog;
  readonly drafts = new TeamDrafts();
  readonly imports: TeamImports;
  readonly deletion: TeamDeletion;
  readonly editor: TeamEditor;
  constructor(private readonly ports: TeamsFeaturePorts) {
    this.catalog = new TeamCatalog(ports, ports.initialQuery);
    this.imports = new TeamImports({ ...ports, capture: () => this.catalog.capture() });
    this.deletion = new TeamDeletion({ ...ports, catalog: this.catalog });
    this.editor = new TeamEditor({ ...ports, catalog: this.catalog, drafts: this.drafts });
  }
  async load(resetCursor = false): Promise<boolean> { const page = this.ports.page(); return this.catalog.load({ resetCursor, isCurrent: () => this.ports.page() === page }); }
  async loadForRoute(page: "teams" | "lineup_presets", isCurrent: () => boolean): Promise<void> {
    const retainId = page === "lineup_presets" ? this.ports.activeTab(page) : this.catalog.detail?.team.id ?? this.ports.activeTab(page);
    if (await this.catalog.load({ resetCursor: this.catalog.list === null, retainId, selectFirst: page !== "lineup_presets", isCurrent }) && page === "lineup_presets" && this.catalog.detail && isCurrent()) this.ports.setPresetTab(this.catalog.detail.team.id);
  }
  async open(teamId: string, section: "directory" | "profile" = "directory", activate = false): Promise<boolean> {
    const page = this.ports.page(); this.drafts.capture(this.ports.root()); this.imports.clear(); this.deletion.cancelForce();
    const pending = this.catalog.open(teamId, () => this.ports.page() === page);
    if (page === "teams") this.ports.render();
    try {
      if (!await pending || !this.catalog.detail) return false;
      this.ports.clearPlayerInspector(); this.ports.appendTab(teamId, this.catalog.detail.team.canonical_name); this.ports.setSection(section, section === "profile" ? true : undefined);
      if (activate && this.ports.page() !== "teams") await this.ports.navigateTeams(); else this.ports.render();
      return true;
    } catch (error) { if (this.ports.page() === page) this.ports.render(); throw error; }
  }
  async readPlayerFromTeam(playerId: string) {
    const team = this.catalog.detail; if (!team) throw new Error("请先选择球队");
    const context = this.catalog.playerRead(), page = this.ports.page();
    const current = () => context() && page === this.ports.page();
    try {
      const player = await this.ports.api.readPlayer(playerId);
      if (!current()) return null;
      if (player.player.id !== playerId) throw new Error("球员响应身份不匹配");
      return { player, teamId: team.team.id, teamName: team.team.canonical_name, current };
    } catch (error) { if (!current()) return null; throw error; }
  }
  async selectPresetTeam(teamId: string): Promise<void> {
    const page = this.ports.page(); this.imports.clear(); this.deletion.cancelForce();
    if (await this.catalog.open(teamId, () => this.ports.page() === page)) { this.ports.clearPlayerInspector(); this.ports.setPresetTab(teamId); this.ports.render(); }
  }
  async openProfile(teamId: string): Promise<void> { await this.open(teamId, "profile"); }
  directory(): void { this.drafts.capture(this.ports.root()); this.catalog.clearDetail(); this.imports.clear(); this.deletion.cancelForce(); this.ports.clearPlayerInspector(); this.ports.setSection("directory"); this.ports.render(); }
  reset(): void { this.catalog.reset(); this.imports.clear(); this.deletion.cancelForce(); this.drafts.clear(this.ports.snapshot()); }
  async search(returnToDirectory = false): Promise<void> {
    const root = this.ports.root(), query: TeamListQuery = { ...this.catalog.query, search: teamValue(root, "team-search") || null, country_code: teamValue(root, "team-filter-country") || null, team_type: (teamValue(root, "team-filter-type") || null) as TeamListQuery["team_type"], active_only: root.querySelector<HTMLInputElement>("#team-filter-active")?.checked ?? true };
    if (!await this.catalog.search(query)) return;
    if (returnToDirectory) { this.directory(); return; } this.ports.clearPlayerInspector(); this.ports.render();
  }
  async clearFilters(returnToDirectory = false): Promise<void> {
    const query = { ...normalizeTeamQuery(null), limit: this.catalog.query.limit };
    if (!await this.catalog.search(query)) return;
    const root = this.ports.root(); for (const id of ["team-search", "team-filter-country", "team-filter-type"]) { const field = root.querySelector<HTMLInputElement>(`#${id}`); if (field) field.value = ""; }
    const active = root.querySelector<HTMLInputElement>("#team-filter-active"); if (active) active.checked = true;
    if (returnToDirectory) this.directory(); else { this.ports.clearPlayerInspector(); this.ports.render(); }
  }
  async next(): Promise<void> { if (await this.catalog.next()) { this.ports.clearPlayerInspector(); this.ports.render(); } }
  async previous(): Promise<void> { if (await this.catalog.previous()) { this.ports.clearPlayerInspector(); this.ports.render(); } }
  async create(): Promise<void> {
    const root = this.ports.root(), draft = { canonical_name: teamValue(root, "new-team-name"), country_code: teamValue(root, "new-team-country") || null, metadata: {} }, current = this.catalog.capture();
    await this.ports.busy(() => this.ports.api.createTeam(draft));
    if (!current()) return; await this.ports.refreshAll(); this.ports.render(); this.ports.notify("球队已创建", "success");
  }
  changeImportMode(kind: "package" | "monthly"): void { if (kind === "package") this.imports.clearPackage(); else this.imports.clearMonthly(); this.ports.render(); }
  async previewPackage(): Promise<void> {
    const mode = teamValue(this.ports.root(), "team-package-import-mode") as "add_and_update" | "add_only";
    if (!await this.imports.previewPackage(mode)) return;
    const preview = this.imports.package; if (!preview) return;
    const blocking = (preview.team_preview?.counts.conflict ?? 0) + (preview.team_preview?.counts.error ?? 0) + (preview.player_preview?.counts.conflict ?? 0) + (preview.player_preview?.counts.error ?? 0) + preview.coverage.blockers.length;
    this.ports.render(); this.ports.notify(blocking ? `统一预检完成：仍有 ${blocking} 项需要处理` : `统一预检通过，P4 就绪度 ${preview.coverage.readiness_score}/100`, blocking ? "error" : "success");
  }
  async resolvePackage(scope: "team" | "player", rowId: string, selectedEntityId: string | null, skip: boolean): Promise<void> {
    if (await this.imports.resolvePackage(scope, { row_id: rowId, selected_entity_id: selectedEntityId, skip })) { this.ports.render(); this.ports.notify(skip ? "该行已标记跳过" : "冲突关联已确认", "success"); }
  }
  async commitPackage(): Promise<void> {
    const outcome = await this.imports.commitPackage(); if (!outcome) return;
    if (!outcome.current) { this.ports.notify("原资料包提交已返回；当前上下文已切换，请刷新核对原批次"); return; }
    const result = outcome.result;
    if (result.error_count === 0) await this.ports.refreshAll();
    this.ports.render(); this.ports.notify(`完整资料包${result.error_count ? "部分处理，仍有错误" : "导入完成"}：新增 ${result.inserted_count}，更新 ${result.updated_count}，结束旧记录 ${result.ended_previous_count}`, result.error_count ? "error" : "success");
  }
  async exportPackagePreview(): Promise<void> { const result = await this.imports.exportPackage(); if (result) this.ports.notify(`完整预检 JSON 已导出：${result.output_path}（${result.exported_row_count} 条记录）`, "success"); }
  async previewMonthly(): Promise<void> {
    const mode = teamValue(this.ports.root(), "team-spreadsheet-import-mode") as "add_and_update" | "add_only";
    if (!await this.imports.previewMonthly(mode)) return;
    const preview = this.imports.monthly; if (!preview) return;
    const blocking = preview.counts.conflict + preview.counts.error; this.ports.render(); this.ports.notify(blocking ? `预检完成：发现 ${blocking} 条需要处理` : "球队月度工作簿预检通过", blocking ? "error" : "success");
  }
  async resolveMonthly(rowId: string, selectedEntityId: string | null, skip: boolean): Promise<void> { if (await this.imports.resolveMonthly({ row_id: rowId, selected_entity_id: selectedEntityId, skip })) { this.ports.render(); this.ports.notify(skip ? "该行已标记跳过" : "冲突关联已确认", "success"); } }
  async commitMonthly(): Promise<void> {
    const outcome = await this.imports.commitMonthly(); if (!outcome) return;
    if (!outcome.current) { this.ports.notify("原月度批次提交已返回；当前上下文已切换，请刷新核对"); return; }
    const result = outcome.result;
    if (result.error_count === 0) await this.ports.refreshAll();
    this.ports.render();
    this.ports.notify(`月度批次${result.error_count ? "部分处理" : "导入完成"}：新增 ${result.inserted_count}，更新 ${result.updated_count}，结束旧记录 ${result.ended_previous_count}`, result.error_count ? "error" : "success");
  }
  async exportFile(kind: "package-template" | "monthly-template" | "monthly-data"): Promise<void> {
    const path = await this.ports.api.chooseExcelExportFile(kind === "package-template" ? "球队完整资料包.xlsx" : kind === "monthly-template" ? "球队月度更新模板.xlsx" : "球队月度更新.xlsx"); if (!path) return;
    const result = await this.ports.busy<{ output_path: string }>(() => kind === "package-template" ? this.ports.api.exportTeamPackageTemplate(path) : kind === "monthly-template" ? this.ports.api.exportTeamMonthlyTemplate(path) : this.ports.api.exportTeamMonthlyData(path));
    this.ports.notify(kind === "package-template" ? `标准资料包已导出：${result.output_path}` : kind === "monthly-template" ? `模板已导出：${result.output_path}` : `已导出 ${"team_count" in result ? result.team_count : "—"} 支球队，关联 ${"related_row_count" in result ? result.related_row_count : "—"} 行`, "success");
  }
  async requestDelete(): Promise<void> {
    const ticket = await this.deletion.prepareNormal(); if (!ticket) return;
    const missing = ticket.checks.filter(check => !check.exists).map(check => check.entity_id), deletable = ticket.checks.filter(check => check.exists && check.can_permanently_delete), protectedChecks = ticket.checks.filter(check => check.exists && !check.can_permanently_delete);
    if (!deletable.length) {
      if (missing.length) await this.ports.removeMissing(missing);
      const missingNotice = missing.length ? `<p>另有 ${missing.length} 个对象已不存在，已从标签页和选择中清除。</p>` : "";
      this.ports.showHtml("所选球队不能永久删除", "请使用归档", `<div class="route-detail"><div class="route-detail-summary"><span>永久删除预检</span><h3>没有可永久删除的球队</h3><p>完整资料包导入后形成的球员效力、教练任期、阵型与评分观察都属于历史或业务引用。为保证 P4 输入可追溯，这些球队只能归档，不能连同关系记录一起删除。</p>${missingNotice}</div><section class="prediction-detail-section"><h3>引用明细</h3>${teamDeletionCheckSummary(protectedChecks)}</section></div>`); return;
    }
    this.ports.confirm("永久删除无引用球队", "只会删除没有任何业务或历史引用的空球队。存在历史或业务引用的球队将保留；提交时后端再次核对引用。", [["原选择", `${ticket.ids.length} 支球队`], ["可永久删除", `${deletable.length} 支球队`], ["受保护", `${protectedChecks.length} 支球队，请使用归档`]], "确认永久删除无引用球队", async () => {
      const outcome = await this.deletion.deleteNormal(ticket);
      if (!outcome.current) { this.ports.notify("原球队删除操作已返回；当前上下文已切换，请刷新核对"); return; }
      if (!await this.ports.afterDelete(outcome.result, missing)) return;
      const combined: BulkDeleteResult = { requested_count: ticket.ids.length, deleted_ids: outcome.result.deleted_ids, blocked: [...protectedChecks.map(check => ({ id: check.entity_id, label: check.label, reason: check.reason })), ...outcome.result.blocked] };
      this.ports.showHtml("球队永久删除结果", combined.blocked.length ? "受引用球队已保留" : "删除完成", teamBulkDeleteSummary(combined));
    });
  }
  async requestForceDeleteTeam(teamId: string): Promise<void> {
    const preview = await this.deletion.prepareForce(teamId); if (!preview) return;
    this.ports.showHtml(`强制删除：${preview.label}`, "最高危险等级", forceDeleteBody(preview), '<button type="button" class="secondary" data-action="close-workspace-detail">取消</button><button type="button" class="primary danger-action" data-action="confirm-force-delete-team">确认强制删除全部资料</button>');
    this.ports.root().querySelector<HTMLInputElement>("#force-delete-team-confirmation")?.focus();
  }
  async confirmForceDeleteTeam(): Promise<void> {
    const confirmation = teamValue(this.ports.root(), "force-delete-team-confirmation"), outcome = await this.deletion.confirmForce(confirmation);
    if (!outcome.current) { this.ports.notify("原球队强制删除已返回；当前上下文已切换，请核查原库结果"); return; }
    this.ports.closePanel(); this.imports.clear(); if (!await this.ports.afterForceDelete(outcome.result)) return;
    this.ports.showHtml("球队及全部资料已强制删除", "事务已提交", forceDeleteResultSummary(outcome.result));
  }
  async archive(): Promise<void> {
    const ids = [...this.catalog.selectedIds], key = this.catalog.selectionKey, current = this.catalog.capture();
    if (!ids.length) throw new Error("请先选择要归档的球队");
    const result = await this.ports.busy(() => this.ports.api.bulkArchiveEntities("team", ids));
    if (!current() || key !== this.catalog.selectionKey) { this.ports.notify("原球队归档已返回；当前选择已变化，请刷新核对"); return; }
    await this.ports.afterArchive(result);
  }
  stop(): void { this.imports.clear(); this.deletion.cancelForce(); this.catalog.stop(); this.drafts.clear(this.ports.snapshot()); }
}
