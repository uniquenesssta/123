import type { CoachDraft, FormationUsageDistributionDraft, TeamCoachPeriodDraft, TeamDraft, TeamProfileDraft } from "../../types";
import type { TeamCatalog } from "./teamCatalog";
import type { TeamDrafts } from "./teamDrafts";
import type { TeamApi } from "./teamPorts";
import { currentLocalizedTeamName } from "./teamPresentation";

export function teamValue(root: ParentNode, id: string): string { const field = root.querySelector<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>(`#${id}`); if (!field) throw new Error(`缺少表单字段：${id}`); return field.value.trim(); }
const nullable = (root: ParentNode, id: string) => teamValue(root, id) || null;
const number = (root: ParentNode, id: string) => { const value = teamValue(root, id); return value === "" ? null : Number(value); };
const checked = (root: ParentNode, id: string) => { const field = root.querySelector<HTMLInputElement>(`#${id}`); if (!field) throw new Error(`缺少复选框：${id}`); return field.checked; };
const todayDate = () => new Date().toISOString().slice(0, 10);

interface EditorPorts {
  api: TeamApi; catalog: TeamCatalog; drafts: TeamDrafts;
  root(): ParentNode; busy<T>(operation: () => Promise<T>): Promise<T>;
  render(): void; notify(message: string, kind?: "normal" | "success" | "error"): void;
  refreshPlayerReferences(): Promise<void>;
  publishCoaches(coaches: Awaited<ReturnType<TeamApi["listCoaches"]>>): void;
}

