import type { SpreadsheetImportCommitResult, SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution } from "../../types";
export type ImportPhase = "idle" | "choosing" | "previewing" | "ready" | "blocked" | "no_changes" | "writing" | "unknown" | "imported" | "error";
export interface ImportReceipt { batch_id: string; source_sha256: string; import_mode: SpreadsheetImportMode; source_file_name: string }
export interface ImportRecovery { read(): ImportReceipt | null; save(receipt: ImportReceipt): void; clear(): void }
export interface ImportTicket { readonly preview: SpreadsheetImportPreview; readonly revision: number; readonly active: () => boolean }
export interface SpreadsheetBatchPorts {
  label: string; capture(): () => boolean; captureView?(): () => boolean; changed?(): void; recovery?: ImportRecovery;
  busy<T>(operation: () => Promise<T>): Promise<T>;
  choose(): Promise<string | null>;
  preview(path: string, mode: SpreadsheetImportMode): Promise<SpreadsheetImportPreview>;
  resolve(id: string, resolution: SpreadsheetImportResolution): Promise<SpreadsheetImportPreview>;
  commit(id: string): Promise<SpreadsheetImportCommitResult>;
  read(id: string): Promise<SpreadsheetImportPreview>;
}
export function previewPhase(preview: SpreadsheetImportPreview | null): ImportPhase {
  if (!preview) return "idle";
  const c = preview.counts;
  if (c.imported > 0) return "imported";
  if (c.conflict > 0 || c.error > 0) return "blocked";
  return c.ready_add + c.ready_update + c.ready_end_previous > 0 ? "ready" : "no_changes";
}
/** Each domain owns one instance; no second client ledger or automatic write retry. */
export class SpreadsheetBatch {
  private value: SpreadsheetImportPreview | null = null;
  private context = () => false;
  private revision = 0;
  private working = false;
  private uncertain = false;
  private settled = false;
  private operation: ImportPhase | null = null;
  private modeValue: SpreadsheetImportMode = "add_and_update";
  private errorValue: string | null = null;
  constructor(private readonly ports: SpreadsheetBatchPorts) {}
  get preview(): SpreadsheetImportPreview | null { return this.context() ? this.value : null; }
  get mode(): SpreadsheetImportMode { return this.preview?.import_mode ?? this.receipt?.import_mode ?? this.modeValue; }
  get error(): string | null { return this.errorValue; }
  get busy(): boolean { return this.working; }
  get receipt(): ImportReceipt | null { return this.ports.recovery?.read() ?? null; }
  get phase(): ImportPhase { return this.operation ?? ((this.uncertain && this.context()) || this.receipt ? "unknown" : this.settled && this.preview ? "imported" : previewPhase(this.preview)); }
  get canCommit(): boolean { return !this.working && this.phase === "ready"; }
  clear(): void { this.revision++; this.value = null; this.operation = null; this.errorValue = null; this.settled = false; }
  setMode(mode: SpreadsheetImportMode): void { this.assertEditable(); this.modeValue = mode === "add_only" ? mode : "add_and_update"; this.clear(); }
  private assertEditable(): void { if (this.working || this.phase === "unknown") throw new Error("原批次正在处理或结果未核实，请先读回账本"); }
  private same(expected: ImportReceipt, actual: SpreadsheetImportPreview): void {
    if (expected.batch_id !== actual.batch_id || expected.source_sha256 !== actual.source_sha256 || expected.import_mode !== actual.import_mode)
      throw new Error(this.ports.label + "批次、文件摘要或导入模式不匹配，请核查原批次");
  }
  private publish(): void { this.ports.changed?.(); }
  confirmation(): ImportTicket {
    if (this.working) throw new Error("原批次正在处理，请等待完成");
    if (this.phase === "unknown") throw new Error("原批次结果未核实，禁止直接重复提交，请先读回账本");
    if (!this.canCommit || !this.preview) throw new Error(this.ports.label + "预检已失效、无变更或仍需核查，请重新预检或读回原批次");
    return { preview: this.preview, revision: this.revision, active: this.ports.captureView?.() ?? (() => true) };
  }
  async load(mode: SpreadsheetImportMode, choose = this.ports.choose, preview = this.ports.preview): Promise<boolean> {
    this.assertEditable();
    const revision = ++this.revision, context = this.ports.capture(), active = this.ports.captureView?.() ?? (() => true);
    this.context = context; this.modeValue = mode; this.value = null; this.errorValue = null; this.settled = false; this.working = true; this.operation = "choosing"; this.publish();
    const current = () => revision === this.revision && context() && active();
    try {
      const path = await choose();
      if (!current() || !path) return false;
      this.operation = "previewing"; this.publish();
      const result = await this.ports.busy(() => preview(path, mode));
      if (!current()) return false;
      if (result.import_mode !== mode) throw new Error("预检导入模式不匹配，请重新预检");
      this.value = structuredClone(result); return true;
    } catch (error) {
      if (!current()) return false;
      this.errorValue = error instanceof Error ? error.message : String(error); this.operation = "error"; throw error;
    } finally {
      this.working = false; if (this.operation !== "error") this.operation = null; this.publish();
    }
  }
  async resolve(resolution: SpreadsheetImportResolution): Promise<boolean> {
    this.assertEditable();
    const preview = this.preview, revision = this.revision, active = this.ports.captureView?.() ?? (() => true);
    if (!preview) throw new Error("预检已失效，请重新预检");
    this.working = true; this.publish();
    try {
      const actual = await this.ports.busy(() => this.ports.resolve(preview.batch_id, structuredClone(resolution)));
      if (revision !== this.revision || !this.context() || !active()) return false;
      this.same(preview, actual); this.value = structuredClone(actual); return true;
    } finally { this.working = false; this.publish(); }
  }
  async readback(): Promise<boolean> {
    if (this.working) throw new Error("原批次正在处理，请等待完成");
    const expected = this.receipt ?? this.preview;
    if (!expected) throw new Error("没有可核查的原批次");
    const context = this.ports.capture(), revision = this.revision;
    this.working = true; this.publish();
    try {
      const result = await this.ports.busy(() => this.ports.read(expected.batch_id));
      if (!context() || revision !== this.revision) return false;
      this.same(expected, result); this.context = context; this.value = structuredClone(result);
      if (result.counts.imported > 0 && result.counts.ready_add + result.counts.ready_update + result.counts.ready_end_previous === 0) {
        this.uncertain = false; this.ports.recovery?.clear();
      }
      // Wire previews have no transaction status; zero imported rows cannot prove rollback.
      this.errorValue = this.uncertain || this.receipt ? "已读回原批次；尚无最终写入证明，请勿重复提交。" : null;
      return true;
    } finally { this.working = false; this.publish(); }
  }
  async commit(ticket = this.confirmation()) {
    if (ticket.revision !== this.revision || this.preview !== ticket.preview || !ticket.active()) throw new Error("预检、文件、模式或页面上下文已变化，旧确认已失效");
    if (!this.canCommit) throw new Error("原批次正在处理或结果未核实，禁止重复提交");
    const expected = ticket.preview;
    this.ports.recovery?.save(expected);
    this.working = true; this.uncertain = true; this.operation = "writing"; this.errorValue = null; this.publish();
    let returned = false;
    try {
      const result = await this.ports.busy(() => this.ports.commit(expected.batch_id)); returned = true;
      if (result.batch_id !== expected.batch_id) throw new Error("提交结果批次不匹配");
      if (!this.context()) return {result,current:false};
      const actual = await this.ports.read(expected.batch_id); this.same(expected, actual);
      const current = ticket.revision === this.revision && this.context();
      if (current) this.value = structuredClone(actual);
      this.uncertain = false; this.settled = true; this.ports.recovery?.clear();
      return { result, current: current && ticket.active() };
    } catch (error) {
      this.errorValue = (returned ? "提交已返回，但账本读回未核实：" : "提交结果未核实：") + (error instanceof Error ? error.message : String(error));
      throw new Error(this.errorValue + "；请先读回原批次，勿直接重复提交", { cause: error });
    } finally { this.working = false; this.operation = null; this.publish(); }
  }
}
