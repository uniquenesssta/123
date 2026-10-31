import type { BootstrapResponse, CoachListItem, FormationRecord, TeamDetail, TeamLineupPresetRecord, TeamListPage, TeamListQuery, TeamMatchLineupHistoryItem } from "../../types";
import type { TeamApi } from "./teamPorts";
import { teamSnapshot } from "./teamSnapshot";

interface CatalogPorts {
  api: TeamApi;
  snapshot(): BootstrapResponse | null;
  publishOptions(coaches: CoachListItem[], formations: FormationRecord[]): void;
  persistQuery(query: Omit<TeamListQuery, "cursor_name" | "cursor_id">): void;
}
export function normalizeTeamQuery(stored: Partial<TeamListQuery> | null): TeamListQuery {
  return { search: typeof stored?.search === "string" ? stored.search : null, country_code: typeof stored?.country_code === "string" ? stored.country_code : null, team_type: typeof stored?.team_type === "string" ? stored.team_type as TeamListQuery["team_type"] : null, active_only: typeof stored?.active_only === "boolean" ? stored.active_only : true, limit: typeof stored?.limit === "number" && stored.limit > 0 ? stored.limit : 50, cursor_name: null, cursor_id: null };
}

/** Sole team catalogue/detail/query/cursor/selection owner, shared through the Teams public feature. */
export class TeamCatalog {
  private sequence = 0;
  private contextRevision = 0;
  private closed = false;
  private playerSequence = 0;
  private selected = new Set<string>();
  private history: Pick<TeamListQuery, "cursor_name" | "cursor_id">[] = [];
  private currentQuery: TeamListQuery;
  private currentPage: TeamListPage | null = null;
  private currentDetail: TeamDetail | null = null;
  private lineupHistory: TeamMatchLineupHistoryItem[] = [];
  private lineupPresets: TeamLineupPresetRecord[] = [];
  constructor(private readonly ports: CatalogPorts, initial: Partial<TeamListQuery> | null) { this.currentQuery = normalizeTeamQuery(initial); }
  get query(): TeamListQuery { return { ...this.currentQuery }; }
  get list(): TeamListPage | null { return this.currentPage; }
  get detail(): TeamDetail | null { return this.currentDetail; }
  get lineups(): TeamMatchLineupHistoryItem[] { return this.lineupHistory; }
  get presets(): TeamLineupPresetRecord[] { return this.lineupPresets; }
  get selectedIds(): ReadonlySet<string> { return new Set(this.selected); }
  get pageNumber(): number { return this.history.length + 1; }
  get revision(): number { return this.sequence; }
  get selectionKey(): string { return [...this.selected].sort().join("\0"); }
  capture(): () => boolean {
    const snapshot = this.ports.snapshot(), revision = this.contextRevision;
    return () => !this.closed && snapshot !== null && snapshot.data.database_configured && snapshot === this.ports.snapshot() && revision === this.contextRevision;
  }
  assertTeam(id: string): () => boolean {
    if (!id || this.currentDetail?.team.id !== id) throw new Error("球队上下文已变化，请重新打开目标球队");
    return this.capture();
  }
  playerRead(): () => boolean { if (!this.currentDetail) throw new Error("请先选择球队"); const current = this.capture(), sequence = ++this.playerSequence; return () => current() && sequence === this.playerSequence; }
  replaceSelection(ids: Iterable<string>): void { this.selected = new Set(ids); }
  toggleSelection(id: string, checked: boolean): void { if (checked) this.selected.add(id); else this.selected.delete(id); }
  selectPage(checked: boolean): void { for (const item of this.currentPage?.items ?? []) this.toggleSelection(item.id, checked); }
  setPresets(teamId: string, presets: TeamLineupPresetRecord[]): void { if (this.currentDetail?.team.id === teamId) this.lineupPresets = teamSnapshot(presets); }
  clearDetail(): void { this.sequence++; this.contextRevision++; this.currentDetail = null; this.lineupHistory = []; this.lineupPresets = []; }
  reset(): void { this.clearDetail(); this.selected.clear(); this.history = []; }
  remove(ids: Iterable<string>): void {
    const removed = new Set(ids); for (const id of removed) this.selected.delete(id);
    if (this.currentDetail && removed.has(this.currentDetail.team.id)) this.clearDetail();
  }
  private start(isCurrent: () => boolean): () => boolean {
    const state = this.ports.snapshot(), ticket = ++this.sequence;
    if (this.closed || !state?.data.database_configured) throw new Error("请先连接数据库");
    return () => !this.closed && ticket === this.sequence && state === this.ports.snapshot() && isCurrent();
  }
  private async readDetail(id: string) {
    const [detail, lineups, presets] = await Promise.all([this.ports.api.readTeam(id), this.ports.api.listTeamMatchLineups(id, 100), this.ports.api.listTeamLineupPresets(id, true)]);
    if (detail.team.id !== id || lineups.some(item => item.lineup.team_id !== id) || presets.some(item => item.team_id !== id)) throw new Error("球队响应身份不匹配，请刷新核对");
    return { detail, lineups, presets };
  }
  private publishDetail(value: Awaited<ReturnType<TeamCatalog["readDetail"]>>): void {
    this.currentDetail = teamSnapshot(value.detail); this.lineupHistory = teamSnapshot(value.lineups); this.lineupPresets = teamSnapshot(value.presets);
  }
  async open(id: string, isCurrent: () => boolean = () => true): Promise<boolean> {
    if (!id) throw new Error("缺少球队 ID");
    this.clearDetail(); const current = this.start(isCurrent);
    try { const result = await this.readDetail(id); if (!current()) return false; this.publishDetail(result); return true; }
    catch (error) { if (!current()) return false; throw error; }
  }
  async load(options: { resetCursor?: boolean; query?: TeamListQuery; history?: Pick<TeamListQuery, "cursor_name" | "cursor_id">[]; retainId?: string | null; selectFirst?: boolean; isCurrent?: () => boolean } = {}): Promise<boolean> {
    this.contextRevision++;
    const proposed = { ...(options.query ?? this.currentQuery), ...(options.resetCursor ? { cursor_name: null, cursor_id: null } : {}) };
    const retained = options.retainId !== undefined ? options.retainId : this.currentDetail?.team.id;
    const current = this.start(options.isCurrent ?? (() => true));
    try {
      const [list, coaches, formations] = await Promise.all([this.ports.api.listTeams(proposed), this.ports.api.listCoaches({ search: null, active_only: false, limit: 500 }), this.ports.api.listFormations(false)]);
      if (!current()) return false;
      const id = retained ?? (options.selectFirst === false ? null : list.items[0]?.id);
      const detail = id ? await this.readDetail(id) : null;
      if (!current()) return false;
      this.currentPage = teamSnapshot(list); this.currentQuery = proposed;
      this.history = options.history ?? (options.resetCursor ? [] : this.history);
      if (detail) this.publishDetail(detail); else { this.currentDetail = null; this.lineupHistory = []; this.lineupPresets = []; }
      this.ports.publishOptions(coaches, formations);
      const { cursor_name: _name, cursor_id: _id, ...stable } = proposed; this.ports.persistQuery(stable);
      return true;
    } catch (error) { if (!current()) return false; throw error; }
  }
  search(query: TeamListQuery): Promise<boolean> { return this.load({ query: { ...query, cursor_name: null, cursor_id: null }, history: [], retainId: null, selectFirst: false }); }
  next(): Promise<boolean> {
    const page = this.currentPage; if (!page?.has_more || !page.next_cursor_name || !page.next_cursor_id) return Promise.resolve(false);
    return this.load({ query: { ...this.currentQuery, cursor_name: page.next_cursor_name, cursor_id: page.next_cursor_id }, history: [...this.history, { cursor_name: this.currentQuery.cursor_name, cursor_id: this.currentQuery.cursor_id }], retainId: null, selectFirst: false });
  }
  previous(): Promise<boolean> {
    const cursor = this.history.at(-1); if (!cursor) return Promise.resolve(false);
    return this.load({ query: { ...this.currentQuery, ...cursor }, history: this.history.slice(0, -1), retainId: null, selectFirst: false });
  }
  refreshSelected(): Promise<boolean> { return this.currentDetail ? this.refreshDetail(this.currentDetail.team.id) : Promise.resolve(false); }
  async refreshDetail(id: string): Promise<boolean> {
    if (this.currentDetail?.team.id !== id) return false;
    const current = this.start(() => this.currentDetail?.team.id === id);
    try { const [detail, list] = await Promise.all([this.readDetail(id), this.ports.api.listTeams(this.currentQuery)]); if (!current()) return false; this.publishDetail(detail); this.currentPage = teamSnapshot(list); return true; }
    catch (error) { if (!current()) return false; throw error; }
  }
  stop(): void { this.closed = true; this.clearDetail(); }
}
