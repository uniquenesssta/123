import type { MatchReviewDetail, MatchReviewPackageWorkflowAction, PostmatchSettlementRecord } from '../../types';
import { matchReviewWorkflowAllows, matchReviewWorkflowCompleted } from '../../app/matchReviewWorkflow';
import { readReviewLineups, validateReview, validateWorkflow, workflowVersion } from './reviewQueries';
import type { ReviewIntent, ReviewOperation, ReviewPorts, ReviewReceipt, ReviewState } from './reviewPorts';
const initial = (): ReviewState => ({ matches: [], reviews: [], choiceId: null, selectedMatchId: null, lineups: [], detail: null, workflow: null, settlement: null, loading: false, listLoading: false, loaded: false, error: null, notice: null });
const action: Partial<Record<ReviewOperation, MatchReviewPackageWorkflowAction>> = { export: 'export_package', preview: 'preview_import', confirm: 'confirm_import', facts: 'commit_facts', generate: 'generate_review' };
const timestamp = (value: unknown) => typeof value === 'string' && Number.isFinite(Date.parse(value));
/** Review owns its directory, selected object and readbacks. Backend alone grants workflow capability. */
export class ReviewFeature {
    private data = initial();
    private dataScope = () => false;
    private selectionRevision = 0;
    private listRevision = 0;
    private detailRevision = 0;
    private viewRevision = 0;
    private confirmationRevision = 0;
    private locks = new Map<symbol, {
        id: string;
        scope: () => boolean;
    }>();
    private consumed = new WeakSet<ReviewIntent>();
    constructor(private readonly ports: ReviewPorts) { }
    get state(): Readonly<ReviewState> { return this.dataScope() ? this.data : initial(); }
    get preview() { return this.state.workflow?.preview ?? null; }
    get pending() { return this.state.selectedMatchId ? this.ports.recovery.read(this.state.selectedMatchId) : null; }
    get busy() { const id = this.state.selectedMatchId; return [...this.locks.values()].some(lock => lock.id === id && lock.scope()); }
    get selected() { return this.state.matches.find(row => row.match_record.id === this.state.selectedMatchId) ?? null; }
    get version() { return JSON.stringify([this.state.selectedMatchId, workflowVersion(this.state.workflow), this.state.detail?.summary.id ?? null]); }
    capture(): () => boolean {
        const selection = this.selectionRevision, view = this.viewRevision, scope = this.ports.scope(), active = this.ports.view();
        return () => selection === this.selectionRevision && view === this.viewRevision && scope() && active();
    }
    viewChanged(): void { this.confirmationRevision++; }
    captureConfirmation(): () => boolean { const revision = this.confirmationRevision, current = this.capture(); return () => revision === this.confirmationRevision && current(); }
    navigationChanged(): void { this.viewChanged(); this.viewRevision++; this.detailRevision++; this.listRevision++; this.data.loading = false; this.data.listLoading = false; }
    private changed() { this.ports.changed(); }
    private enterScope(): void { if (!this.dataScope()) {
        this.data = initial();
        this.selectionRevision++;
    } this.dataScope = this.ports.scope(); }
    async load(): Promise<boolean> {
        this.enterScope();
        const serial = ++this.listRevision, selection = this.selectionRevision, selected = this.data.selectedMatchId, current = this.capture();
        this.data.listLoading = true;
        this.data.error = null;
        this.changed();
        try {
            const [matches, reviews] = await Promise.all([this.ports.api.listReviewableMatches(100), this.ports.api.listMatchReviews(100)]);
            if (!current() || serial !== this.listRevision)
                return false;
            this.data.matches = matches;
            this.data.reviews = reviews;
            this.data.loaded = true;
            if (selected && selection === this.selectionRevision) {
                if(!await this.select(selected, this.data.detail?.summary.id))return false;
                if (this.pending)
                    await this.recover();
            }
            return true;
        }
        catch (error) {
            if (current())
                this.data.error = '刷新未完成，保留已载入记录。 ' + this.ports.formatError(error);
            return false;
        }
        finally {
            if (serial === this.listRevision) {
                this.data.listLoading = false;
                this.changed();
            }
        }
    }
    setSelection(id: string | null): void {
        this.enterScope();
        this.selectionRevision++;
        this.detailRevision++;
        Object.assign(this.data, { choiceId: id, selectedMatchId: id, lineups: [], detail: null, workflow: null, settlement: null, error: null, notice: null, loading: false });
    }
    choose(id: string | null): void { this.setSelection(null); this.data.choiceId = id; this.changed(); }
    async select(id: string, reviewId?: string): Promise<boolean> {
        if (!id)
            throw Error('请先选择需要复盘的比赛');
        if (!reviewId && !this.state.matches.some(row => row.match_record.id === id))
            throw Error('该比赛不在当前可复盘目录中');
        this.setSelection(id);
        const serial = ++this.detailRevision, current = this.capture();
        this.data.loading = true;
        this.changed();
        try {
            const [lineups, workflow, settlements] = await Promise.all([readReviewLineups(this.ports.api, id), this.ports.api.readMatchReviewPackageWorkflow(id), this.ports.api.listPostmatchSettlements(100)]);
            if (!current() || serial !== this.detailRevision)
                return false;
            validateWorkflow(workflow, id);
            const actualReviewId = reviewId ?? workflow?.review_id ?? this.selected?.latest_review?.id;
            const detail = actualReviewId ? await this.ports.api.readMatchReview(actualReviewId) : null;
            if (!current() || serial !== this.detailRevision)
                return false;
            if (detail)
                validateReview(detail, actualReviewId!, id);
            Object.assign(this.data, { lineups, workflow, detail, settlement: settlements.find(item => item.match_id === id) ?? null });
            return true;
        }
        catch (error) {
            if (current())
                this.data.error = '本场复盘资料未载入，不复用其他比赛内容。 ' + this.ports.formatError(error);
            return false;
        }
        finally {
            if (serial === this.detailRevision) {
                this.data.loading = false;
                this.changed();
            }
        }
    }
    async openReview(id: string): Promise<boolean> {
        const summary = this.state.reviews.find(row => row.id === id);
        if (!summary)
            throw Error('复盘不属于当前已载入记录');
        return this.select(summary.match_id, id);
    }
    async refreshLineups(id: string): Promise<void> {
        const current = this.capture();
        if (id !== this.state.selectedMatchId)
            return;
        const records = await readReviewLineups(this.ports.api, id);
        if (current())
            this.data.lineups = records;
    }
    allows(kind: ReviewOperation): boolean {
        if (!this.selected || this.state.loading || this.busy || this.pending)
            return false;
        if (kind === 'manual' && matchReviewWorkflowCompleted(this.state.workflow, 'confirm_import') && !matchReviewWorkflowCompleted(this.state.workflow, 'generate_review'))
            return false;
        const capability = action[kind];
        return capability ? kind === 'export' && !this.state.workflow || matchReviewWorkflowAllows(this.state.workflow, capability) : true;
    }
    prepare(kind: ReviewIntent['kind'], options: Partial<Pick<ReviewIntent, 'confirmedBy' | 'note' | 'draft' | 'candidateId' | 'decision'>> = {}): ReviewIntent {
        if (!this.allows(kind))
            throw Error('当前后端许可或请求状态不允许此操作，请先刷新原工作流');
        if (kind === 'confirm' && (!this.preview?.ready || this.preview.errors.length || !this.preview.source_sha256))
            throw Error('本轮预检尚未通过');
        if (kind === 'manual' && (!options.draft || options.draft.match_id !== this.state.selectedMatchId))
            throw Error('人工事实与当前比赛不一致');
        if (kind === 'candidate' && !this.state.detail?.ability_candidates.some(row => row.id === options.candidateId && row.status === 'pending'))
            throw Error('该候选不属于当前待审核复盘');
        const draft = options.draft ? structuredClone(options.draft) : null;
        if (draft && !draft.review_version)
            draft.review_version = 'manual-' + crypto.randomUUID();
        return { kind, matchId: this.state.selectedMatchId!, version: this.version, confirmedBy: options.confirmedBy ?? null, note: options.note ?? null, draft, candidateId: options.candidateId ?? null, decision: options.decision ?? null };
    }
    private ensureIdle(id: string): void {
        if (this.ports.recovery.read(id) || [...this.locks.values()].some(lock => lock.id === id && lock.scope()))
            throw Error('原复盘请求仍待核实，禁止重复提交');
    }
    private receipt(kind: ReviewOperation, path: string | null = null): ReviewReceipt {
        const w = this.state.workflow;
        return { matchId: this.state.selectedMatchId!, kind, packageId: w?.package_id ?? null, sha: kind === 'preview' ? null : w?.import_sha256 ?? w?.export_sha256 ?? null, path, reviewId: null, reviewVersion: null, candidateId: null, decision: null, startedAt: new Date().toISOString(), baseline: workflowVersion(w) };
    }
    private async fresh(kind: ReviewOperation, current: () => boolean, expectedVersion: string): Promise<boolean> {
        const id = this.state.selectedMatchId!;
        const workflow = await this.ports.api.readMatchReviewPackageWorkflow(id);
        if (!current())
            return false;
        validateWorkflow(workflow, id);
        if (JSON.stringify([id, workflowVersion(workflow), this.state.detail?.summary.id ?? null]) !== expectedVersion)
            throw Error('资料包、SHA或工作流版本已变化，请刷新并重新确认');
        if (action[kind] && !(kind === 'export' && !workflow) && !matchReviewWorkflowAllows(workflow, action[kind]!))
            throw Error('后端当前未允许此步骤');
        return true;
    }
    async file(kind: 'export' | 'preview'): Promise<boolean> {
        if (!this.allows(kind))
            throw Error('当前步骤不允许选择文件或导出');
        const id = this.state.selectedMatchId!, current = this.capture(), scope = this.ports.scope(), version = this.version;
        this.ensureIdle(id);
        const token = Symbol(id);
        this.locks.set(token, { id, scope });
        this.changed();
        let sent = false;
        try {
            const match = this.selected!.match_record;
            const path = kind === 'export' ? await this.ports.api.chooseExcelExportFile(`赛后复盘资料包_${match.home_team_name}_${match.away_team_name}.xlsx`.replace(/[\\/:*?"<>|]/g, '-')) : await this.ports.api.chooseExcelImportFile();
            if (!current() || !path)
                return false;
            if (!await this.fresh(kind, current, version))
                return false;
            const receipt = this.receipt(kind, path);
            this.ports.recovery.save(receipt);
            sent = true;
            if (kind === 'export') {
                const result = await this.ports.api.exportMatchReviewPackage(path, id);
                if (!scope())
                    return false;
                if (result.match_id !== id || !result.package_id || !result.content_sha256)
                    throw Error('导出回执缺少原比赛或包指纹');
                Object.assign(receipt, { packageId: result.package_id, sha: result.content_sha256 });
                this.ports.recovery.save(receipt);
            }
            else {
                const preview = await this.ports.api.previewMatchReviewPackage(path, id);
                if (!scope())
                    return false;
                if (preview.match_id !== id || preview.package_id !== receipt.packageId)
                    throw Error('预检回执与原比赛或资料包不一致');
                receipt.sha = preview.source_sha256;
                this.ports.recovery.save(receipt);
            }
            return await this.readBack(receipt, scope, current);
        }
        catch (error) {
            if (current())
                this.data.error = (sent ? '本次文件操作结果待核实，刷新只查回原工作流。 ' : '') + this.ports.formatError(error);
            throw error;
        }
        finally {
            this.locks.delete(token);
            this.changed();
        }
    }
    async submit(intent: ReviewIntent, confirmed: () => boolean): Promise<boolean> {
        const id = intent.matchId;
        if (this.consumed.has(intent) || id !== this.state.selectedMatchId || intent.version !== this.version || !confirmed())
            throw Error('原确认已失效或已提交，请重新核对本场事实或包身份');
        this.ensureIdle(id);
        const scope = this.ports.scope(), captured = this.capture(), current = () => captured() && confirmed();
        const token = Symbol(id);
        this.locks.set(token, { id, scope });
        this.changed();
        let sent = false;
        try {
            if (!await this.fresh(intent.kind, current, intent.version))
                return false;
            if (intent.kind === 'manual') {
                const lineups = await readReviewLineups(this.ports.api, id);
                if (!current())
                    return false;
                if (JSON.stringify(lineups) !== JSON.stringify(this.state.lineups))
                    throw Error('本场阵容已变化，请重新载入并核对事实草稿');
                const rows = await this.ports.api.listMatchReviews(100);
                if (!current())
                    return false;
                if (rows.some(row => row.match_id === id && row.review_version === intent.draft?.review_version))
                    throw Error('该人工复盘版本已有记录，请先查看原结果');
            }
            if (intent.kind === 'candidate') {
                const detail = await this.ports.api.readMatchReview(this.state.detail!.summary.id);
                if (!current())
                    return false;
                validateReview(detail, this.state.detail!.summary.id, id);
                if (!detail.ability_candidates.some(row => row.id === intent.candidateId && row.status === 'pending'))
                    throw Error('原候选已处理，请重新载入结果');
            }
            const receipt = this.receipt(intent.kind);
            receipt.reviewVersion = intent.draft?.review_version ?? null;
            receipt.candidateId = intent.candidateId;
            receipt.decision = intent.decision;
            receipt.reviewId = intent.kind === 'candidate' ? this.state.detail?.summary.id ?? null : null;
            this.ports.recovery.save(receipt);
            sent = true;
            this.consumed.add(intent);
            switch (intent.kind) {
                case 'confirm':
                    await this.ports.api.confirmMatchReviewPackage({ package_id: receipt.packageId!, confirmed_by: intent.confirmedBy, confirmation_note: intent.note });
                    break;
                case 'facts':
                    await this.ports.api.commitMatchReviewPackageFacts(receipt.packageId!);
                    break;
                case 'generate': {
                    const result = await this.ports.api.generateMatchReviewFromPackage(receipt.packageId!);
                    if (!scope())
                        return false;
                    validateReview(result.review, result.review.summary.id, id);
                    receipt.reviewId = result.review.summary.id;
                    this.ports.recovery.save(receipt);
                    break;
                }
                case 'manual': {
                    const result = await this.ports.api.generateMatchReview(intent.draft!);
                    if (!scope())
                        return false;
                    validateReview(result, result.summary.id, id);
                    receipt.reviewId = result.summary.id;
                    this.ports.recovery.save(receipt);
                    break;
                }
                case 'candidate':
                    await this.ports.api.decideAbilityCandidate({ candidate_id: intent.candidateId!, decision: intent.decision!, decided_by: intent.confirmedBy ?? 'local_user', decision_note: intent.note });
                    break;
            }
            if (!scope())
                return false;
            return await this.readBack(receipt, scope, captured);
        }
        catch (error) {
            // This exact backend validation is emitted before confirm_workflow; it writes no confirmation/facts.
            if (sent && intent.kind === 'confirm' && String(error).includes('资料包在预检后已变化或重新校验失败，请重新预检') && scope())
                this.ports.recovery.clear(id);
            if (current())
                this.data.error = (this.ports.recovery.read(id) ? '本步骤结果待核实，保留原包信息，不会重复事实或自动重发。 ' : '') + this.ports.formatError(error);
            throw error;
        }
        finally {
            this.locks.delete(token);
            this.changed();
        }
    }
    async recover(): Promise<boolean> {
        const id = this.state.selectedMatchId, receipt = id ? this.ports.recovery.read(id) : null;
        if (!id || !receipt || this.busy)
            return false;
        const scope = this.ports.scope(), current = this.capture(), token = Symbol(id);
        this.locks.set(token, { id, scope });
        this.changed();
        try {
            return await this.readBack(receipt, scope, current);
        }
        catch (error) {
            if (current())
                this.data.error = '原步骤仍待核实。 ' + this.ports.formatError(error);
            return false;
        }
        finally {
            this.locks.delete(token);
            this.changed();
        }
    }
    private async readBack(receipt: ReviewReceipt, scope: () => boolean, current: () => boolean): Promise<boolean> {
        const id = receipt.matchId;
        if (receipt.kind === 'manual' || receipt.kind === 'candidate') {
            let reviewId = receipt.reviewId;
            if (!reviewId && receipt.reviewVersion) {
                const rows = await this.ports.api.listMatchReviews(100);
                if (!scope())
                    return false;
                const matches = rows.filter(r => r.match_id === id && r.review_version === receipt.reviewVersion);
                if (matches.length === 1)
                    reviewId = matches[0].id;
            }
            if (!reviewId)
                throw Error('原复盘ID尚未核实，不能把其他结果当作本次成功');
            const detail = await this.ports.api.readMatchReview(reviewId);
            if (!scope())
                return false;
            validateReview(detail, reviewId, id);
            if (receipt.kind === 'manual' && detail.summary.review_version !== receipt.reviewVersion)
                throw Error('人工复盘版本与原请求不一致');
            if (receipt.kind === 'candidate' && detail.ability_candidates.find(row => row.id === receipt.candidateId)?.status !== (receipt.decision === 'accept' ? 'accepted' : 'rejected'))
                throw Error('原候选决定尚未核实');
            if (current()) {
                this.data.detail = detail;
                this.data.notice = receipt.kind === 'manual' ? '人工复盘已读回；不表示资料包事实提交、结算或能力候选已应用。' : '原候选处理状态已读回。';
            }
        }
        else {
            const workflow = await this.ports.api.readMatchReviewPackageWorkflow(id);
            if (!scope())
                return false;
            validateWorkflow(workflow, id);
            if (!workflow || !receipt.packageId || workflow.package_id !== receipt.packageId)
                throw Error('原资料包ID尚未核实或已被新一轮替代；不自动接管其他包');
            const hash = receipt.kind === 'export' ? workflow.export_sha256 : workflow.import_sha256;
            if (receipt.sha && hash !== receipt.sha)
                throw Error('工作流文件摘要与原请求不一致');
            let proven = false;
            switch (receipt.kind) {
                case 'export':
                    proven = workflow.export_path === receipt.path && !!workflow.export_sha256 && timestamp(workflow.exported_at);
                    break;
                case 'preview':
                    proven = workflow.import_path === receipt.path && !!workflow.preview && workflowVersion(workflow) !== receipt.baseline;
                    break;
                case 'confirm':
                    proven = timestamp(workflow.confirmed_at);
                    break;
                case 'facts':
                    proven = timestamp(workflow.facts_committed_at);
                    break;
                case 'generate':
                    proven = !!workflow.review_id && timestamp(workflow.review_created_at);
                    break;
            }
            if (receipt.kind === 'generate' && receipt.reviewId && workflow.review_id !== receipt.reviewId)
                throw Error('生成回执与工作流复盘ID不一致');
            if (receipt.kind === 'preview' && !receipt.sha)
                throw Error('原预检响应未返回文件摘要，请核对当前登记文件后明确采用');
            if (!proven)
                throw Error('原步骤完成状态尚未核实，保留恢复信息，不自动重试');
            const lineups = receipt.kind === 'facts' ? await readReviewLineups(this.ports.api, id) : null;
            if (!scope())
                return false;
            const detail = receipt.kind === 'generate' ? await this.ports.api.readMatchReview(workflow.review_id!) : null;
            if (!scope())
                return false;
            if (detail)
                validateReview(detail, workflow.review_id!, id);
            if (current()) {
                this.data.workflow = workflow;
                if (lineups)
                    this.data.lineups = lineups;
                if (detail)
                    this.data.detail = detail;
                this.data.notice = ({ export: '资料包导出已核实，后续填写、预检与事实提交尚未执行。', preview: '预检结果已读回，是否可确认仍以后端许可与阻断项为准。', confirm: '本轮人工确认已核实，事实尚需单独提交。', facts: '本轮事实已写入；正式复盘生成、结算与候选应用需分别执行。', generate: '正式复盘已读回；不表示已正式结算或已应用能力候选。' } as Record<string, string>)[receipt.kind];
            }
        }
        if (!scope())
            return false;
        this.ports.recovery.clear(id);
        if (current())
            this.data.error = null;
        return true;
    }
    async adoptCurrentFile(version: string, confirmed: () => boolean): Promise<void> {
        const receipt = this.pending, id = this.state.selectedMatchId;
        if (!receipt || !id || !['export', 'preview'].includes(receipt.kind) || this.busy || !confirmed() || version !== this.version)
            throw Error('当前文件核对已失效');
        const current = this.capture(), workflow = await this.ports.api.readMatchReviewPackageWorkflow(id);
        if (!current() || !confirmed() || version !== this.version)
            return;
        validateWorkflow(workflow, id);
        if (!workflow || workflowVersion(workflow) !== workflowVersion(this.state.workflow) || !(workflow.import_sha256 ?? workflow.export_sha256))
            throw Error('当前登记文件已变化，请刷新后重新核对');
        const pending = this.pending;
        if (!pending || JSON.stringify(pending) !== JSON.stringify(receipt))
            throw Error('原文件请求已变化');
        this.ports.recovery.clear(id);
        this.data.workflow = workflow;
        this.data.error = null;
        this.data.notice = '已明确采用当前数据库登记的文件状态；未将原文件请求认定为成功，也未重发或写入事实。';
        this.changed();
    }
    adoptSettlement(settlement: PostmatchSettlementRecord): void { if (settlement.match_id === this.state.selectedMatchId)
        this.data.settlement = settlement; }
    adoptDetail(detail: MatchReviewDetail): void { if (detail.summary.match_id === this.state.selectedMatchId)
        this.data.detail = detail; }
}
