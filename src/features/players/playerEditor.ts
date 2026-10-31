import type * as T from "../../types";
import type { PlayerCatalog } from "./playerCatalog";
import type { PlayerDrafts } from "./playerDrafts";
import type { PlayerApi } from "./playerPorts";
import { currentLocalizedPlayerName } from "./playerPresentation";
const value = (root: ParentNode, id: string): string => { const field = root.querySelector<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>("#"+id); if (!field) throw new Error("缺少球员表单字段："+id); return field.value; };
const nullableValue = (root: ParentNode, id: string) => value(root,id).trim() || null;
const nullableNumber = (root: ParentNode, id: string) => { const raw = value(root,id).trim(); if (!raw) return null; const number = Number(raw); if (!Number.isFinite(number)) throw new Error("数字格式无效："+id); return number; };
const checked = (root: ParentNode,id: string) => { const input=root.querySelector<HTMLInputElement>("#"+id); if(!input)throw new Error("缺少球员复选项："+id);return input.checked; };
const todayDate = () => new Date().toISOString().slice(0,10);
function localDateTimeToIso(
  raw: string | null,
  fallbackNow = false,
): string | null {
  if (!raw) return fallbackNow ? new Date().toISOString() : null;
  const parsed = new Date(raw);
  if (Number.isNaN(parsed.getTime())) throw new Error(`时间格式无效：${raw}`);
  return parsed.toISOString();
}
interface EditorPorts { refreshAfterCreate():Promise<boolean>; api: PlayerApi; catalog: PlayerCatalog; drafts: PlayerDrafts; root(): ParentNode; busy<T>(operation:()=>Promise<T>):Promise<T>; render():void; notify(message:string,kind?:"normal"|"success"|"error"):void; showResult(title:string,result:unknown):void; }
export class PlayerEditor {
 constructor(private readonly ports: EditorPorts) {}
 private async write(id:string,keys:string[],operation:(current:()=>boolean)=>Promise<unknown>,message:string):Promise<void>{
  const current=this.ports.catalog.assertPlayer(id),values=this.ports.drafts.submitted(this.ports.root()),submitted=Object.fromEntries(Object.entries(values).filter(([key])=>keys.includes(key)));
  try {await this.ports.busy(()=>operation(current));}catch(error){if(!current()){this.ports.notify("原球员操作未能完整确认；当前对象已切换，请核对原对象结果");return;}throw error;}
  if(!current()){this.ports.notify("原球员操作已返回；当前对象已切换，请核对原结果");return;}
  try {if(!await this.ports.catalog.refreshDetail(id)||!current())return;this.ports.drafts.acknowledge(id,submitted);this.ports.render();this.ports.notify(message,"success");}catch(error){throw new Error("写入已完成，但球员资料刷新失败；请先核对结果，勿重复提交",{cause:error});}
 }
 async updatePlayer(playerId:string):Promise<void>{ const root=this.ports.root(); this.ports.catalog.assertPlayer(playerId);

  const height = nullableNumber(root,"edit-player-height");
  const localizedName = value(root,"edit-player-localized-name").trim();
  const previousLocalizedName = currentLocalizedPlayerName(this.ports.catalog.detail);
  const draft: T.PlayerDraft = {
    canonical_name: value(root,"edit-player-name").trim(),
    date_of_birth: nullableValue(root,"edit-player-birth"),
    nationality_code: nullableValue(root,"edit-player-nationality"),
    preferred_foot: value(root,"edit-player-foot") as T.PlayerDraft["preferred_foot"],
    height_cm: height,
    status: value(root,"edit-player-status") as T.PlayerDraft["status"],
    metadata: { entry_mode: "manual_edit" },
  };

 await this.write(playerId,["edit-player-height","edit-player-localized-name","edit-player-name","edit-player-birth","edit-player-nationality","edit-player-foot","edit-player-status"],async current => { await this.ports.api.updatePlayer(playerId,draft); if (localizedName && localizedName!==previousLocalizedName) { try {if(!current())throw new Error("数据上下文已切换，中文名称尚未保存");await this.ports.api.addPlayerName({player_id:playerId,name:localizedName,language_code:"zh-CN",is_primary:false,valid_from:todayDate(),valid_to:null});}catch(error){throw new Error("球员基础身份已保存，但中文名称保存未完成，请核对原对象",{cause:error});} } },"球员基础资料已更新并重新载入");
 }
 async addPlayerName(playerId:string):Promise<void>{ const root=this.ports.root(); this.ports.catalog.assertPlayer(playerId);

  const name = value(root,"player-alias-name").trim();
  const languageCode = nullableValue(root,"player-alias-language");
  if (!name) throw new Error("请输入需要保存的球员名称");
  if (!languageCode) throw new Error("请选择名称语言");
  const draft: T.PlayerNameDraft = {
    player_id: playerId,
    name,
    language_code: languageCode,
    is_primary: checked(root,"player-alias-primary"),
    valid_from: todayDate(),
    valid_to: null,
  };

 await this.write(playerId,["player-alias-name","player-alias-language","player-alias-primary"],() => this.ports.api.addPlayerName(draft),"球员名称已保存并持续显示");
 }
 async assignPlayerPosition(playerId:string):Promise<void>{ const root=this.ports.root(); this.ports.catalog.assertPlayer(playerId);

  const draft: T.PlayerPositionDraft = {
    player_id: playerId,
    position_code: value(root,"player-position-code"),
    proficiency: Number(value(root,"player-position-proficiency")),
    default_role_code: nullableValue(root,"player-position-default-role"),
    is_primary: checked(root,"player-position-primary"),
    valid_from: null,
    valid_to: null,
    source_document_id: null,
  };

 await this.write(playerId,["player-position-code","player-position-proficiency","player-position-default-role","player-position-primary"],() => this.ports.api.assignPlayerPosition(draft),"球员位置已记录");
 }
 async addPlayerTeamPeriod(playerId:string):Promise<void>{ const root=this.ports.root(); this.ports.catalog.assertPlayer(playerId);

  const teamId = nullableValue(root,"player-team-id");
  const validFrom = nullableValue(root,"player-team-valid-from");
  if (!teamId || !validFrom) throw new Error("请选择球队并填写效力开始日期");
  const squadNumber = nullableNumber(root,"player-squad-number");
  const draft: T.PlayerTeamPeriodDraft = {
    player_id: playerId,
    team_id: teamId,
    season_id: null,
    squad_number: squadNumber,
    valid_from: validFrom,
    valid_to: nullableValue(root,"player-team-valid-to"),
    registration_status: value(root,"player-registration-status"),
    source_document_id: null,
  };

 await this.write(playerId,["player-team-id","player-team-valid-from","player-squad-number","player-team-valid-to","player-registration-status"],() => this.ports.api.addPlayerTeamPeriod(draft),"球员效力时间线已更新");
 }
 async addPlayerAvailability(playerId:string):Promise<void>{ const root=this.ports.root(); this.ports.catalog.assertPlayer(playerId);

  const validFrom = localDateTimeToIso(
    nullableValue(root,"player-availability-from"),
    true,
  );
  if (!validFrom) throw new Error("可用性生效时间无效");
  const draft: T.PlayerAvailabilityDraft = {
    player_id: playerId,
    team_id: nullableValue(root,"player-team-id"),
    competition_id: null,
    status: value(root,
      "player-availability-status",
    ) as T.PlayerAvailabilityDraft["status"],
    reason: nullableValue(root,"player-availability-reason"),
    confidence: Number(value(root,"player-availability-confidence")),
    valid_from: validFrom,
    valid_to: localDateTimeToIso(nullableValue(root,"player-availability-to")),
    source_document_id: null,
    metadata: {},
  };

 await this.write(playerId,["player-availability-from","player-team-id","player-availability-reason","player-availability-confidence","player-availability-to"],() => this.ports.api.addPlayerAvailability(draft),"球员可用性已记录");
 }
 async addPlayerAbility(playerId:string):Promise<void>{ const root=this.ports.root(); this.ports.catalog.assertPlayer(playerId);

  const observedAt = localDateTimeToIso(
    nullableValue(root,"player-ability-observed-at"),
    true,
  );
  if (!observedAt) throw new Error("能力观察时间无效");
  const draft: T.PlayerAbilityObservationDraft = {
    player_id: playerId,
    dimension_code: value(root,"player-ability-dimension"),
    context_type: "general",
    context_id: null,
    value: Number(value(root,"player-ability-value")),
    confidence: Number(value(root,"player-ability-confidence")),
    sample_size: Number(value(root,"player-ability-sample-size")),
    observed_at: observedAt,
    effective_from: observedAt,
    effective_to: null,
    calculation_version: "manual.v1",
    source_document_id: null,
    metadata: { entry_mode: "manual" },
  };

 await this.write(playerId,["player-ability-observed-at","player-ability-dimension","player-ability-value","player-ability-confidence","player-ability-sample-size"],() => this.ports.api.addPlayerAbilityObservation(draft),"能力观察已写入，当前能力投影已更新");
 }
 async addPlayerExternalId(playerId:string):Promise<void>{ const root=this.ports.root(); this.ports.catalog.assertPlayer(playerId);

  const providerId = nullableValue(root,"player-provider-id");
  if (!providerId) throw new Error("请先创建并选择数据源");
  const draft: T.ExternalEntityIdDraft = {
    provider_id: providerId,
    entity_type: "player",
    entity_id: playerId,
    external_id: value(root,"player-external-id").trim(),
    metadata: {},
  };

 await this.write(playerId,["player-provider-id","player-external-id"],() => this.ports.api.addExternalEntityId(draft),"外部球员 ID 已绑定");
 }
 async addPlayerDynamicTag(playerId:string):Promise<void>{ const root=this.ports.root(); this.ports.catalog.assertPlayer(playerId);

  const now = new Date();
  const validFrom =
    localDateTimeToIso(nullableValue(root,"player-dynamic-tag-from")) ??
    now.toISOString();
  const validTo =
    localDateTimeToIso(nullableValue(root,"player-dynamic-tag-to")) ??
    new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000).toISOString();
  const draft: T.PlayerDynamicTagDraft = {
    player_id: playerId,
    tag_code: value(root,"player-dynamic-tag-code"),
    value: Number(value(root,"player-dynamic-tag-value")),
    label: nullableValue(root,"player-dynamic-tag-label"),
    confidence: Number(value(root,"player-dynamic-tag-confidence")),
    observed_at: now.toISOString(),
    valid_from: validFrom,
    valid_to: validTo,
    competition_id: nullableValue(root,"player-dynamic-tag-competition"),
    position_code: nullableValue(root,"player-dynamic-tag-position"),
    opponent_team_id: null,
    sample_size: 1,
    source_type: "manual",
    calculation_version: "manual.v1",
    source_document_id: null,
    metadata: {},
  };

