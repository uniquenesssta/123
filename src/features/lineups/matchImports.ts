import type { SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution } from "../../types";
import type { LineupApi, LineupFiles } from "./lineupPorts";
interface ImportPorts {
    api: LineupApi;
    files: LineupFiles;
    capture(): () => boolean;
    busy<T>(operation: () => Promise<T>): Promise<T>;
}
/** One match/XLSX/AI-package staging owner, shared by Matches and Workbooks. */
export class MatchImports {
    private value: SpreadsheetImportPreview | null = null;
    private context = () => false;
    private sequence = 0;
    private processing: number | null = null;
    private uncertain = false;
    constructor(private readonly ports: ImportPorts) { }
    get preview(): SpreadsheetImportPreview | null { return this.context() ? this.value : null; }
    clear(): void { this.sequence++; this.value = null; this.uncertain = false; }
    private ticket() {
        const value = this.preview, sequence = this.sequence;
        if (!value)
            throw new Error("比赛预检已失效，请重新选择文件预检");
        return { value, sequence, current: () => sequence === this.sequence && this.preview === value };
    }
    private verify(expected: SpreadsheetImportPreview, actual: SpreadsheetImportPreview): void {
        if (actual.batch_id !== expected.batch_id || actual.source_sha256 !== expected.source_sha256 || actual.import_mode !== expected.import_mode)
            throw new Error("比赛批次、文件摘要或导入模式不匹配，请核查原批次");
    }
    async read(aiPackage: boolean, mode: SpreadsheetImportMode): Promise<boolean> {
        const sequence = ++this.sequence, context = this.ports.capture();
        this.context = context;
        this.value = null;
        this.uncertain = false;
        const current = () => context() && sequence === this.sequence;
        try {
            const path = aiPackage ? await this.ports.files.chooseZipImportFile() : await this.ports.files.chooseExcelImportFile();
            if (!current() || !path)
                return false;
            const preview = await this.ports.busy(() => aiPackage ? this.ports.api.previewAiMatchPackage(path, mode) : this.ports.api.previewMatchLineupImport(path, mode));
            if (!current())
                return false;
            if (preview.import_mode !== mode)
                throw new Error("比赛预检导入模式不匹配");
            this.value = structuredClone(preview);
            return true;
        }
        catch (error) {
            if (!current())
                return false;
            throw error;
        }
    }
    async resolve(resolution: SpreadsheetImportResolution): Promise<boolean> {
        const ticket = this.ticket();
        if (this.processing === ticket.sequence || this.uncertain)
            throw new Error("原比赛批次正在处理或结果未核实，请先读回账本");
        this.processing = ticket.sequence;
        try {
            const preview = await this.ports.busy(() => this.ports.api.resolveMatchLineupImportConflict(ticket.value.batch_id, structuredClone(resolution)));
            if (!ticket.current())
                return false;
            this.verify(ticket.value, preview);
            this.value = structuredClone(preview);
            return true;
        }
        catch (error) {
            if (!ticket.current())
                return false;
            throw error;
        }
        finally {
            if (this.processing === ticket.sequence)
                this.processing = null;
        }
    }
    async readback(): Promise<boolean> {
        const ticket = this.ticket();
        const preview = await this.ports.api.readMatchLineupImportPreview(ticket.value.batch_id);
        if (!ticket.current())
            return false;
        this.verify(ticket.value, preview);
        this.value = structuredClone(preview);
        return true;
    }
    async commit() {
        const ticket = this.ticket();
        if (this.processing === ticket.sequence)
            throw new Error("当前比赛批次正在处理");
        if (this.uncertain) {
            await this.readback();
            throw new Error("原比赛提交结果尚未核实，已尝试读回账本；请核查原批次或重新预检，禁止直接重复提交");
        }
        if (ticket.value.counts.conflict || ticket.value.counts.error)
            throw new Error("比赛预检仍有冲突或错误，请处理后再提交");
        if (ticket.value.counts.imported)
            throw new Error("此批次已有导入记录，请先读回账本并重新预检");
        this.processing = ticket.sequence;
        this.uncertain = true;
        try {
            const result = await this.ports.busy(() => this.ports.api.commitMatchLineupImport(ticket.value.batch_id));
            if (!ticket.current())
                return { result, current: false };
            if (result.batch_id !== ticket.value.batch_id)
                throw new Error("比赛提交批次不匹配");
            const preview = await this.ports.api.readMatchLineupImportPreview(ticket.value.batch_id);
            if (!ticket.current())
                return { result, current: false };
            this.verify(ticket.value, preview);
            this.value = structuredClone(preview);
            this.uncertain = false;
            return { result, current: true };
        }
        catch (error) {
            if (!ticket.current())
                return null;
            throw new Error("比赛提交结果或账本未核实，请查看原批次，勿直接重复提交：" + (error instanceof Error ? error.message : String(error)), { cause: error });
        }
        finally {
            if (this.processing === ticket.sequence)
                this.processing = null;
        }
    }
}
