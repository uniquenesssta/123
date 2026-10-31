import type { BootstrapResponse, PlayerCatalogReferenceData, PlayerDetail, PlayerListPage, PlayerListQuery, PlayerNavigationContext } from "../../types";
import type { PlayerApi } from "./playerPorts";
import { playerSnapshot } from "./playerSnapshot";

export function normalizePlayerQuery(stored: Partial<PlayerListQuery> | null): PlayerListQuery {
  return { search: typeof stored?.search === "string" ? stored.search : null, team_id: typeof stored?.team_id === "string" ? stored.team_id : null, position_code: typeof stored?.position_code === "string" ? stored.position_code : null, availability_status: typeof stored?.availability_status === "string" ? stored.availability_status : null, player_status: typeof stored?.player_status === "string" ? stored.player_status : "active", limit: typeof stored?.limit === "number" && stored.limit > 0 ? stored.limit : 50, cursor_name: null, cursor_id: null };
}
export function normalizePlayerSource(stored: Partial<PlayerNavigationContext> | null): PlayerNavigationContext | null {
  if (!stored || !["match_lineup", "team_roster"].includes(stored.source ?? "") || typeof stored.team_id !== "string" || typeof stored.team_name !== "string") return null;
  const now = new Date().toISOString();
  return { source: stored.source as PlayerNavigationContext["source"], team_id: stored.team_id, team_name: stored.team_name, player_id: typeof stored.player_id === "string" ? stored.player_id : null, origin_page: stored.origin_page === "lineups" ? "lineups" : "teams", return_section: stored.return_section === "builder" || stored.return_section === "chain" ? stored.return_section : null, created_at: stored.created_at ?? now, updated_at: stored.updated_at ?? now };
}
interface CatalogPorts { api: PlayerApi; snapshot(): BootstrapResponse | null; persistQuery(query: Omit<PlayerListQuery, "cursor_id" | "cursor_name">): void; persistSource(source: PlayerNavigationContext | null): void }
type Cursor = Pick<PlayerListQuery, "cursor_id" | "cursor_name">;

