import { escapeHtml, formatPercent } from "../components/format";

const visualLabels: Record<string, string> = {
  id: "记录编号", run_id: "运行编号", match_id: "比赛记录", player_id: "球员记录", team_id: "球队记录",
  status: "状态", name: "名称", title: "标题", display_name: "显示名称", canonical_name: "球员姓名",
  competition_name: "赛事", competition_kind: "赛事类型", season_name: "赛季", stage_name: "阶段", round_name: "轮次",
  home_team_name: "主队", away_team_name: "客队", home_win: "主胜概率", draw: "平局概率", away_win: "客胜概率",
  btts: "双方都进球概率", over_2_5: "总进球大于 2.5 概率", kickoff_time: "开球时间", captured_at: "记录时间",
  created_at: "创建时间", updated_at: "更新时间", valid_from: "生效时间", valid_to: "有效至", finalized_at: "确认时间",
  reason: "原因", confidence: "可信度", priority: "匹配优先级", model_id: "模型", model_version: "模型版本",
  parameter_version: "参数版本", package_display_name: "规则包", package_version: "规则包版本", rule_package_name: "规则包",
  source: "选择来源", message: "说明", summary: "结果摘要", details: "详细内容", evidence: "依据",
  output: "推演结果", input: "输入信息", identity: "模型身份", route: "模型选择路径", metrics: "指标",
  metadata: "补充信息", warnings: "提醒", blocking_errors: "需要先处理的问题", suggestions: "建议",
  sample_size: "样本数量", data_coverage: "数据覆盖率", duration_ms: "计算耗时（毫秒）", snapshot_type: "数据时点",
  lineup_type: "阵容类型", formation: "阵型", player_count: "球员人数", starter_count: "首发人数",
  quality_score: "数据可信度", current_value: "当前值", proposed_value: "建议值", calculation_version: "计算版本",
  error_message: "失败原因", progress: "完成进度", attempts: "已尝试次数", max_attempts: "最多尝试次数",
};

const visualTokenLabels: Record<string, string> = {
  home: "主队", away: "客队", team: "球队", player: "球员", match: "比赛", competition: "赛事", season: "赛季",
  stage: "阶段", round: "轮次", win: "胜率", draw: "平局", goal: "进球", goals: "进球", probability: "概率",
  score: "得分", value: "数值", current: "当前", proposed: "建议", expected: "预期", actual: "实际",
  model: "模型", version: "版本", package: "规则包", rule: "规则", parameter: "参数", source: "来源",
  data: "数据", quality: "质量", coverage: "覆盖率", confidence: "可信度", status: "状态", type: "类型",
  created: "创建", updated: "更新", valid: "有效", from: "开始", to: "结束", time: "时间", at: "时间",
  count: "数量", size: "规模", reason: "原因", message: "说明", summary: "摘要", detail: "详情", details: "详情",
  rank: "排名", priority: "优先级", result: "结果", evaluation: "评估", review: "复盘", lineup: "阵容",
};

function visualLabel(key: string): string {
  const direct = visualLabels[key];
  if (direct) return direct;
  const translated = key.split("_").map((token) => visualTokenLabels[token] ?? "").filter(Boolean).join("");
  return translated || "补充信息";
}

function visualScalar(value: string | number | boolean, key: string): string {
  if (typeof value === "boolean") return value ? "是" : "否";
  if (typeof value === "number" && /(confidence|probability|coverage|ratio|rate)$/.test(key.toLowerCase()) && value >= 0 && value <= 1) {
    return formatPercent(value);
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return Number.isInteger(value) ? String(value) : value.toFixed(4).replace(/0+$/, "").replace(/\.$/, "");
  }
  if (typeof value === "string" && /(_at|_time|valid_from|valid_to)$/.test(key)) {
    const parsed = Date.parse(value);
    if (!Number.isNaN(parsed)) return new Date(parsed).toLocaleString();
  }
  if (typeof value === "string") {
    const labels: Record<string, string> = {
      PASS: "通过",
      CHECK: "需要检查",
      succeeded: "已完成",
      pending: "待处理",
      failed: "失败",
      league: "联赛",
      group_stage: "小组赛",
      knockout_single_leg: "单回合淘汰赛",
      knockout_two_leg: "两回合淘汰赛",
      friendly: "友谊赛",
      custom: "自定义",
    };
    return labels[value] ?? value;
  }
  return String(value);
}

function isTechnicalVisualKey(key: string): boolean {
  return /(^id$|_id$|sha|hash|payload|metadata|binding_id|package_key|model_version_id|parameter_set_id|route_reason)/i.test(key);
}

