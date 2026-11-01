import { escapeHtml } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { workspacePaneToggle } from "../ui/layout";
import { iconSlot } from "../ui/icons";
import { previewPhase } from "../app/imports/spreadsheetBatch";
import { workbookPreview, importPhaseLabel } from "../features/workbooks/workbookPresentation";
import type { WorkbooksFeature } from "../features/workbooks/workbooksFeature";
import type { BootstrapResponse, MatchRecord, SpreadsheetImportPreview } from "../types";

export function workbooksPage(state:BootstrapResponse,teamPreview:SpreadsheetImportPreview|null,playerPreview:SpreadsheetImportPreview|null,matchPreview:SpreadsheetImportPreview|null,moduleSidebarCollapsed:boolean,inspectorCollapsed:boolean,activeSection:string,view?:WorkbooksFeature["state"],matches:MatchRecord[]=[],selectedMatchId:string=""):string {
  if(!state.data.database_configured)return `<section class="page-heading simple-heading"><div><h1>数据导入与导出</h1><p>连接数据库后导出、预检与提交工作簿。</p></div></section>${inlineDatabaseSetup("连接数据服务以使用工作包","导入必须先预检，再由用户确认提交。",state.connection_error)}`;
  const section=activeSection==="player"||activeSection==="match"?activeSection:"team";
  const previews={team:teamPreview,player:playerPreview,match:matchPreview},preview=previews[section],batch=view?.targets[section].batch;
  const labels={team:"球队月度",player:"球员月度",match:"比赛与阵容"};
  const copy={
    team:{title:"球队、教练、阵型与观察",description:"维护球队身份、教练任期、阵型概率与球队能力观察。",note:"本入口是球队月度工作簿，不是球队完整资料包。",modeId:"team-spreadsheet-import-mode",preview:"preview-team-import",details:"show-team-import-preview-json"},
    player:{title:"身份、履历、位置与状态",description:"维护球员身份、球队履历、位置、状态与能力观察。",note:"兼容球员月度和球员目录工作簿；球队月度文件不可在此导入。",modeId:"spreadsheet-import-mode",preview:"preview-player-import",details:"show-import-preview-json"},
    match:{title:"比赛、四个数据窗口与阵容版本",description:"维护比赛、阵容快照、首发替补与模型输入版本。",note:"比赛现有数据按所选比赛导出；AI分析包仍从比赛中心进入。",modeId:"match-import-mode",preview:"preview-match-import",details:"show-match-import-json"},
  }[section];
  const phase=batch?.phase??previewPhase(preview),busy=view?.busy??false,locked=busy||phase==="unknown";
  const mode=batch?.mode??preview?.import_mode??"add_and_update",ready=batch?.canCommit??phase==="ready";
  const count=Object.values(previews).filter(Boolean).length;
  const status=view?.exporting===section?"正在导出":(["unknown","writing","blocked","error","imported"].includes(phase)?importPhaseLabel[phase]:view?.exportResult?.category===section?view.exportResult.message:importPhaseLabel[phase]);
  return `<div class="workbooks-module-workspace"><section class="page-heading simple-heading entity-page-heading"><div><h1>数据导入与导出</h1><p>按类别维护工作簿；先导出、再预检，核对批次后确认提交。</p></div></section>
  <nav class="workbook-page-actions" aria-label="工作簿相关入口"><button class="secondary" data-action="toggle-workspace-pane" data-pane="inspector">${inspectorCollapsed?"导入规则":"关闭规则"}</button><button class="secondary" data-page="teams">球队目录</button><button class="secondary" data-page="players">球员目录</button><button class="secondary" data-page="lineups">比赛中心</button></nav>
  <section class="workspace-grid workbooks-workspace ${inspectorCollapsed?"inspector-collapsed":""}" data-module-sidebar-state="${moduleSidebarCollapsed?"legacy-collapsed":"fixed"}">
    <aside class="panel module-sidebar" data-workspace-panel="workbooks-sidebar"><header><h2>工作包目录</h2><p>按数据类型分别维护</p></header><nav class="workbook-category-nav workspace-section-nav" aria-label="工作包类别">${(["team","player","match"] as const).map(id=>`<button class="${section===id?"primary active":"secondary"}" data-action="select-workspace-section" data-section-id="${id}" aria-current="${section===id?"page":"false"}" ${busy?"disabled":""}>${labels[id]}</button>`).join("")}</nav><strong>已完成预检 ${count} / 3</strong><p>预检过不等于可以提交，也不等于已经写入。</p></aside>
    <main class="workspace-main"><div class="workbook-content-scroll" data-workspace-scroll-key="workbooks-${section}"><section class="workspace-module-view active" data-workspace-section="${section}">
      <section class="panel workbook-scope"><header><h2>${labels[section]} · ${copy.title}</h2><p>${copy.description}</p></header><p>${copy.note}</p></section>
      <section class="panel workbook-steps"><header><h2>01 导出 → 02 预检 → 03 确认</h2><p>最终提交前核对类别、文件、模式和返回的批次身份。</p></header><div class="workbook-flow">
        <article><h3>01　导出</h3><p>空白模板用于新建；现有数据用于维护。</p>${section==="match"?`<label class="field"><span>导出比赛</span><select id="exchange-match-id" ${locked?"disabled":""}><option value="">请选择比赛</option>${matches.map(m=>`<option value="${escapeHtml(m.id)}" ${m.id===selectedMatchId?"selected":""}>${escapeHtml(m.home_team_name+" vs "+m.away_team_name)}</option>`).join("")}</select></label>`:""}<button class="secondary" data-action="workbook-export-template" ${locked?"disabled":""}>空白模板</button><button class="secondary" data-action="workbook-export-data" ${locked?"disabled":""}>现有数据</button></article>
        <article><h3>02　预检</h3><label class="field"><span>导入模式</span><span class="workbook-mode-control"><select id="${copy.modeId}" ${locked?"disabled":""}><option value="add_and_update" ${mode==="add_and_update"?"selected":""}>新增并更新</option><option value="add_only" ${mode==="add_only"?"selected":""}>仅新增</option></select>${iconSlot("chevron-down",16)}</span></label><button class="primary" data-action="${copy.preview}" ${locked?"disabled":""}>选择文件并预检</button></article>
        <article><h3>03　确认提交</h3><p>提交按钮固定在底部。预检存在冲突或错误时不可写入。</p><button class="secondary" data-action="${copy.details}" ${preview?"":"disabled"}>查看批次明细</button><small>${escapeHtml(preview?.batch_id??batch?.receipt?.batch_id??"尚未预检")}</small></article>
      </div></section>
      ${workbookPreview(preview,phase,batch?.error??null,section)}
    </section></div>
    <footer class="workbook-commit-bar"><button class="primary" data-action="workbook-commit" ${ready&&!busy?"":"disabled"}>确认提交</button><button class="secondary" data-action="toggle-workspace-pane" data-pane="inspector">字段规则</button>${phase==="unknown"?`<button class="secondary" data-action="workbook-readback" ${busy?"disabled":""}>核查原批次</button>`:""}<span role="status">${status}</span></footer></main>
    <aside class="panel workspace-inspector" data-workspace-panel="workbooks-inspector">${workspacePaneToggle("inspector",inspectorCollapsed)}<h2>先预检，再提交</h2><ul class="inspector-list"><li>空白默认保留；clear 才显式清空，仍须验证。</li><li>冲突和错误分别处理，查看不等于解决。</li><li>类别、文件或模式变化后，旧确认失效。</li><li>重复与已导入批次不可再次提交。</li><li>结果未知时先核查原批次，不自动重发。</li></ul></aside>
  </section></div>`;
}
