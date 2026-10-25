import { dataTableView, mountDataTable, type TableRow, type TableColumn } from "../data-table";
import { keyValueListView } from "../key-value-list";
import { statCardView } from "../stat-card";
import { statusBadgeView } from "../status-badge";
import { timelineView, type TimelineItem } from "../timeline";
import { progressView } from "../progress";
import { emptyStateView } from "../empty-state";
import { codeBlockView } from "../code-block";
import { databasePage } from "../../../pages/database";
import { logsPage } from "../../../pages/logs";
import { matchEventTimelineView } from "../../../pages/review/index";
import { ModalController } from "../../../app/modal";
import type { BootstrapResponse, IssueLogEntry, MatchReviewDetail } from "../../../types";

export function installDisplayFixture() {
  const root=document.querySelector<HTMLElement>("#display-root")!,checks:string[]=[],sorts:string[]=[],selections:string[]=[];
  const expect=(value:unknown,name:string)=>{if(!value)throw new Error(name);checks.push(name);};
  const columns:TableColumn[]=[{key:"name",label:"球员",width:180},{key:"score",label:"能力",type:"Number",sortable:true,sort:"ascending",width:100},{key:"status",label:"状态",width:100},{key:"action",label:"操作",type:"Action",width:120}];
  const rows:TableRow[]=[{id:"a",selected:true,cells:{name:{kind:"metadata",text:"Alex Morgan",supporting:"USA · 34"},score:{kind:"number",text:"87.4",supporting:"92%"},status:{kind:"status",status:{label:"可用",tone:"Success"}},action:{kind:"action",button:{label:"完整档案",attributes:{"data-record":"a"}}}}},{id:"b",cells:{name:"Marta <&>",score:91.2,status:{kind:"status",status:{label:"有争议",tone:"Warning"}},action:{kind:"action",button:{label:"完整档案"}}}},{id:"c",disabled:true,cells:{name:"禁用记录",score:null,status:"待处理",action:{kind:"action",button:{label:"不可操作"}}}}];
  const items:TimelineItem[]=[];for(const verification of ["Verified","Unverified","Disputed"] as const)for(const revision of ["Original","Corrected","Cancelled"] as const)items.push({id:verification+revision,time:"90+3′",title:revision==="Cancelled"?"已取消的原始事件":"事件 "+verification,description:"原始事实仍保留 <&>，长文字可以换行。",metadata:"主队 · 序号 "+items.length,verification,revision,...(items.length===0?{score:"0–0"}:{})});
  const original=JSON.stringify({rows,items});
  root.innerHTML=`<div class="display-gallery"><section id="tables"><h2>数据表格</h2><div id="loaded">${dataTableView({label:"实际记录",columns,rows,selection:true})}</div><div id="busy">${dataTableView({label:"加载中的记录",columns,rows,selection:true,busy:true})}</div><div id="empty-table">${dataTableView({label:"空表格",columns,rows:[],emptyTitle:"没有匹配记录",emptyDescription:"请调整筛选条件"})}</div></section><section><h2>指标与事实</h2><div class="display-stats">${statCardView({label:"球员",value:"1,248",note:"12 个待审核能力候选"})}${statCardView({label:"零值",value:0})}</div>${keyValueListView([{label:"对象",value:"皇家社会 <&>"},{label:"保留",value:"所有原始记录"}])}${statusBadgeView({label:"需要检查",tone:"Warning",style:"Outline"})}<div id="progress-zero">${progressView({label:"尚未开始",value:0})}</div><div id="progress-value">${progressView({label:"已知进度",value:37.5,size:"Default"})}</div><div id="progress-unknown">${progressView({label:"未知进度"})}</div>${emptyStateView({title:"暂无内容",description:"加载完成后仍没有可显示的数据。"})}${emptyStateView({title:"没有匹配记录",size:"Default",action:{label:"重置筛选",attributes:{id:"empty-action"}}})}${codeBlockView({label:"技术日志",text:("long unbroken "+"x".repeat(120)+"\n").repeat(40)+"</code><script>window.injected=true</script>"})}</section><section id="timeline-section"><h2>核验与修订独立</h2>${timelineView({label:"事件时间线",items})}</section><section id="production"><h2>生产数据投影</h2><div id="db-output"></div><div id="logs-output"></div><div id="review-output"></div></section></div><main class="main-content"><div class="page-container"></div><div id="modal-root"></div></main><button id="after-display">后续操作</button>`;
  const q=<T extends HTMLElement=HTMLElement>(selector:string):T=>{const n=root.querySelector<T>(selector);if(!n)throw Error(selector);return n;};
  const table=q("#loaded .ui-data-table"),busy=q("#busy .ui-data-table");
  const handle=mountDataTable(table,{sort:(key,direction)=>sorts.push(key+":"+direction),select:(id,checked)=>selections.push(id+":"+checked),selectAll:checked=>selections.push("all:"+checked)}),busyHandle=mountDataTable(busy,{sort:()=>sorts.push("unexpected busy sort"),select:()=>selections.push("unexpected busy selection")});
  function structure(){
    expect(JSON.stringify({rows,items})===original,"display does not mutate source records or ordering");
    expect(table.querySelectorAll("thead th").length===5&&table.querySelectorAll("tbody tr").length===3,"native table structure");
    expect(table.querySelector("caption")?.textContent==="实际记录","table has accessible caption");
    expect(q<HTMLInputElement>("#loaded [data-table-select-all]").indeterminate,"select-all indeterminate native projection");
    expect(Math.abs(q("#loaded thead tr").getBoundingClientRect().height-34)<0.1,"Figma table header 34px");
    expect(Math.abs(q("#loaded tbody tr").getBoundingClientRect().height-36)<0.1,"Figma table row 36px with metadata/button/badge");
    expect(getComputedStyle(q("#loaded td[data-align=Number]")).textAlign==="right","number cell aligns right");
    expect(getComputedStyle(q('#loaded tr[data-table-row="b"] td[data-align=Number]')).fontWeight==="600","plain numeric cell uses Figma semibold typography");
    expect(getComputedStyle(q("#loaded .ui-table-cell-copy > small")).fontWeight==="400","supporting metadata remains regular weight");
    expect(getComputedStyle(q("#loaded .ui-table-cell-copy > span")).color===getComputedStyle(q(".ui-stat-value")).color,"primary cell text uses Figma strong semantic color");
    const icon=q("#loaded .ui-table-sort-icon .ui-icon-slot").getBoundingClientRect();expect(icon.width===12&&icon.height===12,"exact 12px sort icon geometry");
    expect(q("#loaded [aria-sort]").getAttribute("aria-sort")==="ascending","sort state maps to aria-sort");
    expect(busy.getAttribute("aria-busy")==="true"&&busy.querySelectorAll("tbody tr").length===3,"busy retains existing rows");
    expect(Array.from(busy.querySelectorAll<HTMLInputElement|HTMLButtonElement>("input,button")).every(c=>c.disabled),"busy disables all row, select and sort controls");
    expect(q<HTMLTableCellElement>("#empty-table td").colSpan===4,"empty body spans all columns");
    expect(q("#empty-table td").getBoundingClientRect().height===140,"Figma table empty body 140px");
    expect(getComputedStyle(q(".ui-stat-value")).fontSize==="20px"&&getComputedStyle(q(".ui-stat-value")).lineHeight==="24px","Figma metric 20/24");
    expect(q(".display-stats").textContent?.includes("零值0"),"zero metric is preserved");
    expect(q(".ui-key-value-list dt").getBoundingClientRect().width===100,"Figma key-value label 100px");
    expect(q(".ui-key-value-list dd").textContent==="皇家社会 <&>","native fact text escaped");
    expect(q("#progress-zero [role=progressbar]").getAttribute("aria-valuenow")==="0","zero progress is determinate");
    expect(q("#progress-value [role=progressbar]").getAttribute("aria-valuenow")==="37.5","fractional progress retained");
    expect(q("#progress-zero .ui-progress").getBoundingClientRect().height===2&&q("#progress-value .ui-progress").getBoundingClientRect().height===4,"Figma progress 2/4px");
    expect(!q("#progress-unknown [role=progressbar]").hasAttribute("aria-valuenow"),"unknown progress never fabricates a percentage");
    const fill=q("#progress-value .ui-progress-fill").getBoundingClientRect(),track=q("#progress-value .ui-progress").getBoundingClientRect();expect(Math.abs(fill.width/track.width-.375)<.001,"progress exact rendered fraction");
    expect(q(".ui-empty-state").getBoundingClientRect().height>=54,"Figma Compact empty minimum54px");
    expect(getComputedStyle(q(".ui-code-block")).fontSize==="11px"&&getComputedStyle(q(".ui-code-block")).lineHeight==="17px","Figma technical code11/17");
    expect(q(".ui-code-block").textContent!.includes("<script>")&&!q(".ui-code-block").querySelector("script"),"technical text escaped without losing content");
    expect(q(".ui-code-block").scrollHeight>q(".ui-code-block").clientHeight,"technical text owns scroll viewport");
    expect(document.documentElement.scrollWidth<=innerWidth,"no horizontal viewport overflow");
    expect(q(".ui-timeline").querySelectorAll(".ui-timeline-item").length===9,"all9 verification/revision combinations retained");
    expect(q(".ui-timeline").querySelectorAll(".ui-timeline-connector").length===8,"last event has no trailing connector");
    expect(q(".ui-timeline-time").getBoundingClientRect().width===64,"Figma timestamp column64px");
    for(const row of q(".ui-timeline").querySelectorAll<HTMLElement>(".ui-timeline-item")){
      expect(row.querySelectorAll(".ui-status-badge").length===(row.dataset.revision==="Original"?1:2),"independent axes "+row.dataset.eventKey);
      expect(getComputedStyle(row).opacity==="1"&&!row.hasAttribute("aria-disabled"),"cancelled/unverified remains readable "+row.dataset.eventKey);
    }
    expect(q('[data-revision="Cancelled"] .ui-timeline-heading > strong').textContent?.includes("原始事件")&&getComputedStyle(q('[data-revision="Cancelled"] .ui-timeline-heading > strong')).textDecorationLine.includes("line-through"),"cancelled title marks revision and keeps original text");
    expect(q(".ui-timeline").querySelectorAll(".ui-timeline-score").length===1&&q(".ui-timeline-score").textContent==="0–0","missing score is absent; explicit zero score retained");
  }
  function production(){
    // Isolated input records exercise actual page/view bindings, not live database access.
    const state={config_path:"C:/configuration.json",data:{database_health:{database_name:"actual-fixture-db",server_version:"PostgreSQL fixture",database_size_bytes:1024,migration_count:43,latency_ms:7},stats:{players:731,matches:11,active_lineups:4,model_runs:17,rule_packages:5,large_counts_are_estimates:true,teams:41,competitions:6,data_providers:2,availability_records:9,ability_observations:23,route_bindings:8,pending_ability_updates:3}}} as unknown as BootstrapResponse;
    q("#db-output").innerHTML=databasePage(state);const db=q("#db-output");expect(db.querySelectorAll(".ui-stat-card").length===3&&db.textContent!.includes("约 731"),"production database uses actual supplied counts and estimate prefix");
    expect(db.querySelectorAll(".ui-data-table tbody tr").length===12&&db.querySelector(".ui-data-table")?.textContent?.includes("23"),"all12 real database statistics projected through Data Table");
    const entry={id:"e",severity:"error",source:"frontend",app_version:"0.23.0",user_message:"调用失败",technical_message:"技术行1\n<script>unsafe</script>",occurrence_count:3,first_seen_at:"2026-10-26T00:00:00Z",last_seen_at:"2026-10-26T01:00:00Z",operations:["bootstrap"]} as IssueLogEntry;
    q("#logs-output").innerHTML=logsPage([entry]);expect(q("#logs-output .ui-code-block").textContent===entry.technical_message,"production log block preserves actual multiline technical message");expect(q("#logs-output").querySelectorAll(".ui-stat-card").length===4,"production log metrics use shared cards");
    q("#logs-output").innerHTML=logsPage([]);expect(q("#logs-output .ui-empty-state").textContent!.includes("目前没有记录到问题"),"production empty logs use shared Empty State");
    const detail={event_summary:{effective_count:42,verified_count:2,total_count:3,cancelled_count:1,disputed_count:1,last_event_minute:93,latest_home_score:null,latest_away_score:null},events:[{id:"original",event_key:"original",event_type:"goal",minute:90,stoppage_minute:3,description:"原记录",player_name:"甲",related_player_name:null,team_name:"主队",confidence:.8,sequence_no:1,home_score:null,away_score:null,verification_status:"disputed",revision_status:"cancelled"},{id:"audit",event_key:"audit",event_type:"var",minute:67,stoppage_minute:null,description:"历史审计",team_name:null,confidence:.5,sequence_no:2,home_score:0,away_score:0,verification_status:"unverified",revision_status:"superseded"}]} as unknown as MatchReviewDetail;
    q("#review-output").innerHTML=matchEventTimelineView(detail);const view=q("#review-output");expect(view.textContent!.includes("42 条有效")&&view.textContent!.includes("未逐事件记录"),"host summary retained without recalculating event counts or missing score");expect(view.textContent!.includes("有争议")&&view.textContent!.includes("已取消")&&view.textContent!.includes("已被替代")&&view.textContent!.includes("待核验"),"production adapter preserves independent and audit-only statuses");expect(view.querySelectorAll(".ui-timeline-item").length===2&&view.querySelectorAll(".ui-timeline-score").length===1,"production adapter retains cancelled/audit entries and missing-score semantics");
    const modal=new ModalController();modal.show("真实详情",{name:"保留文本",probability:.75,technical_id:"trace <&>"});expect(q("#modal-root .ui-key-value-list").textContent!.includes("75.00%"),"production detail keeps existing probability formatting");expect(q("#modal-root .technical-details").textContent!.includes("trace <&>"),"technical detail remains collapsed and escaped");modal.close();
    // Keep production branches compact in the screenshot; their data assertions ran above.
    q("#db-output").replaceChildren();q("#logs-output").replaceChildren();
  }
  function contracts(){
    for(const call of [()=>dataTableView({label:"",columns,rows}),()=>dataTableView({label:"x",columns:[columns[0],columns[0]],rows}),()=>dataTableView({label:"x",columns,rows:[rows[0],rows[0]]}),()=>progressView({label:"x",value:NaN}),()=>timelineView({label:"x",items:[items[0],items[0]]})]){let rejected=false;try{call()}catch{rejected=true}expect(rejected,"invalid public display contract rejected");}
    expect(progressView({label:"bound",value:-1}).includes('aria-valuenow="0"')&&progressView({label:"bound",value:101}).includes('aria-valuenow="100"'),"out-of-range percentage visually clamps without altering host record");
    const before=sorts.length;q<HTMLButtonElement>("#busy .ui-table-sort").dispatchEvent(new MouseEvent("click",{bubbles:true}));expect(sorts.length===before,"synthetic busy click cannot bypass gesture gate");
  }
  function cleanup(){const before=sorts.length;handle.destroy();handle.destroy();busyHandle.destroy();q<HTMLButtonElement>("#loaded .ui-table-sort").click();expect(sorts.length===before,"destroy removes sorting listeners");const again=mountDataTable(table,{sort:(key,d)=>sorts.push(key+":"+d)});q<HTMLButtonElement>("#loaded .ui-table-sort").click();expect(sorts.length===before+1,"remount binds exactly once");again.destroy();}
  return {q,checks,expect,structure,production,contracts,cleanup,sorts,selections};
}
