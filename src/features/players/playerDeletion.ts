import type { EntityDeletionCheck } from "../../types";
import type { PlayerCatalog } from "./playerCatalog";
import type { PlayerApi } from "./playerPorts";
import { playerSnapshot } from "./playerSnapshot";
export interface PlayerDeleteTicket { readonly ids: readonly string[]; readonly current: () => boolean; readonly check?: EntityDeletionCheck }
interface DeletionPorts { api: PlayerApi; catalog: PlayerCatalog; busy<T>(operation: () => Promise<T>): Promise<T> }
/** Confirmation capability, stable input and result validation; backend owns reference/transaction policy. */
export class PlayerDeletion {
  private consumed = new WeakSet<PlayerDeleteTicket>();
  constructor(private readonly ports: DeletionPorts) {}
  prepareBulk(): PlayerDeleteTicket {
    const ids = [...this.ports.catalog.selectedIds], key = this.ports.catalog.selectionKey, current = this.ports.catalog.capture(); if (!ids.length) throw new Error("请先选择球员");
    return Object.freeze({ ids: playerSnapshot(ids), current: () => current() && key === this.ports.catalog.selectionKey });
  }
  async prepareSingle(id: string): Promise<PlayerDeleteTicket | null> {
    const current = this.ports.catalog.assertPlayer(id);
    try { const check = await this.ports.api.checkEntityDeletion("player", id); if (!current()) return null; if (check.entity_id !== id) throw new Error("球员删除预检身份不匹配"); return Object.freeze({ ids: Object.freeze([id]), current, check: playerSnapshot(check) }); }
    catch (error) { if (!current()) return null; throw error; }
  }
  private consume(ticket: PlayerDeleteTicket): void { if (!ticket.current() || this.consumed.has(ticket)) throw new Error("球员选择或确认对象已变化，请重新确认"); this.consumed.add(ticket); }
  async bulk(ticket: PlayerDeleteTicket) {
    this.consume(ticket); const result = await this.ports.busy(() => this.ports.api.bulkDeletePlayers([...ticket.ids]));
    if (result.deleted_ids.some(id => !ticket.ids.includes(id))) throw new Error("球员删除结果包含其他对象，请核查原请求"); return { result, current: ticket.current() };
  }
  async single(ticket: PlayerDeleteTicket) {
    if (!ticket.check?.exists || !ticket.check.can_permanently_delete) throw new Error("该球员存在历史引用或已不存在，不能永久删除");
    this.consume(ticket); await this.ports.busy(() => this.ports.api.deletePlayer(ticket.ids[0])); return ticket.current();
  }
  async archive() {
    const ticket = this.prepareBulk(); this.consume(ticket); const result = await this.ports.busy(() => this.ports.api.bulkArchiveEntities("player", [...ticket.ids]));
    if ([...result.archived_ids, ...result.already_archived_ids].some(id => !ticket.ids.includes(id))) throw new Error("球员归档结果包含其他对象"); return { result, current: ticket.current() };
  }
}