/** Captures edit identity and DTOs before awaits. A late save never rebinds the current team's form. */
export class TeamEditor {
  constructor(private readonly ports: EditorPorts) {}
  private async write(teamId: string, keys: string[], operation: (current: () => boolean) => Promise<unknown>, message: string): Promise<void> {
    const current = this.ports.catalog.assertTeam(teamId), values = this.ports.drafts.submitted(this.ports.root());
    const submitted = Object.fromEntries(Object.entries(values).filter(([key]) => keys.includes(key)));
    try { await this.ports.busy(() => operation(current)); }
    catch (error) { if (!current()) { this.ports.notify("原球队操作未能完整确认；当前对象已切换，请核对原对象结果", "normal"); return; } throw error; }
    if (!current()) { this.ports.notify("原球队操作已返回；当前对象已切换，请刷新核对原对象结果", "normal"); return; }
    try {
      if (!await this.ports.catalog.refreshDetail(teamId) || !current()) return;
      this.ports.drafts.acknowledge(teamId, submitted); this.ports.render(); this.ports.notify(message, "success");
    } catch (error) { throw new Error("保存已完成，但资料刷新失败；请核对结果，勿重复提交", { cause: error }); }
  }
  async update(teamId: string): Promise<void> {
    const root = this.ports.root(), detail = this.ports.catalog.detail;
    this.ports.catalog.assertTeam(teamId);
    const localizedName = teamValue(root, "team-localized-name"), previous = detail ? currentLocalizedTeamName(detail).trim() : "";
    const draft: TeamDraft = { canonical_name: teamValue(root, "team-canonical-name"), country_code: nullable(root, "team-country-code"), metadata: { entry_mode: "manual_team_center" } };
    await this.write(teamId, ["team-localized-name", "team-canonical-name", "team-country-code"], async current => {
      await this.ports.api.updateTeam(teamId, draft);
      try {
        if (!current()) throw new Error("数据上下文已切换，中文名称尚未保存");
        if (localizedName && localizedName !== previous) await this.ports.api.addTeamName({ team_id: teamId, name: localizedName, language_code: "zh-CN", valid_from: todayDate(), valid_to: null });
      } catch (error) { throw new Error("球队基础身份已保存，中文名称保存失败；请核对名称后继续", { cause: error }); }
      if (current()) await this.ports.refreshPlayerReferences();
    }, "球队基础身份已更新并重新载入");
  }
  async addName(teamId: string): Promise<void> {
    const root = this.ports.root(), name = teamValue(root, "team-alias-name"), language = nullable(root, "team-alias-language");
    if (!name || !language) throw new Error("请输入名称并选择名称语言");
    await this.write(teamId, ["team-alias-name", "team-alias-language"], () => this.ports.api.addTeamName({ team_id: teamId, name, language_code: language, valid_from: todayDate(), valid_to: null }), "球队名称已保存并持续显示");
  }
  async profile(teamId: string): Promise<void> {
    const root = this.ports.root(); this.ports.catalog.assertTeam(teamId);
    const draft: TeamProfileDraft = {
      short_name: nullable(root, "team-short-name"), team_type: teamValue(root, "team-profile-type") as TeamProfileDraft["team_type"], founded_year: number(root, "team-founded-year"), city: nullable(root, "team-city"), stadium: nullable(root, "team-stadium"), head_coach: this.ports.catalog.detail?.profile?.head_coach ?? null,
      default_formation: nullable(root, "team-default-formation"), tactical_style: teamValue(root, "team-tactical-style") as TeamProfileDraft["tactical_style"], attack_rating: number(root, "team-attack-rating"), midfield_rating: number(root, "team-midfield-rating"), defence_rating: number(root, "team-defence-rating"), goalkeeper_rating: number(root, "team-goalkeeper-rating"), reputation: number(root, "team-reputation"), data_confidence: Number(teamValue(root, "team-profile-confidence")), notes: nullable(root, "team-profile-notes"), metadata: { source: "manual_team_center", organization_model: "identity_roster_tactics_ratings_history" },
    };
    const keys = ["team-short-name", "team-profile-type", "team-founded-year", "team-city", "team-stadium", "team-default-formation", "team-tactical-style", "team-attack-rating", "team-midfield-rating", "team-defence-rating", "team-goalkeeper-rating", "team-reputation", "team-profile-confidence", "team-profile-notes"];
    await this.write(teamId, keys, () => this.ports.api.upsertTeamProfile(teamId, draft), "球队档案与战术资料已保存");
  }
  async formation(teamId: string): Promise<void> {
    const root = this.ports.root(), scope = teamValue(root, "formation-scope-type") as FormationUsageDistributionDraft["scope_type"], coach = nullable(root, "formation-coach-id");
    if (scope === "team_coach" && !coach) throw new Error("球队 + 教练作用域必须选择教练");
    const entries = [...root.querySelectorAll<HTMLInputElement>(".formation-usage-count")].map(input => ({ formation_id: input.dataset.formationId ?? "", usage_count: Number(input.value || 0) })).filter(item => item.formation_id && item.usage_count > 0);
    const draft: FormationUsageDistributionDraft = { scope_type: scope, team_id: teamId, coach_id: scope === "team_coach" ? coach : null, competition_id: null, window_preset: teamValue(root, "formation-window-preset") as FormationUsageDistributionDraft["window_preset"], window_start: nullable(root, "formation-window-start"), window_end: nullable(root, "formation-window-end"), observed_matches: Number(teamValue(root, "formation-observed-matches")), confidence: Number(teamValue(root, "formation-confidence")), alpha: Number(teamValue(root, "formation-alpha")), source_document_id: null, metadata: { source: "manual_team_center" }, entries };
    const keys = [...root.querySelectorAll<HTMLInputElement>("[id^='formation-'], .formation-usage-count")].map(item => item.id);
    await this.write(teamId, keys, () => this.ports.api.saveFormationUsageDistribution(draft), "阵型观察已保存并归一化");
  }
  async coachPeriod(teamId: string): Promise<void> {
    const root = this.ports.root(), coachId = nullable(root, "team-coach-id"), from = nullable(root, "team-coach-valid-from");
    if (!coachId || !from) throw new Error("请选择教练和任期开始日期");
    const role = teamValue(root, "team-coach-role") as TeamCoachPeriodDraft["role"];
    const draft: TeamCoachPeriodDraft = { team_id: teamId, coach_id: coachId, role, valid_from: from, valid_to: nullable(root, "team-coach-valid-to"), is_interim: checked(root, "team-coach-interim") || role === "interim_head_coach" || role === "caretaker", confidence: Number(teamValue(root, "team-coach-confidence")), source_document_id: null, end_previous: checked(root, "team-coach-end-previous"), metadata: { entry_mode: "manual_team_center" } };
    await this.write(teamId, ["team-coach-id", "team-coach-role", "team-coach-valid-from", "team-coach-valid-to", "team-coach-interim", "team-coach-confidence", "team-coach-end-previous"], async current => {
      await this.ports.api.addTeamCoachPeriod(draft); const coaches = await this.ports.api.listCoaches({ search: null, active_only: false, limit: 500 }); if (current()) this.ports.publishCoaches(coaches);
    }, "教练任期已保存，主教练投影已刷新");
  }
  async createCoach(): Promise<void> {
    const root = this.ports.root(), name = teamValue(root, "new-coach-name"); if (!name) throw new Error("请输入教练正式姓名");
    const draft: CoachDraft = { canonical_name: name, nationality_code: nullable(root, "new-coach-nationality"), status: "active", metadata: { entry_mode: "manual_team_center" } };
    const current = this.ports.catalog.capture();
    const created = await this.ports.busy(() => this.ports.api.createCoach(draft));
    if (!current()) return;
    const coaches = await this.ports.api.listCoaches({ search: null, active_only: false, limit: 500 });
    if (!current()) return; this.ports.publishCoaches(coaches); this.ports.render(); this.ports.notify(`教练“${created.canonical_name}”已创建`, "success");
  }
}
