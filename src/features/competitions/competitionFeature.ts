import type { ModalController } from "../../app/modal";
import type { BootstrapResponse, CompetitionDraft, SeasonDraft, StageDraft, RoundDraft } from "../../types";
import { readCompetitionDraft, readHierarchyDrafts } from "./competitionDrafts";
import { competitionPathBody } from "./competitionPathView";

export interface CompetitionFeaturePorts {
  snapshot(): BootstrapResponse | null;
  createCompetition(draft: CompetitionDraft): Promise<unknown>;
  createSeason(draft: SeasonDraft): Promise<unknown>;
  createStage(draft: StageDraft): Promise<unknown>;
  createRound(draft: RoundDraft): Promise<unknown>;
  deleteCompetition(id: string): Promise<unknown>;
  busy<T>(operation: () => Promise<T>): Promise<T>;
  refresh(): Promise<boolean>;
  active(): boolean;
  render(): void;
  invalidateRoutes(): void;
  notify(message: string): void;
  confirm: ModalController["confirm"];
  showPath(body: string): void;
}

/** Catalogue writes share Session's summary; draft controls remain solely in Workspace. */
export class CompetitionFeature {
  constructor(private readonly ports: CompetitionFeaturePorts) {}
  private snapshot(): BootstrapResponse {
    const state = this.ports.snapshot();
    if (!state?.data.database_configured) throw new Error("请先连接数据库");
    return state;
  }
  private async write(operation: () => Promise<unknown>, message: string): Promise<void> {
    const state = this.snapshot();
    this.ports.invalidateRoutes();
    await this.ports.busy(operation);
    if (state !== this.ports.snapshot()) throw new Error("提交期间数据上下文已变化，请刷新核对结果后继续");
    if (!await this.ports.refresh()) throw new Error("操作已保存，但目录刷新未完成；请刷新核对，勿重复提交");
    if (this.ports.active()) { this.ports.render(); this.ports.notify(message); }
  }
  createCompetition(root: ParentNode): Promise<void> {
    const draft = readCompetitionDraft(root);
    return this.write(() => this.ports.createCompetition(draft), "赛事已创建");
  }
  createSeason(root: ParentNode): Promise<void> {
    const draft = readHierarchyDrafts(root, this.snapshot()).season();
    return this.write(() => this.ports.createSeason(draft), "赛季已创建");
  }
  createStage(root: ParentNode): Promise<void> {
    const draft = readHierarchyDrafts(root, this.snapshot()).stage();
    return this.write(() => this.ports.createStage(draft), "赛事阶段已创建");
  }
  createRound(root: ParentNode): Promise<void> {
    const draft = readHierarchyDrafts(root, this.snapshot()).round();
    return this.write(() => this.ports.createRound(draft), "比赛轮次已创建");
  }
  showPath(id: string): void { this.ports.showPath(competitionPathBody(this.snapshot(), id)); }
  requestDelete(id: string): void {
    const state = this.snapshot();
    const item = state.data.competitions.find(item => item.id === id && item.is_active);
    if (!item) throw new Error("赛事不存在或已停用，请刷新目录");
    this.ports.confirm("删除赛事", "赛事将从目录和自动路由中隐藏，已有比赛、推演和复盘历史会保留。是否允许停用由后端校验。", [["赛事", item.name], ["处理方式", "停用赛事并停用赛事级绑定"]], "确认删除", async () => {
      if (this.ports.snapshot() !== state) throw new Error("数据上下文已变化，请重新核对赛事后确认");
      await this.write(() => this.ports.deleteCompetition(id), "赛事已删除，历史数据仍然保留");
    });
  }
}
