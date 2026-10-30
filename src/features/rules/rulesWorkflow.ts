import type { BootstrapResponse, CompetitionBindingDraft, RouteDecision, RoutePreviewCommand, RulePackageDraft } from "../../types";
import { assertHierarchy } from "../competitions/index";
import { RulePackageImport } from "./rulePackageImport";

export interface RulesWorkflowPorts {
  snapshot(): BootstrapResponse | null;
  registerRulePackage(draft: RulePackageDraft): Promise<unknown>;
  createBinding(draft: CompetitionBindingDraft): Promise<unknown>;
  previewRoute(command: RoutePreviewCommand): Promise<RouteDecision>;
  refresh(): Promise<boolean>;
}

/** Registration/binding validity and formal model eligibility always remain behind the real API. */
export class RulesWorkflow {
  readonly packages = new RulePackageImport();
  private revision = 0;
  private writing = false;
  private closed = false;
  private preview: RouteDecision | null = null;
  constructor(private readonly ports: RulesWorkflowPorts) {}
  get route(): RouteDecision | null { return this.preview; }
  invalidatePreview(): void { this.revision++; this.preview = null; }
  private snapshot(): BootstrapResponse {
    const state = this.ports.snapshot();
    if (this.closed || !state?.data.database_configured) throw new Error("请先连接数据库");
    return state;
  }
  private async write(operation: () => Promise<unknown>): Promise<void> {
    if (this.writing) throw new Error("规则保存正在进行，请等待完成");
    const state = this.snapshot(); this.writing = true; this.invalidatePreview();
    try {
      await operation();
      if (this.closed || state !== this.ports.snapshot()) throw new Error("提交期间数据上下文已变化，请刷新核对结果后继续");
      if (!await this.ports.refresh()) throw new Error("规则已保存，但目录刷新未完成；请刷新核对，勿重复提交");
    } finally { this.writing = false; }
  }
  async register(): Promise<void> {
    const ticket = this.packages.registration();
    await this.write(() => this.ports.registerRulePackage(ticket.draft));
    this.packages.registered(ticket.revision);
  }
  async saveBinding(draft: CompetitionBindingDraft): Promise<void> {
    const input = structuredClone(draft), state = this.snapshot();
    assertHierarchy(state, input);
    if (!input.competition_id || !state.data.rule_packages.some(item => item.id === input.rule_package_id && item.status === "active" && (item.model_id === "p4" || item.model_id.startsWith("p4_")))) throw new Error("请选择当前有效赛事和 P4 规则包");
    await this.write(() => this.ports.createBinding(input));
  }
  async resolve(command: RoutePreviewCommand, isCurrent: () => boolean): Promise<RouteDecision | null> {
    if (this.writing) throw new Error("规则保存正在进行，请完成后重新预览");
    const input = structuredClone(command), state = this.snapshot();
    assertHierarchy(state, input);
    this.invalidatePreview(); const ticket = this.revision;
    const current = () => !this.closed && ticket === this.revision && state === this.ports.snapshot() && isCurrent();
    try {
      const result = await this.ports.previewRoute(input);
      if (!current()) return null;
      this.preview = result;
      return result;
    } catch (error) { if (!current()) return null; throw error; }
  }
  stop(): void { this.closed = true; this.invalidatePreview(); this.packages.stop(); }
}