function renderVisualValue(value: unknown, key = "value", depth = 0): string {
  if (value === null || value === undefined || value === "") return `<span class="visual-empty">未记录</span>`;
  if (["string", "number", "boolean"].includes(typeof value)) {
    return `<span class="visual-value">${escapeHtml(visualScalar(value as string | number | boolean, key))}</span>`;
  }
  if (Array.isArray(value)) {
    if (value.length === 0) return `<span class="visual-empty">暂无记录</span>`;
    const visible = value.slice(0, 30);
    return `<div class="detail-list">${visible.map((item, index) => `<article><span class="visual-index">${index + 1}</span><div>${renderVisualValue(item, key, depth + 1)}</div></article>`).join("")}${value.length > visible.length ? `<div class="visual-more">另有 ${value.length - visible.length} 条记录未展开</div>` : ""}</div>`;
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    if (entries.length === 0) return `<span class="visual-empty">暂无内容</span>`;
    const normal = entries.filter(([entryKey]) => !isTechnicalVisualKey(entryKey));
    const technical = entries.filter(([entryKey]) => isTechnicalVisualKey(entryKey));
    const scalarEntries = normal.filter(([, entryValue]) => entryValue === null || ["string", "number", "boolean"].includes(typeof entryValue));
    const sectionEntries = normal.filter(([, entryValue]) => entryValue !== null && !["string", "number", "boolean"].includes(typeof entryValue));
    const scalarHtml = scalarEntries.length > 0
      ? `<dl class="detail-facts">${scalarEntries.map(([entryKey, entryValue]) => `<div><dt>${escapeHtml(visualLabel(entryKey))}</dt><dd>${renderVisualValue(entryValue, entryKey, depth + 1)}</dd></div>`).join("")}</dl>`
      : "";
    const sectionHtml = sectionEntries.map(([entryKey, entryValue]) => `<section class="detail-section"><h3>${escapeHtml(visualLabel(entryKey))}</h3>${renderVisualValue(entryValue, entryKey, depth + 1)}</section>`).join("");
    const technicalHtml = technical.length > 0
      ? `<details class="technical-details"><summary>技术追踪信息</summary><dl class="detail-facts compact">${technical.map(([entryKey, entryValue]) => `<div><dt>${escapeHtml(visualLabel(entryKey))}</dt><dd>${renderVisualValue(entryValue, entryKey, depth + 1)}</dd></div>`).join("")}</dl></details>`
      : "";
    return `<div class="detail-stack ${depth > 0 ? "nested" : ""}">${scalarHtml}${sectionHtml}${technicalHtml}</div>`;
  }
  return `<span class="visual-value">${escapeHtml(String(value))}</span>`;
}

interface WorkspaceControlState {
  readonly key: string;
  readonly value: string;
  readonly checked: boolean | null;
  readonly selectedValues: readonly string[] | null;
}

interface WorkspacePanelEntry {
  readonly title: string;
  readonly subtitle: string;
  readonly body: string;
  readonly footer: string;
  readonly panelClass: string;
  readonly action: (() => Promise<void>) | null;
  formState?: WorkspaceControlState[];
  bodyScrollTop?: number;
  activeControlKey?: string | null;
}

export class ModalController {
  private readonly history: WorkspacePanelEntry[] = [];
  private historyIndex = -1;

  private currentEntry(): WorkspacePanelEntry | null {
    return this.historyIndex >= 0 ? (this.history[this.historyIndex] ?? null) : null;
  }

  private controlKey(control: HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement, index: number): string {
    return control.id || control.getAttribute("name") || control.dataset.workspaceControlKey || `${control.tagName.toLowerCase()}:${index}`;
  }

  private captureCurrentState(): void {
    const entry = this.currentEntry();
    const root = document.querySelector<HTMLDivElement>("#modal-root");
    if (!entry || !root) return;
    const controls = Array.from(root.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>("input, select, textarea"));
    entry.formState = controls.map((control, index) => ({
      key: this.controlKey(control, index),
      value: control.value,
      checked: control instanceof HTMLInputElement && ["checkbox", "radio"].includes(control.type) ? control.checked : null,
      selectedValues: control instanceof HTMLSelectElement && control.multiple
        ? Array.from(control.selectedOptions).map((option) => option.value)
        : null,
    }));
    entry.bodyScrollTop = root.querySelector<HTMLElement>(".workspace-detail-body")?.scrollTop ?? 0;
    const active = document.activeElement;
    const activeIndex = active instanceof HTMLInputElement || active instanceof HTMLSelectElement || active instanceof HTMLTextAreaElement
      ? controls.indexOf(active)
      : -1;
    entry.activeControlKey = activeIndex >= 0 ? this.controlKey(controls[activeIndex], activeIndex) : null;
  }

  private restoreCurrentState(entry: WorkspacePanelEntry, root: HTMLDivElement): void {
    const controls = Array.from(root.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>("input, select, textarea"));
    const states = new Map((entry.formState ?? []).map((state) => [state.key, state]));
    controls.forEach((control, index) => {
      const state = states.get(this.controlKey(control, index));
      if (!state) return;
      if (control instanceof HTMLSelectElement && control.multiple && state.selectedValues) {
        const selected = new Set(state.selectedValues);
        Array.from(control.options).forEach((option) => { option.selected = selected.has(option.value); });
      } else {
        control.value = state.value;
      }
      if (control instanceof HTMLInputElement && state.checked !== null) control.checked = state.checked;
    });
    const body = root.querySelector<HTMLElement>(".workspace-detail-body");
    if (body) body.scrollTop = entry.bodyScrollTop ?? 0;
    if (entry.activeControlKey) {
      const control = controls.find((item, index) => this.controlKey(item, index) === entry.activeControlKey);
      control?.focus({ preventScroll: true });
    }
  }

