import type { SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution, TeamPackageImportPreview } from "../../types";
import type { TeamApi } from "./teamPorts";
import { teamSnapshot } from "./teamSnapshot";
import { SpreadsheetBatch, type ImportRecovery, type ImportTicket } from "../../app/imports/spreadsheetBatch";

interface ImportPorts { api: TeamApi; capture(): () => boolean; captureView?(): () => boolean; changed?(): void; recovery?: ImportRecovery; busy<T>(operation: () => Promise<T>): Promise<T> }

/** Owns package and monthly batch identities. R13-09 consumes this same owner. No client-side ledger. */
export class TeamImports {
  private packageValue: TeamPackageImportPreview | null = null;
  readonly monthlyBatch: SpreadsheetBatch;
  private packageContext = () => false;
  private packageSequence = 0;
  private packageBusy: number | null = null;
  constructor(private readonly ports: ImportPorts) {
    this.monthlyBatch = new SpreadsheetBatch({ ...ports, label:"球队月度", choose:()=>ports.api.chooseExcelImportFile(),
      preview:(path, mode)=>ports.api.previewTeamMonthlyImport(path, mode),
      resolve:(id, resolution)=>ports.api.resolveTeamMonthlyImportConflict(id, resolution),
      commit:id=>ports.api.commitTeamMonthlyImport(id), read:id=>ports.api.readTeamMonthlyImportPreview(id) });
  }
  get package(): TeamPackageImportPreview | null { return this.packageContext() ? this.packageValue : null; }
  get monthly(): SpreadsheetImportPreview | null { return this.monthlyBatch.preview; }
  clearPackage(): void { this.packageSequence++; this.packageValue = null; }
  clearMonthly(): void { this.monthlyBatch.clear(); }
  clear(): void { this.clearPackage(); this.clearMonthly(); }
  private packageTicket() {
    const value = this.package, revision = this.packageSequence;
    if (!value) throw new Error("球队资料包预检已失效，请重新选择文件预检");
    return { value, revision, current: () => this.packageSequence === revision && this.package === value };
  }
  async previewPackage(mode: SpreadsheetImportMode): Promise<boolean> {
    const revision = ++this.packageSequence, context = this.ports.capture(); this.packageValue = null; this.packageContext = context;
    const current = () => revision === this.packageSequence && context();
    try {
      const path = await this.ports.api.chooseExcelImportFile();
      if (!current() || !path) return false;
      const result = await this.ports.api.previewTeamPackageImport(path, mode);
      if (!current()) return false; this.packageValue = teamSnapshot(result); return true;
    } catch (error) { if (!current()) return false; throw error; }
  }
  previewMonthly(mode: SpreadsheetImportMode): Promise<boolean> { return this.monthlyBatch.load(mode); }
  private verifyBatch(expected: SpreadsheetImportPreview, actual: SpreadsheetImportPreview): void {
    if (expected.batch_id !== actual.batch_id || expected.source_sha256 !== actual.source_sha256 || expected.import_mode !== actual.import_mode) throw new Error("预检批次或文件摘要已变化，拒绝回填其他资料包结果");
  }
  async resolvePackage(scope: "team" | "player", resolution: SpreadsheetImportResolution): Promise<boolean> {
    const ticket = this.packageTicket(), part = scope === "team" ? ticket.value.team_preview : ticket.value.player_preview;
    if (!part) throw new Error("资料包没有对应的预检批次");
    if (this.packageBusy === ticket.revision) throw new Error("当前资料包正在处理，请等待完成");
    this.packageBusy = ticket.revision;
    try {
      const result = await this.ports.busy(() => scope === "team" ? this.ports.api.resolveTeamMonthlyImportConflict(part.batch_id, structuredClone(resolution)) : this.ports.api.resolvePlayerCatalogImportConflict(part.batch_id, structuredClone(resolution)));
      if (!ticket.current()) return false; this.verifyBatch(part, result);
      this.packageValue = teamSnapshot({ ...ticket.value, [scope === "team" ? "team_preview" : "player_preview"]: result }); return true;
    } catch (error) { if (!ticket.current()) return false; throw error; }
    finally { if (this.packageBusy === ticket.revision) this.packageBusy = null; }
  }
  resolveMonthly(resolution: SpreadsheetImportResolution): Promise<boolean> { return this.monthlyBatch.resolve(resolution); }
  async commitPackage() {
    const ticket = this.packageTicket();
    if (this.packageBusy === ticket.revision) throw new Error("当前资料包正在处理，请等待完成");
    const parts = [ticket.value.team_preview, ticket.value.player_preview].filter((part): part is SpreadsheetImportPreview => part !== null);
    if (!parts.length || ticket.value.coverage.blockers.length || parts.some(part => part.counts.conflict > 0 || part.counts.error > 0)) throw new Error("资料包仍有冲突或错误，请处理后再提交");
    this.packageBusy = ticket.revision;
    let commitReturned = false;
    try {
      const result = await this.ports.busy(() => this.ports.api.commitTeamPackageImport({ team_batch_id: ticket.value.team_preview?.batch_id ?? null, player_batch_id: ticket.value.player_preview?.batch_id ?? null }));
      commitReturned = true;
      if (!ticket.current()) return { result, current: false };
      for (const scope of ["team", "player"] as const) {
        const expected = ticket.value[scope === "team" ? "team_preview" : "player_preview"], actual = result[scope === "team" ? "team_result" : "player_result"];
        if ((expected?.batch_id ?? null) !== (actual?.batch_id ?? null)) throw new Error("资料包提交结果子批次不匹配，请核查原批次");
      }
      if (result.error_count === 0) this.packageValue = null;
      else {
        const [team, player] = await Promise.all([ticket.value.team_preview ? this.ports.api.readTeamMonthlyImportPreview(ticket.value.team_preview.batch_id) : null, ticket.value.player_preview ? this.ports.api.readPlayerCatalogImportPreview(ticket.value.player_preview.batch_id) : null]);
        if (!ticket.current()) return { result, current: false };
        if (team && ticket.value.team_preview) this.verifyBatch(ticket.value.team_preview, team);
        if (player && ticket.value.player_preview) this.verifyBatch(ticket.value.player_preview, player);
        this.packageValue = teamSnapshot({ ...ticket.value, team_preview: team, player_preview: player });
      }
      return { result, current: true };
    } catch (error) {
      if (!ticket.current()) return null;
      if (commitReturned) throw new Error(`提交已返回，但批次读回未能核实；请先核对原批次，勿直接重复提交：${error instanceof Error ? error.message : String(error)}`, { cause: error });
      throw error;
    }
    finally { if (this.packageBusy === ticket.revision) this.packageBusy = null; }
  }
  commitMonthly(ticket?: ImportTicket) { return this.monthlyBatch.commit(ticket); }
  async exportPackage() {
    const ticket = this.packageTicket(), preview = structuredClone(ticket.value);
    const base = preview.source_file_name.replace(/\.xlsx$/i, "").trim() || "球队完整资料包";
    const safe = base.replace(/[<>:\"/\\|?*\u0000-\u001f]+/g, "_");
    const path = await this.ports.api.chooseJsonExportFile(`${safe}-完整预检.json`);
    if (!path || !ticket.current()) return null;
    return this.ports.busy(() => this.ports.api.exportTeamPackagePreviewJson(path, preview));
  }
}
