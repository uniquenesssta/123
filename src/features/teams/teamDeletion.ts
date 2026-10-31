import type { EntityDeletionCheck, TeamForceDeletePreview } from "../../types";
import type { TeamCatalog } from "./teamCatalog";
import type { TeamApi } from "./teamPorts";
import { teamSnapshot } from "./teamSnapshot";

export interface TeamDeleteTicket { readonly checks: EntityDeletionCheck[]; readonly ids: string[]; readonly current: () => boolean }
interface DeletePorts { api: TeamApi; catalog: TeamCatalog; busy<T>(operation: () => Promise<T>): Promise<T> }

/** Owns backend preflight identity and single-use confirmation capabilities, not deletion policy. */
export class TeamDeletion {
  private pending: { preview: TeamForceDeletePreview; current: () => boolean } | null = null;
  private revision = 0;
  private consumed = new WeakSet<TeamDeleteTicket>();
  constructor(private readonly ports: DeletePorts) {}
  get forcePreview(): TeamForceDeletePreview | null { return this.pending?.current() ? this.pending.preview : null; }
  cancelForce(): void { this.revision++; this.pending = null; }
  async prepareNormal(): Promise<TeamDeleteTicket | null> {
    const ids = [...this.ports.catalog.selectedIds]; if (!ids.length) throw new Error("请先选择球队");
    const context = this.ports.catalog.capture(), key = this.ports.catalog.selectionKey;
    const current = () => context() && key === this.ports.catalog.selectionKey;
    try {
      const checks = await Promise.all(ids.map(id => this.ports.api.checkEntityDeletion("team", id)));
      if (!current()) return null;
      if (checks.length !== ids.length || checks.some((check, index) => check.entity_id !== ids[index])) throw new Error("删除预检响应身份不匹配");
      return Object.freeze({ ids: teamSnapshot(ids), checks: teamSnapshot(checks), current });
    } catch (error) { if (!current()) return null; throw error; }
  }
  async deleteNormal(ticket: TeamDeleteTicket) {
    if (!ticket.current() || this.consumed.has(ticket)) throw new Error("球队选择或删除预检已变化，请重新预检确认");
    const ids = ticket.checks.filter(check => check.exists && check.can_permanently_delete).map(check => check.entity_id);
    if (!ids.length) throw new Error("没有可永久删除的无引用球队");
    this.consumed.add(ticket);
    const result = await this.ports.busy(() => this.ports.api.bulkDeleteTeams(ids));
    return { result, current: ticket.current() };
  }
  async prepareForce(teamId: string): Promise<TeamForceDeletePreview | null> {
    const context = this.ports.catalog.assertTeam(teamId), revision = ++this.revision; this.pending = null;
    const current = () => context() && revision === this.revision && this.ports.catalog.detail?.team.id === teamId;
    try {
      const preview = await this.ports.api.previewForceDeleteTeam(teamId);
      if (!current()) return null;
      if (preview.team_id !== teamId) throw new Error("强制删除预检身份不匹配");
      this.pending = { preview: teamSnapshot(preview), current }; return this.pending.preview;
    } catch (error) { if (!current()) return null; throw error; }
  }
  async confirmForce(confirmation: string) {
    const ticket = this.pending, preview = this.forcePreview;
    if (!ticket || !preview) throw new Error("强制删除预检已失效，请重新打开");
    if (confirmation !== preview.confirmation_text) throw new Error(`确认文字不匹配，请完整输入：${preview.confirmation_text}`);
    this.pending = null;
    const result = await this.ports.busy(() => this.ports.api.forceDeleteTeam({ team_id: preview.team_id, confirmation_text: confirmation }));
    if (result.team_id !== preview.team_id) throw new Error("删除结果身份不匹配，请核查数据库实际状态");
    return { result, current: ticket.current() };
  }
}
