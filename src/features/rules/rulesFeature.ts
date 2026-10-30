import type { CompetitionBindingDraft } from "../../types";
import { syncHierarchyControls } from "../competitions/index";
import { RulesWorkflow, type RulesWorkflowPorts } from "./rulesWorkflow";

export interface RulesFeaturePorts extends RulesWorkflowPorts {
  busy<T>(operation: () => Promise<T>): Promise<T>;
  render(): void;
  active(): boolean;
  notify(message: string): void;
}

/** UI adapter: controls belong to Workspace; file identity and preview lifetime belong to RulesWorkflow. */
export class RulesFeature extends RulesWorkflow {
  constructor(private readonly ui: RulesFeaturePorts) {
    super({ ...ui,
      registerRulePackage: draft => ui.busy(() => ui.registerRulePackage(draft)),
      createBinding: draft => ui.busy(() => ui.createBinding(draft)),
      previewRoute: command => ui.busy(() => ui.previewRoute(command)),
    });
  }
  async readPackage(file: { text(): Promise<string> }): Promise<void> {
    const pending = this.packages.read(file);
    if (this.ui.active()) this.ui.render();
    try {
      if (await pending && this.ui.active()) this.ui.notify("规则包已读取，请核对摘要后注册");
    } finally { if (this.ui.active()) this.ui.render(); }
  }
  clearPackage(): void { this.packages.clear(); if (this.ui.active()) this.ui.render(); }
  async registerPackage(): Promise<void> {
    await this.register();
    if (this.ui.active()) { this.ui.render(); this.ui.notify("规则包已校验并注册"); }
  }
  async createBinding(root: ParentNode): Promise<void> {
    const value = (id: string): string => {
      const input = root.querySelector<HTMLInputElement | HTMLSelectElement>(`#${id}`);
      if (!input) throw new Error(`缺少表单字段：${id}`);
      return input.value.trim();
    };
    const draft: CompetitionBindingDraft = { binding_name: value("binding-name") || null, competition_id: value("binding-competition-id") || null, season_id: value("binding-season-id") || null, stage_id: value("binding-stage-id") || null, competition_kind: null, rule_package_id: value("binding-rule-package-id"), priority: Number(value("binding-priority")), valid_from: null, valid_to: null };
    await this.saveBinding(draft);
    if (this.ui.active()) { this.ui.render(); this.ui.notify("赛事专属路由已建立；正式运行仍由后端重新判定"); }
  }
  syncHierarchy(root: ParentNode, changed?: string): void {
    const state = this.ui.snapshot(); if (!state) return;
    syncHierarchyControls(root, state, "binding-", changed);
    syncHierarchyControls(root, state, "", changed);
  }
}
