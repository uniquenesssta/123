import { competitionKindOptions, stageKindOptions } from "../../components/competition";
import type { BootstrapResponse } from "../../types";
import { workspaceTaskAnchorNavigation } from "../../ui/navigation";
import { hierarchyOptions } from "./hierarchyOptions";
export function competitionStructure(state: BootstrapResponse, section: string, bindingCount: number): string {
  const disabled = state.data.database_configured ? "" : "disabled";
  const { competitionOptions, seasonOptions, stageOptions } = hierarchyOptions(state);
  return `      <section class="workspace-module-view ${section === "structure" ? "active" : ""}" data-workspace-section="structure">
        <div class="module-section-stack">
          <div class="module-section-heading"><div><span>赛事结构</span><h2>从赛事身份继续建立赛季、阶段与轮次</h2><p>每一层都依赖上一层的明确身份；所有创建操作保持在本页。</p></div></div>
          <div class="core-local-navigation">${workspaceTaskAnchorNavigation([
            { id: "rules-custom-competition", index: "A", label: "赛事身份", description: "创建自定义赛事" },
            { id: "rules-season-structure", index: "B", label: "赛季与阶段", description: "建立赛季、阶段、轮次" },
          ])}</div>
          <section id="rules-custom-competition" class="panel workspace-anchor-target">
            <div class="panel-heading"><div><span>赛事身份</span><h2>新增自定义赛事</h2></div><span class="status-pill">内置赛事不会被覆盖</span></div>
            <div class="two-column rules-admin-grid">
              <article class="subpanel"><label class="field"><span>赛事名称</span><input id="competition-name" placeholder="例如 地区邀请赛" /></label><div class="field-row"><label class="field"><span>赛事类型</span><select id="new-competition-kind">${competitionKindOptions(null)}</select></label><label class="field"><span>国家或地区</span><input id="competition-country" placeholder="例如 韩国（也可填写 KR）" /></label></div><label class="field"><span>比赛所在地时区</span><select id="competition-timezone"><option value="Asia/Seoul">韩国时间</option><option value="Asia/Tokyo">日本时间</option><option value="Asia/Shanghai">中国时间</option><option value="Europe/London">英国时间</option><option value="Europe/Paris">欧洲中部时间</option><option value="America/New_York">美国东部时间</option><option value="UTC">世界协调时间</option></select></label><button class="primary" data-action="create-competition" ${disabled}>创建赛事</button></article>
              <article class="subpanel"><div class="panel-heading"><div><span>当前状态</span><h2>赛事与规则概览</h2></div></div><div class="summary-number-grid"><div><strong>${state.data.competitions.length}</strong><span>赛事</span></div><div><strong>${state.data.seasons.length}</strong><span>赛季</span></div><div><strong>${state.data.stages.length}</strong><span>阶段</span></div><div><strong>${bindingCount}</strong><span>P4 规则</span></div></div><p class="field-note">内置赛事与用户赛事共用同一目录；升级不会删除用户自建赛事、赛季、球队或比赛。</p></article>
            </div>
          </section>
          <section id="rules-season-structure" class="panel workspace-anchor-target">
            <div class="panel-heading"><div><span>赛事层级</span><h2>创建赛季、阶段和轮次</h2></div><span class="field-note">按左至右顺序完成</span></div>
            <div class="three-column hierarchy-grid">
              <article class="subpanel"><div class="panel-heading"><div><span>赛季</span><h2>创建赛季</h2></div></div><label class="field"><span>所属赛事</span><select id="season-competition-id"><option value="">选择赛事</option>${competitionOptions}</select></label><label class="field"><span>赛季名称</span><input id="season-name" placeholder="2026" /></label><div class="field-row"><label class="field"><span>开始日期</span><input id="season-starts-on" type="date" /></label><label class="field"><span>结束日期</span><input id="season-ends-on" type="date" /></label></div><label class="field"><span>状态</span><select id="season-status"><option value="planned">计划中</option><option value="active">进行中</option><option value="completed">已完成</option><option value="archived">已归档</option></select></label><button class="primary" data-action="create-season" ${disabled}>创建赛季</button></article>
              <article class="subpanel"><div class="panel-heading"><div><span>阶段</span><h2>创建阶段</h2></div></div><label class="field"><span>所属赛季</span><select id="stage-season-id"><option value="">选择赛季</option>${seasonOptions}</select></label><label class="field"><span>阶段名称</span><input id="stage-name" placeholder="常规赛" /></label><div class="field-row"><label class="field"><span>阶段类型</span><select id="stage-kind">${stageKindOptions("league")}</select></label><label class="field"><span>顺序</span><input id="stage-sequence" type="number" value="1" /></label></div><button class="primary" data-action="create-stage" ${disabled}>创建阶段</button></article>
              <article class="subpanel"><div class="panel-heading"><div><span>轮次</span><h2>创建轮次</h2></div></div><label class="field"><span>所属阶段</span><select id="round-stage-id"><option value="">选择阶段</option>${stageOptions}</select></label><label class="field"><span>轮次名称</span><input id="round-name" placeholder="第 1 轮" /></label><label class="field"><span>顺序</span><input id="round-sequence" type="number" value="1" /></label><button class="primary" data-action="create-round" ${disabled}>创建轮次</button></article>
            </div>
          </section>
        </div>
      </section>

`;
}