  private syncHistoryButtons(): void {
    const canBack = this.historyIndex >= 0;
    const canForward = this.historyIndex + 1 < this.history.length;
    document.querySelectorAll<HTMLButtonElement>("[data-action='workspace-history-back']")
      .forEach((button) => { button.disabled = !canBack; });
    document.querySelectorAll<HTMLButtonElement>("[data-action='workspace-history-forward']")
      .forEach((button) => { button.disabled = !canForward; });
  }

  private renderCurrent(): void {
    const root = document.querySelector<HTMLDivElement>("#modal-root");
    const main = document.querySelector<HTMLElement>(".main-content");
    const page = document.querySelector<HTMLElement>(".page-container");
    const entry = this.currentEntry();
    if (!root || !main || !page) {
      this.syncHistoryButtons();
      return;
    }
    if (!entry) {
      root.replaceChildren();
      main.classList.remove("workspace-panel-open");
      page.removeAttribute("aria-hidden");
      this.syncHistoryButtons();
      return;
    }
    main.classList.add("workspace-panel-open");
    page.setAttribute("aria-hidden", "true");
    root.innerHTML = `<section class="workspace-detail-page ${entry.panelClass}" role="region" aria-label="${escapeHtml(entry.title)}">
      <div class="workspace-detail-toolbar">
        <div class="workspace-detail-history" aria-label="页面历史">
          <button type="button" class="secondary compact" data-action="workspace-history-back">← 返回</button>
          <button type="button" class="secondary compact" data-action="workspace-history-forward">前进 →</button>
        </div>
        <button type="button" class="secondary compact" data-action="close-workspace-detail">返回当前页面</button>
      </div>
      <header class="workspace-detail-header"><div><span>${escapeHtml(entry.subtitle)}</span><h2>${escapeHtml(entry.title)}</h2></div></header>
      <div class="workspace-detail-body">${entry.body}</div>
      ${entry.footer ? `<footer class="workspace-detail-footer">${entry.footer}</footer>` : ""}
    </section>`;
    this.syncHistoryButtons();
    this.restoreCurrentState(entry, root);
    if (!entry.activeControlKey) root.querySelector<HTMLElement>("input, select, textarea, button:not([disabled])")?.focus();
  }

  private push(entry: WorkspacePanelEntry): void {
    this.captureCurrentState();
    if (this.historyIndex < 0) {
      this.history.splice(0, this.history.length);
    } else {
      this.history.splice(this.historyIndex + 1);
    }
    this.history.push(entry);
    this.historyIndex = this.history.length - 1;
    this.renderCurrent();
  }

  restore(): void {
    this.renderCurrent();
  }

  close(): void {
    this.captureCurrentState();
    this.historyIndex = -1;
    this.renderCurrent();
  }

  reset(): void {
    this.history.splice(0, this.history.length);
    this.historyIndex = -1;
    this.renderCurrent();
  }

  back(): void {
    this.captureCurrentState();
    if (this.historyIndex >= 0) this.historyIndex -= 1;
    this.renderCurrent();
  }

  forward(): void {
    this.captureCurrentState();
    if (this.historyIndex + 1 < this.history.length) this.historyIndex += 1;
    this.renderCurrent();
  }

  showHtml(
    title: string,
    subtitle: string,
    body: string,
    footer = "",
    panelClass = "",
  ): void {
    const safeClass = panelClass.trim().replace(/[^a-zA-Z0-9_-]+/g, " ");
    this.push({ title, subtitle, body, footer, panelClass: safeClass, action: null });
  }

  show(title: string, payload: unknown, footer = ""): void {
    this.showHtml(title, "可视化详情", renderVisualValue(payload), footer);
  }

  confirm(
    title: string,
    description: string,
    facts: Array<[string, string]>,
    confirmLabel: string,
    action: () => Promise<void>,
  ): void {
    const body = `<div class="confirm-visual"><div class="confirm-icon">!</div><p>${escapeHtml(description)}</p><div class="visual-grid">${facts.map(([label, value]) => `<div class="visual-field"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`).join("")}</div></div>`;
    const footer = `<button type="button" class="secondary" data-action="close-workspace-detail">取消</button><button type="button" class="primary danger-action" data-action="confirm-workspace-action">${escapeHtml(confirmLabel)}</button>`;
    this.push({ title, subtitle: "请确认", body, footer, panelClass: "confirmation-workspace", action });
  }

  async runPendingAction(): Promise<void> {
    const action = this.currentEntry()?.action ?? null;
    this.close();
    if (action) await action();
  }
}
