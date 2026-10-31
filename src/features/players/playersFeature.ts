import type { ModalController } from "../../app/modal";
import { bulkArchiveSummary, bulkDeleteSummary } from "../../components/entityDeletion";
import { escapeHtml } from "../../components/format";
import type { BootstrapResponse, Page, PlayerDetail, PlayerListQuery, PlayerNavigationContext, SpreadsheetImportMode } from "../../types";
import { PlayerCatalog, normalizePlayerQuery } from "./playerCatalog";
import { PlayerDeletion } from "./playerDeletion";
import { refreshPlayerPageDom, setPlayerPageLoading } from "./playerDirectoryDom";
import { PlayerDrafts } from "./playerDrafts";
import { PlayerEditor } from "./playerEditor";
import { PlayerImports } from "./playerImports";
import type { PlayerApi } from "./playerPorts";

export interface PlayersPorts {
  api: PlayerApi; initialQuery: Partial<PlayerListQuery> | null; initialSource: Partial<PlayerNavigationContext> | null;
  snapshot(): BootstrapResponse | null; page(): Page; root(): ParentNode;
  busy<T>(operation: () => Promise<T>): Promise<T>; persistQuery(query: Omit<PlayerListQuery, "cursor_name" | "cursor_id">): void; persistSource(source: PlayerNavigationContext | null): void;
  render(): void; notify(message: string, kind?: "normal" | "success" | "error"): void;
  section(id: "directory" | "profile", collapsed?: boolean): void; appendTab(id: string, label: string): void;
  refreshAll(): Promise<void>; afterMutation(ids: string[]): Promise<boolean>;
  confirm: ModalController["confirm"]; showHtml(title: string, subtitle: string, body: string): void; showResult(title: string, result: unknown): void;
}
export class PlayersFeature {
  readonly catalog: PlayerCatalog; readonly drafts = new PlayerDrafts(); readonly editor: PlayerEditor; readonly imports: PlayerImports; readonly deletion: PlayerDeletion;
  constructor(private readonly ports: PlayersPorts) { this.catalog = new PlayerCatalog(ports, ports.initialQuery, ports.initialSource); this.editor = new PlayerEditor({ ...ports, catalog: this.catalog, drafts: this.drafts, refreshAfterCreate: () => ports.afterMutation([]) }); this.imports = new PlayerImports({ ...ports, capture: () => this.catalog.capture() }); this.deletion = new PlayerDeletion({ ...ports, catalog: this.catalog }); }
  next(): Promise<void> { return this.paginate("next"); }
  previous(): Promise<void> { return this.paginate("previous"); }
  openDirectory(id: string): Promise<void> { return this.open(id, false); }
  openProfile(id: string): Promise<void> { return this.open(id, true); }
  exportTemplate(): Promise<void> { return this.exportFile(true); }
  exportData(): Promise<void> { return this.exportFile(false); }
  async load(resetCursor = false, isCurrent: () => boolean = () => true): Promise<boolean> { return this.catalog.load({ resetCursor, isCurrent }); }
  directEntry(): boolean { const changed=this.catalog.directEntry(); if (changed) { this.imports.clear(); this.ports.section("directory", true); } return changed; }
  async open(id: string, profile = false): Promise<void> {
    const page = this.ports.page(); this.drafts.capture(this.ports.root()); this.imports.clear(); const pending = this.catalog.open(id, () => page === this.ports.page()); this.ports.render();
    try { if (!await pending || !this.catalog.detail) return; this.ports.appendTab(id, this.catalog.detail.player.canonical_name); this.ports.section(profile ? "profile" : "directory", profile); this.ports.render(); }
    catch (error) { if (page === this.ports.page()) this.ports.render(); throw error; }
  }
  enterSource(source: PlayerNavigationContext, detail: PlayerDetail): void { if (source.player_id !== detail.player.id) throw new Error("来源球员身份不匹配"); this.catalog.replaceQuery({ ...normalizePlayerQuery(null), limit: this.catalog.query.limit, player_status: null, team_id: source.team_id }); this.catalog.setSource(source); this.catalog.adopt(detail); this.imports.clear(); }
  async search(): Promise<void> {
    const root = this.ports.root(), field = (id: string) => root.querySelector<HTMLInputElement | HTMLSelectElement>("#" + id)?.value.trim() || null;
    const query = { ...this.catalog.query, search: field("player-search"), team_id: field("player-filter-team"), position_code: field("player-filter-position"), availability_status: field("player-filter-availability") as PlayerListQuery["availability_status"], player_status: field("player-filter-status") as PlayerListQuery["player_status"] };
    const page = this.ports.page(); if (!await this.ports.busy(() => this.catalog.search(query, () => page === this.ports.page())) || page !== this.ports.page()) return;
    if (!this.catalog.detail) this.ports.section("directory", true); this.ports.render();
  }
  async clearFilters(): Promise<void> { const page = this.ports.page(); if (!await this.ports.busy(() => this.catalog.search({ ...normalizePlayerQuery(null), limit: this.catalog.query.limit, player_status: null }, () => page === this.ports.page())) || page !== this.ports.page()) return; this.catalog.setSource(null); if (!this.catalog.detail) this.ports.section("directory", true); this.ports.render(); }
  private domState() { return { page: this.ports.page(), list: this.catalog.list, detail: this.catalog.detail, query: this.catalog.query, selectedIds: this.catalog.selectedIds, pageNumber: this.catalog.pageNumber, loading: this.catalog.busy }; }
  async paginate(direction: "next" | "previous"): Promise<void> {
    const page = this.ports.page(), task = this.catalog[direction](() => page === this.ports.page()); setPlayerPageLoading(this.domState(), true);
    try { if (!await task || page !== this.ports.page()) return; if (!refreshPlayerPageDom(this.domState())) this.ports.render(); }
    finally { if (page === this.ports.page()) setPlayerPageLoading(this.domState(), this.catalog.busy); }
  }
  changeImportMode(): void { this.imports.clear(); this.ports.render(); }
  async previewImport(): Promise<void> { const mode = this.ports.root().querySelector<HTMLSelectElement>("#spreadsheet-import-mode")?.value as SpreadsheetImportMode; if (!await this.imports.read(mode)) return; const preview = this.imports.preview; if (!preview) return; this.ports.render(); const blocking = preview.counts.conflict + preview.counts.error; this.ports.notify(blocking ? `预检完成：发现 ${blocking} 条需要修正` : "预检通过，可以确认导入", blocking ? "error" : "success"); }
  async resolveImport(rowId: string, selectedEntityId: string | null, skip: boolean): Promise<void> { if (await this.imports.resolve({ row_id: rowId, selected_entity_id: selectedEntityId, skip })) { this.ports.render(); this.ports.notify(skip ? "该行已标记跳过" : "冲突关联已确认", "success"); } }
  async commitImport(): Promise<void> { const outcome = await this.imports.commit(); if (!outcome) return; if (!outcome.current) { this.ports.notify("原球员批次提交已返回；当前上下文已变化，请核对原批次"); return; } const result = outcome.result; if (!result.error_count) await this.ports.refreshAll(); this.ports.render(); this.ports.notify(`${result.error_count ? "部分处理，仍有失败项" : "导入完成"}：新增 ${result.inserted_count}，更新 ${result.updated_count}，结束旧记录 ${result.ended_previous_count}，错误 ${result.error_count}`, result.error_count ? "error" : "success"); }
  async exportFile(template: boolean): Promise<void> { const path = await this.ports.api.chooseExcelExportFile(template ? "球员月度更新模板.xlsx" : "球员月度更新.xlsx"); if (!path) return; const result = await this.ports.busy(() => template ? this.ports.api.exportPlayerCatalogTemplate(path) : this.ports.api.exportPlayerCatalogData(path)); this.ports.notify(template ? `模板已导出：${result.output_path}` : `已导出 ${result.player_count} 名球员和 ${result.team_count} 支球队`, "success"); }
  requestBulkDelete(): void { const ticket = this.deletion.prepareBulk(); this.ports.confirm("批量删除球员", "将逐个执行安全删除；存在不可删除关联的球员会被保留并显示原因。", [["已选择", `${ticket.ids.length} 名球员`], ["处理方式", "逐条事务、保留失败项"]], "确认批量删除", async () => { const outcome = await this.deletion.bulk(ticket); if (!outcome.current) { this.ports.notify("原球员删除已返回；当前对象已变化，请核对原结果"); return; } if (await this.ports.afterMutation(outcome.result.deleted_ids)) this.ports.showHtml("球员批量删除结果", outcome.result.blocked.length ? "部分对象受保护，已保留" : "安全删除", bulkDeleteSummary(outcome.result, "名球员")); }); }
  async requestDelete(id: string, name: string): Promise<void> { const ticket = await this.deletion.prepareSingle(id); if (!ticket?.check) return; const check = ticket.check;
    if (!check.exists || !check.can_permanently_delete) { const references = check.references.filter(item => item.count > 0).map(item => `<li>${escapeHtml(item.relation)}：${item.count}</li>`).join(""); this.ports.showHtml("球员不能永久删除", "请改用归档", `<div class="route-detail"><div class="route-detail-summary"><span>历史保护</span><h3>${escapeHtml(name)}</h3><p>${escapeHtml(check.reason)}</p></div><section class="prediction-detail-section"><h3>引用统计</h3>${references ? `<ul>${references}</ul>` : "<p>存在受保护引用。</p>"}</section></div>`); return; }
    this.ports.confirm("永久删除空球员", "该球员尚未进入正式履历、阵容、观察或复盘链路；删除后仅保留审计记录。", [["球员", name], ["引用检查", "未发现历史引用"]], "永久删除球员", async () => { if (await this.deletion.single(ticket) && await this.ports.afterMutation([id])) this.ports.notify("空球员对象已永久删除", "success"); });
  }
  async archive(): Promise<void> { const outcome = await this.deletion.archive(); if (!outcome.current) { this.ports.notify("原球员归档已返回；当前选择已变化，请核对原对象"); return; } if (await this.ports.afterMutation([...outcome.result.archived_ids, ...outcome.result.already_archived_ids])) this.ports.showHtml("球员批量归档结果", "保留历史关系", bulkArchiveSummary(outcome.result, "名球员")); }
  async searchTeamOptions(): Promise<void> { const search=this.ports.root().querySelector<HTMLInputElement>("#team-option-search")?.value.trim()||null; if(await this.ports.busy(()=>this.catalog.searchTeamOptions(search))){this.ports.render();this.ports.notify(`已加载 ${this.catalog.references?.teams.length??0} 支球队选项`,"success");} }
  reset(): void { this.catalog.reset(); this.imports.clear(); this.drafts.clear(this.ports.snapshot()); }
  stop(): void { this.catalog.stop(); this.imports.clear(); this.drafts.clear(this.ports.snapshot()); }
}
