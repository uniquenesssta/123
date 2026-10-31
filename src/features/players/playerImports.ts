import type { SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution } from "../../types";
import type { PlayerApi } from "./playerPorts";
import { playerSnapshot } from "./playerSnapshot";
interface ImportPorts { api: PlayerApi; capture(): () => boolean; busy<T>(operation: () => Promise<T>): Promise<T> }
/** Player workbook staging identity. Shared by Players and Workbooks; R13-09 reuses it. */
export class PlayerImports {
  private value: SpreadsheetImportPreview | null = null; private context = () => false; private sequence = 0; private processing: number | null = null;
  constructor(private readonly ports: ImportPorts) {}
  get preview(): SpreadsheetImportPreview | null { return this.context() ? this.value : null; }
  clear(): void { this.sequence++; this.value = null; }
  private ticket() { const value = this.preview, sequence = this.sequence; if (!value) throw new Error("球员预检已失效，请重新选择文件预检"); return { value, sequence, current: () => sequence === this.sequence && this.preview === value }; }
  private verify(expected: SpreadsheetImportPreview, actual: SpreadsheetImportPreview) { if (actual.batch_id !== expected.batch_id || actual.source_sha256 !== expected.source_sha256 || actual.import_mode !== expected.import_mode) throw new Error("球员批次、文件摘要或导入模式不匹配，请核查原批次"); }
  async read(mode: SpreadsheetImportMode): Promise<boolean> {
    const sequence = ++this.sequence, context = this.ports.capture(); this.context = context; this.value = null;
    const current = () => context() && sequence === this.sequence;
    try { const path = await this.ports.api.chooseExcelImportFile(); if (!current() || !path) return false; const preview = await this.ports.api.previewPlayerCatalogImport(path, mode); if (!current()) return false; this.value = playerSnapshot(preview); return true; }
    catch (error) { if (!current()) return false; throw error; }
  }
  async resolve(resolution: SpreadsheetImportResolution): Promise<boolean> {
    const ticket = this.ticket(); if (this.processing === ticket.sequence) throw new Error("当前球员批次正在处理"); this.processing = ticket.sequence;
    try { const preview = await this.ports.busy(() => this.ports.api.resolvePlayerCatalogImportConflict(ticket.value.batch_id, structuredClone(resolution))); if (!ticket.current()) return false; this.verify(ticket.value, preview); this.value = playerSnapshot(preview); return true; }
    catch (error) { if (!ticket.current()) return false; throw error; }
    finally { if (this.processing === ticket.sequence) this.processing = null; }
  }
  async commit() {
    const ticket = this.ticket(); if (this.processing === ticket.sequence) throw new Error("当前球员批次正在处理");
    if (ticket.value.counts.conflict || ticket.value.counts.error) throw new Error("球员预检仍有冲突或错误，请处理后再提交");
    this.processing = ticket.sequence; let returned = false;
    try {
      const result = await this.ports.busy(() => this.ports.api.commitPlayerCatalogImport(ticket.value.batch_id)); returned = true;
      if (!ticket.current()) return { result, current: false };
      if (result.batch_id !== ticket.value.batch_id) throw new Error("球员提交批次不匹配");
      const preview = await this.ports.api.readPlayerCatalogImportPreview(ticket.value.batch_id);
      if (!ticket.current()) return { result, current: false }; this.verify(ticket.value, preview); this.value = playerSnapshot(preview); return { result, current: true };
    } catch (error) { if (!ticket.current()) return null; if (returned) throw new Error(`提交已返回，但球员账本读回未核实，请先核对原批次，勿直接重复提交：${error instanceof Error ? error.message : String(error)}`, { cause: error }); throw error; }
    finally { if (this.processing === ticket.sequence) this.processing = null; }
  }
}
