import type { BootstrapResponse, ModelRunListItem } from "../../types";
import type { HideReceipt, RunRecovery } from "./runRecovery";
export interface RunsPorts {
    api: {
        readRun(id: string): Promise<Record<string, unknown>>;
        hideModelRunHistory(id: string, reason: string): Promise<void>;
    };
    source: {
        snapshot(): BootstrapResponse | null;
        refresh(): Promise<boolean>;
        capture(): () => boolean;
        hidden(id: string): void;
    };
    view(): () => boolean;
    changed(): void;
    recovery: RunRecovery;
    formatError?(error: unknown): string;
}
export interface HideIntent {
    runId: string;
    label: string;
    rowVersion: string | null;
}
const object = (v: unknown): Record<string, unknown> => v && typeof v === "object" && !Array.isArray(v) ? v as Record<string, unknown> : {};
export function runDocumentId(doc: Record<string, unknown>): string { return typeof doc.id === "string" ? doc.id : typeof doc.run_id === "string" ? doc.run_id : ""; }
export function runLineage(doc: Record<string, unknown>): string {
    const audit = object(doc.input_audit);
    return JSON.stringify([runDocumentId(doc), doc.match_key, doc.snapshot_type, doc.model_key, doc.model_version, doc.parameter_version, doc.rule_package_id, doc.input_sha256, audit.feature_snapshot_id, audit.manifest_sha256 ?? audit.input_manifest_sha256]);
}
function hidden(doc: Record<string, unknown>): boolean { return typeof doc.history_hidden_at === "string" && Number.isFinite(Date.parse(doc.history_hidden_at)); }
function rowVersion(row: ModelRunListItem): string { return JSON.stringify([row.id, row.match_key, row.model_key, row.model_version, row.parameter_version, row.snapshot_type, row.input_manifest_sha256, row.created_at]); }
/** Session remains sole list-cache owner. This feature owns selection, request states and visibility actions. */
export class RunsFeature {
    private viewRevision = 0;
    private listRevision = 0;
    private detailRevision = 0;
    private listScope = () => true;
    private detailScope = () => true;
    private listLoaded = false;
    private listLoading = false;
    private listError: string | null = null;
    private detailError: string | null = null;
    private detailValue: Record<string, unknown> | null = null;
    private selectedValue: string | null = null;
    private detailLoading = false;
    private notice: string | null = null;
    private locked = new Map<symbol, {
        id: string;
        scope: () => boolean;
    }>();
    constructor(private readonly ports: RunsPorts) { }
    get rows(): readonly ModelRunListItem[] { return this.ports.source.snapshot()?.data.recent_runs ?? []; }
    get selectedId(): string | null { return this.detailScope() ? this.selectedValue : null; }
    get detail(): Record<string, unknown> | null { return this.selectedId ? this.detailValue : null; }
    get view() { const valid = this.listScope(); return { rows: this.rows, selectedId: this.selectedId, detail: this.detail, detailLoading: this.detailLoading && this.detailScope(), detailError: this.detailScope() ? this.detailError : null, listLoading: this.listLoading && valid, firstLoad: (!this.listLoaded || !valid) && !this.rows.length, listError: valid ? this.listError : null, notice: valid ? this.notice : null, pending: this.ports.recovery.all(), locked: [...this.locked.values()].filter(lock => lock.scope()).map(lock => lock.id) }; }
    captureView(): () => boolean { const revision = this.viewRevision, view = this.ports.view(), scope = this.ports.source.capture(); return () => revision === this.viewRevision && view() && scope(); }
    navigationChanged(): void { this.viewRevision++; this.listRevision++; this.detailRevision++; this.listLoading = false; this.selectedValue = null; this.detailValue = null; this.detailLoading = false; this.detailError = null; }
    back(): void { this.viewRevision++; this.detailRevision++; this.selectedValue = null; this.detailValue = null; this.detailLoading = false; this.detailError = null; this.ports.changed(); }
    async refresh(): Promise<boolean> {
        if (this.listLoading && this.listScope())
            return false;
        const serial = ++this.listRevision, current = this.captureView(), scope = this.ports.source.capture();
        this.listScope = scope;
        this.listLoading = true;
        this.listError = null;
        this.ports.changed();
        try {
            const applied = await this.ports.source.refresh();
            if (!scope() || !current() || serial !== this.listRevision)
                return false;
            if (!applied)
                return false;
            this.listLoaded = true;
            for (const receipt of this.ports.recovery.all()) {
                if (!scope() || !current() || serial !== this.listRevision)
                    return false;
                await this.recover(receipt.runId);
            }
            return true;
        }
        catch (error) {
            if (current() && scope() && serial === this.listRevision)
                this.listError = "刷新未完成，保留原记录。 " + (this.ports.formatError?.(error) ?? "请求未完成，请核对原记录");
            return false;
        }
        finally {
            if (serial === this.listRevision) {
                this.listLoading = false;
                this.ports.changed();
            }
        }
    }
    async open(id: string): Promise<boolean> {
        if (!id)
            throw Error("记录缺少运行ID，无法读取详情");
        this.viewRevision++;
        const serial = ++this.detailRevision, current = this.captureView(), scope = this.ports.source.capture();
        this.detailScope = scope;
        this.selectedValue = id;
        this.detailValue = null;
        this.detailLoading = true;
        this.detailError = null;
        this.ports.changed();
        try {
            const doc = await this.ports.api.readRun(id);
            if (!current() || !scope() || serial !== this.detailRevision)
                return false;
            this.validate(doc, id);
            this.detailValue = doc;
            return true;
        }
        catch (error) {
            if (current() && scope() && serial === this.detailRevision)
                this.detailError = "读取本条记录失败，请重新载入。 " + (this.ports.formatError?.(error) ?? "请求未完成，请核对原记录");
            return false;
        }
        finally {
            if (serial === this.detailRevision) {
                this.detailLoading = false;
                this.ports.changed();
            }
        }
    }
    prepareHide(id: string): HideIntent {
        if (!id || this.isLocked(id) || this.ports.recovery.read(id))
            throw Error("该记录正在处理或结果待核实，请先刷新记录查回原请求");
        const row = this.rows.find(r => r.id === id), doc = this.detail;
        if (!row && (!doc || runDocumentId(doc) !== id))
            throw Error("该记录不属于当前目录或已打开的详情");
        if (doc && runDocumentId(doc) === id && hidden(doc))
            throw Error("该记录已从历史列表移除，底层记录仍可追溯");
        const label = row?.home_team_name && row.away_team_name ? `${row.home_team_name} vs ${row.away_team_name}` : row?.match_key || String(doc?.match_key || "未命名比赛");
        return { runId: id, label, rowVersion: row ? rowVersion(row) : null };
    }
    async hide(intent: HideIntent, confirmationCurrent: () => boolean): Promise<boolean> {
        const id = intent.runId;
        if (this.isLocked(id) || this.ports.recovery.read(id))
            throw Error("该记录结果仍待核实，禁止重复提交");
        const row = this.rows.find(r => r.id === id);
        if (!confirmationCurrent() || (intent.rowVersion !== null && (!row || rowVersion(row) !== intent.rowVersion)))
            throw Error("原确认已失效，请重新核对记录");
        const current = this.captureView(), scope = this.ports.source.capture();
        const token = Symbol(id);
        this.locked.set(token, { id, scope });
        this.listScope = scope;
        this.listError = null;
        this.ports.changed();
        let dispatched = false;
        try {
            const original = await this.ports.api.readRun(id);
            if (!current() || !scope() || !confirmationCurrent())
                return false;
            this.validate(original, id);
            if (row) {
                for (const key of ["match_key", "model_key", "model_version", "parameter_version", "snapshot_type"] as const) {
                    if (row[key] && original[key] !== row[key])
                        throw Error("原运行身份已变化，请刷新后重新确认");
                }
                const audit = object(original.input_audit), manifest = audit.manifest_sha256 ?? audit.input_manifest_sha256;
                if (row.input_manifest_sha256 && manifest !== row.input_manifest_sha256)
                    throw Error("原运行输入指纹已变化，请重新确认");
            }
            const receipt: HideReceipt = { runId: id, label: intent.label, lineage: runLineage(original) };
            if (hidden(original)) {
                this.adoptHidden(original, receipt, scope, current);
                return true;
            }
            this.ports.recovery.save(receipt);
            dispatched = true;
            await this.ports.api.hideModelRunHistory(id, "用户从历史列表移除");
            if (!scope())
                return false;
            const actual = await this.ports.api.readRun(id);
            if (!scope())
                return false;
            this.adoptHidden(actual, receipt, scope, current);
            return true;
        }
        catch (error) {
            if (!scope() || !current())
                return false;
            this.listError = dispatched ? "移除结果尚未核实；刷新记录只查回原对象，不自动重发。 " + (this.ports.formatError?.(error) ?? "请求未完成，请核对原记录") : (this.ports.formatError?.(error) ?? "请求未完成，请核对原记录");
            throw error;
        }
        finally {
            this.locked.delete(token);
            this.ports.changed();
        }
    }
    async recover(id: string): Promise<boolean> {
        const receipt = this.ports.recovery.read(id);
        if (!receipt || this.isLocked(id))
            return false;
        const scope = this.ports.source.capture(), current = this.captureView();
        const token = Symbol(id);
        this.locked.set(token, { id, scope });
        this.ports.changed();
        try {
            const doc = await this.ports.api.readRun(id);
            if (!scope())
                return false;
            this.validate(doc, id);
            if (!hidden(doc)) {
                if (current())
                    this.notice = "尚未核实原记录已隐藏；当前可见或未返回状态不等于原请求未执行，保留恢复信息。";
                return false;
            }
            this.adoptHidden(doc, receipt, scope, current);
            return true;
        }
        catch (error) {
            if (current() && scope())
                this.listError = "查回原记录失败，移除请求仍待核实。 " + (this.ports.formatError?.(error) ?? "请求未完成，请核对原记录");
            return false;
        }
        finally {
            this.locked.delete(token);
            this.ports.changed();
        }
    }
    private isLocked(id: string): boolean { return [...this.locked.values()].some(lock => lock.id === id && lock.scope()); }
    private validate(doc: Record<string, unknown>, id: string): void { if (!doc || runDocumentId(doc) !== id)
        throw Error("返回的运行ID与原记录不一致"); }
    private adoptHidden(doc: Record<string, unknown>, receipt: HideReceipt, scope: () => boolean, current: () => boolean): void {
        this.validate(doc, receipt.runId);
        if (!hidden(doc) || runLineage(doc) !== receipt.lineage)
            throw Error("可见性或原运行血缘尚未核实，保留原请求信息");
        if (!scope())
            return;
        this.ports.source.hidden(receipt.runId);
        this.ports.recovery.clear(receipt.runId);
        if (current()) {
            if (this.selectedId === receipt.runId)
                this.detailValue = doc;
            this.notice = "记录已从列表移除；原运行、矩阵、输入快照及复盘血缘仍可追溯。";
            this.listError = null;
        }
    }
}