/** Sole owner of player identity, directory, cursor, selection, references and source context. */
export class PlayerCatalog {
  private optionSequence = 0; private sequence = 0; private epoch = 0; private closed = false; private loading = false;
  private queryValue: PlayerListQuery; private listValue: PlayerListPage | null = null; private detailValue: PlayerDetail | null = null;
  private referencesValue: PlayerCatalogReferenceData | null = null; private sourceValue: PlayerNavigationContext | null;
  private history: Cursor[] = []; private selection = new Set<string>();
  constructor(private readonly ports: CatalogPorts, query: Partial<PlayerListQuery> | null, source: Partial<PlayerNavigationContext> | null) {
    this.queryValue = normalizePlayerQuery(query); this.sourceValue = normalizePlayerSource(source); if (this.sourceValue) this.queryValue.team_id = this.sourceValue.team_id;
  }
  get query(): PlayerListQuery { return { ...this.queryValue }; }
  get list(): PlayerListPage | null { return this.listValue; }
  get detail(): PlayerDetail | null { return this.detailValue; }
  get references(): PlayerCatalogReferenceData | null { return this.referencesValue; }
  get source(): PlayerNavigationContext | null { return this.sourceValue; }
  get selectedIds(): ReadonlySet<string> { return new Set(this.selection); }
  get pageNumber(): number { return this.history.length + 1; }
  get busy(): boolean { return this.loading; }
  get revision(): number { return this.sequence; }
  get selectionKey(): string { return [...this.selection].sort().join("\0"); }
  capture(): () => boolean { const state = this.ports.snapshot(), epoch = this.epoch; return () => !this.closed && !!state?.data.database_configured && state === this.ports.snapshot() && epoch === this.epoch; }
  assertPlayer(id: string): () => boolean { if (!id || this.detailValue?.player.id !== id) throw new Error("球员上下文已变化，请重新打开目标球员"); return this.capture(); }
  private ticket(isCurrent: () => boolean) { const state = this.ports.snapshot(), sequence = ++this.sequence; if (this.closed || !state?.data.database_configured) throw new Error("请先连接数据库"); return () => !this.closed && state === this.ports.snapshot() && sequence === this.sequence && isCurrent(); }
  adopt(detail: PlayerDetail | null): void { this.sequence++; this.epoch++; this.loading = false; this.detailValue = detail ? playerSnapshot(detail) : null; }
  setReferences(references: PlayerCatalogReferenceData | null): void { this.referencesValue = references ? playerSnapshot(references) : null; }
  setSource(source: PlayerNavigationContext | null): void { this.sourceValue = source ? playerSnapshot(source) : null; this.ports.persistSource(source); }
  replaceQuery(query: PlayerListQuery): void { this.sequence++; this.epoch++; this.queryValue = { ...query }; this.history = []; this.persist(); }
  private persist(): void { const { cursor_id: _id, cursor_name: _name, ...query } = this.queryValue; this.ports.persistQuery(query); }
  replaceSelection(ids: Iterable<string>): void { this.selection = new Set(ids); }
  toggleSelection(id: string, checked: boolean): void { if (checked) this.selection.add(id); else this.selection.delete(id); }
  selectPage(checked: boolean): void { for (const item of this.listValue?.items ?? []) this.toggleSelection(item.id, checked); }
  remove(ids: Iterable<string>): void { const removed = new Set(ids); for (const id of removed) this.selection.delete(id); if (this.detailValue && removed.has(this.detailValue.player.id)) this.adopt(null); if (this.sourceValue?.player_id && removed.has(this.sourceValue.player_id)) this.setSource(null); }
  reset(): void { this.adopt(null); this.selection.clear(); this.history = []; this.setSource(null); }
  directEntry(): boolean {
    const teamId = this.sourceValue?.team_id; if (!teamId) return false;
    this.replaceQuery({ ...this.queryValue, ...(this.queryValue.team_id === teamId ? { team_id: null } : {}), cursor_name: null, cursor_id: null }); this.setSource(null); this.adopt(null); return true;
  }
  async open(id: string, isCurrent: () => boolean = () => true): Promise<boolean> {
    if (!id) throw new Error("球员身份缺失"); this.adopt(null); const current = this.ticket(isCurrent);
    try { const detail = await this.ports.api.readPlayer(id); if (!current()) return false; if (detail.player.id !== id) throw new Error("球员响应身份不匹配"); this.detailValue = playerSnapshot(detail); return true; }
    catch (error) { if (!current()) return false; throw error; }
  }
  async load(options: { query?: PlayerListQuery; resetCursor?: boolean; history?: Cursor[]; preserveDetail?: boolean; isCurrent?: () => boolean; cachedReferences?: boolean } = {}): Promise<boolean> {
    this.epoch++; const query = { ...(options.query ?? this.queryValue), ...(options.resetCursor ? { cursor_name: null, cursor_id: null } : {}) }, selected = this.detailValue;
    const current = this.ticket(options.isCurrent ?? (() => true)), sequence = this.sequence; this.loading = true;
    try {
      const [references, list] = await Promise.all([options.cachedReferences && this.referencesValue ? this.referencesValue : this.ports.api.playerCatalogReferenceData(), this.ports.api.listPlayers(query)]);
      if (!current()) return false;
      this.referencesValue = playerSnapshot(references); this.listValue = playerSnapshot(list); this.queryValue = query; this.history = options.history ?? (options.resetCursor ? [] : this.history);
      this.detailValue = options.preserveDetail || selected && list.items.some(item => item.id === selected.player.id) ? selected : null; this.persist(); return true;
    } catch (error) { if (!current()) return false; throw error; }
    finally { if (sequence === this.sequence) this.loading = false; }
  }
  async search(query: PlayerListQuery, isCurrent: () => boolean = () => true): Promise<boolean> {
    const loaded = await this.load({ query: { ...query, cursor_name: null, cursor_id: null }, history: [], isCurrent });
    if (loaded && this.sourceValue && this.sourceValue.team_id !== query.team_id) this.setSource(null); return loaded;
  }
  next(isCurrent: () => boolean = () => true): Promise<boolean> {
    const list = this.listValue; if (this.loading || !list?.has_more || !list.next_cursor_name || !list.next_cursor_id) return Promise.resolve(false);
    return this.load({ query: { ...this.queryValue, cursor_name: list.next_cursor_name, cursor_id: list.next_cursor_id }, history: [...this.history, { cursor_name: this.queryValue.cursor_name, cursor_id: this.queryValue.cursor_id }], preserveDetail: true, cachedReferences: true, isCurrent });
  }
  previous(isCurrent: () => boolean = () => true): Promise<boolean> {
    const cursor = this.history.at(-1); if (this.loading || !cursor) return Promise.resolve(false);
    return this.load({ query: { ...this.queryValue, ...cursor }, history: this.history.slice(0, -1), preserveDetail: true, cachedReferences: true, isCurrent });
  }
  async refreshDetail(id: string): Promise<boolean> {
    if (this.detailValue?.player.id !== id) return false; const current = this.ticket(() => this.detailValue?.player.id === id);
    try { const [detail, list] = await Promise.all([this.ports.api.readPlayer(id), this.ports.api.listPlayers(this.queryValue)]); if (!current()) return false; if (detail.player.id !== id) throw new Error("球员响应身份不匹配"); this.detailValue = playerSnapshot(detail); this.listValue = playerSnapshot(list); return true; }
    catch (error) { if (!current()) return false; throw error; }
  }
  async refreshReferences(): Promise<void> { const current = this.capture(), value = await this.ports.api.playerCatalogReferenceData(); if (current()) this.setReferences(value); }
  async searchTeamOptions(search: string | null): Promise<boolean> { const current=this.capture(),sequence=++this.optionSequence; const teams=await this.ports.api.listTeamOptions(search,200);if(!current()||sequence!==this.optionSequence)return false;const references=this.referencesValue??await this.ports.api.playerCatalogReferenceData();if(!current()||sequence!==this.optionSequence)return false;this.setReferences({...references,teams});return true; }
  stop(): void { this.closed = true; this.adopt(null); this.referencesValue = null; this.sourceValue = null; }
}
