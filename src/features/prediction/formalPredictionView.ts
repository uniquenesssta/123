import { iconSlot } from "../../ui/icons";
import { workspaceAnchorNavigation } from "../../ui/navigation/section-navigation/sectionNavigationView";
import { escapeHtml } from "../../components/format";
import { inlineMessageView } from "../../ui/feedback/inline-message";
import { routeCard } from "../rules/index";
import { predictionViewData } from "./predictionProjection";
import { inlinePredictionResult } from "./predictionResultView";
import { modelDisplayName,routeSourceLabel } from "./predictionLabels";
import type { PredictionFeature } from "./predictionFeature";

export interface FormalPredictionOptions {
  enabled:boolean;matchOptions:string;modelOptions:string;snapshotOptions:string;ruleOptions:string;
  parameter:string;guide:string;family:string;view?:Partial<PredictionFeature["view"]>;
}
const text=(value:unknown)=>escapeHtml(typeof value==="string"?value:"未记录");
export function formalPredictionView(o:FormalPredictionOptions):string{
 const v=o.view,r=v?.readiness,phase=v?.phase??"unchecked",busy=v?.busy??false,unknown=phase==="unknown";
 const level=r?.can_run_formal?"可正式推演":r?.can_run_shadow?"仅允许影子推演":r?"正式推演被阻断":phase==="checking"?"正在检查当前输入":phase==="changed"?"输入已变化，请重新检查":"尚未检查正式输入";
 const provider=inlineMessageView({title:phase==="unavailable"?"外部模型不可用":"外部模型未捆绑",message:"数据准备和路由可用；门禁通过不代表模型已接入。未取得提供器结果前，不显示概率或比分。",tone:"warning"});
 const stateMessage=v?.error?inlineMessageView({title:unknown?"推演结果尚未核实":phase==="result"?"运行记录":"本次操作未完成",message:v.error,tone:unknown?"warning":phase==="result"?"info":"danger",live:true}):"";
 const candidates=v?.candidates??[],actualAudit=v?.lastResult?predictionViewData(v.lastResult).inputAudit:null;
 return `<section class="workspace-module-view active prediction-formal-view" data-workspace-section="formal" data-prediction-phase="${phase}">
  ${provider}${stateMessage}
  <section id="prediction-form-setup" class="panel workspace-anchor-target prediction-settings">
    <header><h2>选择比赛、模型与数据窗口</h2><p>比赛身份、时点、模型及规则变更后，旧门禁结果失效，必须重新检查。</p></header>
    <fieldset class="prediction-fields" ${busy?"disabled":""}>
      <div class="prediction-selectors"><label class="field"><span>比赛</span><select data-select-chevron="icon" id="prediction-match-id" ${o.enabled?"":"disabled"}>${o.matchOptions||'<option value="">暂无可选比赛</option>'}</select></label><label class="field"><span>计算模型</span><select data-select-chevron="icon" id="prediction-model-family" data-search-placeholder="输入 P4、P7、联赛、淘汰赛或友谊赛">${o.modelOptions}</select></label><label class="field"><span>数据窗口</span><select data-select-chevron="icon" id="prediction-stored-snapshot">${o.snapshotOptions}</select></label></div>
      <div class="prediction-routing-selectors"><label class="field"><span>外部提供器规则入口</span><select data-select-chevron="icon" id="explicit-rule-package-id">${o.ruleOptions}</select></label><label class="field"><span>参数版本</span><select data-select-chevron="icon" disabled aria-label="参数版本"><option>${escapeHtml(o.parameter)}</option></select></label></div>
      <details class="prediction-model-guide"><summary>当前模型选择说明</summary>${o.guide}</details>
      <div class="button-row"><button class="secondary" data-action="dry-run">检查外部入口</button><button class="secondary" data-action="preview-stored-route" ${o.enabled?"":"disabled"}>查看模型判定</button></div>
    </fieldset>
  </section>
  <section id="prediction-form-readiness" class="panel workspace-anchor-target prediction-gate">
    <header><h2>赛前数据完整度门禁</h2><p>检查比赛身份、窗口、双方阵容、首发门将、位置角色、历史、路由与输入质量。</p></header>
    <strong>${escapeHtml(level)}${r?` · ${Number.isFinite(r.score)?r.score:"未记录"}/100`:""}</strong>
    <p>${r?escapeHtml([...r.blockers,...r.warnings].join("；")||"当前输入通过门禁；实际提供器能力独立核验。"):"未知不是 0 分；检查结果绑定比赛、窗口及模型身份，旧结果不得跨上下文复用。"}</p>
    <div class="button-row"><button class="primary" data-action="check-prediction-lineup-chain" ${busy||unknown||!o.enabled?"disabled":""}>检查数据完整度</button><button class="secondary" data-action="prepare-prediction-lineups" ${busy||!o.enabled?"disabled":""}>查看双方阵容</button>${r?'<button class="secondary" data-action="show-prediction-readiness">查看全部检查</button>':""}</div>
    ${r?`<details class="prediction-check-details"><summary>${r.checks.length} 项实际检查</summary><dl>${r.checks.map(c=>`<div><dt>${escapeHtml(c.label)}</dt><dd>${escapeHtml(c.status)} · ${escapeHtml(c.summary)}</dd></div>`).join("")}</dl></details>`:""}
  </section>
  <section class="panel prediction-audit"><header><h2>输入审计</h2></header><p>预检数据截止：${text(r?.data_cutoff_at??null)}　·　${actualAudit?"实际运行指纹":"预检输入指纹"}：<code>${text(actualAudit?.input_manifest_sha256??r?.input_manifest_sha256??null)}</code>　·　审计版本：${text(actualAudit?.audit_version??r?.audit_version??null)}</p></section>
  ${unknown?`<section class="panel prediction-recovery"><header><h2>查回原运行</h2><p>不会重新执行模型；相近记录须明确核对，未找到不代表请求失败。</p></header><p>原比赛：${text(v?.receipt?.command.match_id)} · ${text(v?.receipt?.command.snapshot_type)} · ${text(v?.receipt?.command.model_family)}</p><p>原指纹：<code>${text(v?.receipt?.manifest)}</code></p><div class="button-row"><button class="primary" data-action="prediction-readback" ${busy?"disabled":""}>查回已有运行</button>${v?.receipt?.mode==="shadow"?'<button class="secondary" data-action="prediction-retry-shadow">重新检查影子请求</button>':""}</div>${candidates.map(c=>`<article><p>运行：${text(c.id)}　·　窗口：${text(c.snapshot_type)}</p><p>实际指纹：<code>${text((c.input_audit as Record<string,unknown>|undefined)?.manifest_sha256)}</code></p><div class="button-row"><button class="secondary" data-action="prediction-open-candidate" data-run-id="${text(c.id)}">打开相关运行</button><button class="secondary" data-action="prediction-adopt-candidate" data-run-id="${text(c.id)}" ${busy?"disabled":""}>采用已核对的运行</button></div></article>`).join("")}</section>`:""}
  <div class="prediction-outputs"><article class="panel workspace-anchor-target" id="prediction-result">${v?.lastResult&&v.resultMode?inlinePredictionResult(v.lastResult,v.resultMode):'<header><h2>等待推演结果</h2></header><p>当前没有真实结果；不预填概率、比分或“成功”。</p>'}</article><article class="panel workspace-anchor-target" id="route-preview"><header><h2>为什么使用这个模型</h2></header>${v?.route?routeCard(v.route,{source:routeSourceLabel(v.route.source),model:modelDisplayName(v.route.model_id)}):"<p>查看模型判定后展示赛事、规则和模型的匹配依据。</p>"}</article></div>
 </section>`;
}
export function predictionExecutionBar(o:FormalPredictionOptions):string{
 const v=o.view,phase=v?.phase??"unchecked",formal=o.enabled&&(v?.canFormal??true),shadow=o.enabled&&(v?.canShadow??false);
 return `<footer class="prediction-execution-bar" aria-live="polite"><p>${phase==="unknown"?"原请求结果尚未核实；先查回已有运行，禁止直接重跑。":phase==="running"?"正在等待提供器返回；请勿重复提交。":"正式运行需同时满足当前输入门禁与实际提供器能力。"}</p><div class="button-row"><button class="primary" data-action="calculate-prediction-match" ${formal?"":"disabled"}>${phase==="running"?"正在运行":v?.readiness?"运行正式 "+escapeHtml(o.family.toUpperCase()):"检查并运行 "+escapeHtml(o.family.toUpperCase())}</button><button class="secondary" data-action="run-shadow-prediction-match" ${shadow?"":"disabled"}>运行影子 ${escapeHtml(o.family.toUpperCase())}</button><details class="prediction-locations" data-workspace-anchor-menu data-workspace-persist="false"><summary>页内定位</summary>${workspaceAnchorNavigation("正式推演", [
  { id: "prediction-form-setup", label: "推演设置" },
  { id: "prediction-form-readiness", label: "完整度门禁" },
  { id: "prediction-result", label: "概率结果" },
  { id: "route-preview", label: "模型判定" },
 ], "vertical")}</details></div></footer>`;
}

/** Reapply the sole context owner after generic Workspace recovery, before control enhancement. */
export function syncPredictionControls(root:ParentNode,command:import("../../types").StoredMatchPredictionCommand):void{
  root.querySelectorAll<HTMLDetailsElement>("[data-workspace-anchor-menu]").forEach(menu=>menu.addEventListener("keydown",event=>{
    if(event.key!=="Escape")return;
    event.preventDefault();menu.open=false;menu.querySelector<HTMLElement>("summary")?.focus();
  }));
  const values:Record<string,string>={"prediction-match-id":command.match_id,"prediction-model-family":command.model_family,"prediction-stored-snapshot":command.snapshot_type,"explicit-rule-package-id":command.explicit_rule_package_id??""};
  for(const[id,value]of Object.entries(values)){const select=root.querySelector<HTMLSelectElement>("#"+id);if(select&&Array.from(select.options).some(o=>o.value===value))select.value=value;}
}

/** Presentation belongs to this approved screen; shared select search/diagnostics remain unchanged. */
export function syncPredictionSelectIcons(root:ParentNode):void {
  root.querySelectorAll<HTMLElement>('.prediction-settings .searchable-select-toggle').forEach(toggle=>{toggle.innerHTML=iconSlot("chevron-down",16);});
}