 await this.write(playerId,["player-dynamic-tag-from","player-dynamic-tag-to","player-dynamic-tag-code","player-dynamic-tag-value","player-dynamic-tag-label","player-dynamic-tag-confidence","player-dynamic-tag-competition","player-dynamic-tag-position"],() => this.ports.api.addPlayerDynamicTag(draft),"动态标签已保存，并将在失效时间后自动退出计算");
 }
async calculatePlayerContribution(playerId: string): Promise<void> {
  const current=this.ports.catalog.assertPlayer(playerId), root=this.ports.root();
  const request: T.PlayerMatchContributionRequest = {
    player_id: playerId,
    match_id: null,
    competition_id: nullableValue(root,"player-dynamic-tag-competition"),
    position_code: nullableValue(root,"player-dynamic-tag-position"),
    role_code: null,
    role_origin: null,
    role_source_position_code: null,
    opponent_team_id: null,
    as_of: new Date().toISOString(),
    expected_minutes: null,
  };
  const result = await this.ports.busy(() =>
    this.ports.api.calculatePlayerMatchContribution(request),
  );
  if(current())this.ports.showResult("球员当前有效贡献", result);
}
async createPlayer():Promise<void>{const root=this.ports.root(),current=this.ports.catalog.capture();
  const height = nullableNumber(root,"new-player-height");
  const draft: T.PlayerDraft = {
    canonical_name: value(root,"new-player-name").trim(),
    date_of_birth: nullableValue(root,"new-player-birth"),
    nationality_code: nullableValue(root,"new-player-nationality"),
    preferred_foot: value(root,"new-player-foot") as T.PlayerDraft["preferred_foot"],
    height_cm: height,
    status: "active",
    metadata: {},
  };

const created=await this.ports.busy(()=>this.ports.api.createPlayer(draft));if(!current()){this.ports.notify("原球员创建已返回，请核对原库结果");return;}try{if(!await this.ports.refreshAfterCreate())return;if(await this.ports.catalog.open(created.id)){this.ports.render();this.ports.notify("球员已创建并加入目录","success");}}catch(error){throw new Error("球员已创建，但目录或详情刷新未完成；请先核对原对象，勿重复创建",{cause:error});}}
async createProvider():Promise<void>{const root=this.ports.root(),current=this.ports.catalog.capture();
  const providerName = value(root,"new-provider-name").trim();
  if (!providerName) throw new Error("请输入数据源名称");
  const draft: T.DataProviderDraft = {
    code: `provider-${Date.now()}`,
    name: providerName,
    provider_type: "football_data",
    base_url: nullableValue(root,"new-provider-url"),
    metadata: { entry_mode: "desktop" },
  };

await this.ports.busy(()=>this.ports.api.createDataProvider(draft));if(!current())return;try{await this.ports.catalog.refreshReferences();}catch(error){throw new Error("数据源已保存，但参考目录刷新失败，请核对结果，勿重复创建",{cause:error});}if(!current())return;this.ports.render();this.ports.notify("数据源已保存","success");}
}
