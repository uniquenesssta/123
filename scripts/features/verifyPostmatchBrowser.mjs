import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {mountedFixture} from '../design-refresh/mountedFixture.mjs';
export async function verifyPostmatchBrowser(root,directory){
 const results=[],boundary=fs.readFileSync(new URL('./reviewBoundary.js',import.meta.url),'utf8')+'\n'+fs.readFileSync(new URL('./postmatchBoundary.js',import.meta.url),'utf8');
 await mountedFixture(root,directory,async({cdp,evaluate,screenshot,route})=>{
  const check=async(name,expression)=>{const ok=await evaluate(`(async()=>{const end=Date.now()+6000;while(!(${expression})){if(Date.now()>end)return false;await new Promise(r=>setTimeout(r,35));}return true;})()`);results.push({name,ok});fs.writeFileSync(path.join(directory,'results.json'),JSON.stringify(results,null,2));if(!ok){fs.writeFileSync(path.join(directory,'failure-body.txt'),await evaluate('document.body.innerText'));await screenshot('failed-'+results.length);}assert.ok(ok,name);};
  const click=s=>evaluate(`(async()=>{const node=document.querySelector(${JSON.stringify(s)});if(!node)throw Error('Missing '+${JSON.stringify(s)});node.focus();node.click();await new Promise(r=>setTimeout(r,180));})()`);
  const confirm=()=>click('[data-confirmation-action="confirm"]'),cancel=()=>click('[data-confirmation-action="cancel"]');
  const select=async(id,value)=>{await click('#'+id);await click('#'+id+'-options .ui-select-option[data-value="'+value+'"]');};
  const history=()=>evaluate('(()=>{document.querySelector("[data-analytics-step=history]").click();})()');
  await route('review');await select('review-match-id','a');await click('[data-action="load-review-match"]');await click('[data-review-step="8"]');
  await check('generated review alone does not authorize settlement','document.querySelector("[data-action=settle-postmatch-review]").disabled&&window.__pm.writes.length===0');
  await click('[data-action="inspect-postmatch-readiness"]');
  await check('blocked gate shows actual backend checks and reason','document.querySelector("[data-postmatch-panel=settlement]").textContent.includes("QA 缺少合法冻结快照")&&document.querySelector("[data-action=settle-postmatch-review]").disabled');await screenshot('settlement-blocked');
  await evaluate('window.__pm.ready=true');await click('[data-action="inspect-postmatch-readiness"]');
  await check('fresh ready gate unlocks the exact review action','!document.querySelector("[data-action=settle-postmatch-review]").disabled&&document.querySelectorAll("[data-postmatch-panel=settlement] tbody tr").length===8');
  await click('[data-action="settle-postmatch-review"]');
  await check('settlement confirmation names original review and formal run with neutral action','document.querySelector("[data-confirmation-action=confirm]").classList.contains("ui-button-primary")&&document.body.textContent.includes("review-a")&&document.body.textContent.includes("run-a")');await screenshot('settlement-confirm');await cancel();
  await check('cancel creates no settlement command','window.__pm.writes.length===0');
  await click('[data-action="settle-postmatch-review"]');await evaluate('window.__pm.ready=false');await confirm();
  await check('expired backend gate rejects without a write','window.__pm.writes.length===0&&document.body.textContent.includes("许可已变化")');await cancel();
  await evaluate('window.__pm.ready=true');await click('[data-action="inspect-postmatch-readiness"]');
  for(const [width,height]of [[1440,900],[960,720],[1280,720],[1920,1080]])for(const theme of ['light','dark']){
   await cdp.send('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:1,mobile:false});
   await evaluate(`(async()=>{if(document.documentElement.dataset.theme!==${JSON.stringify(theme)})document.querySelector('[data-action=toggle-theme]').click();const collapsed=document.querySelector('.secondary-sidebar').hidden;if(collapsed!==${width<1100})[...document.querySelectorAll('[data-action=toggle-global-sidebar]')].find(e=>!e.closest('[hidden],[inert]')).click();await new Promise(r=>setTimeout(r,100));})()`);
   await check(width+' '+theme+' settlement contained and independently scrollable','(()=>{const page=document.querySelector(".page-container"),v=document.querySelector(".review-workspace-viewport"),p=document.querySelector("[data-postmatch-panel=settlement]");return page.scrollWidth<=page.clientWidth+1&&v.getBoundingClientRect().bottom<=innerHeight+1&&p.getBoundingClientRect().right<=v.getBoundingClientRect().right+1;})()');
   await check(width+' '+theme+' settlement type and row centers','(()=>{const panel=document.querySelector("[data-postmatch-panel=settlement]"),h=getComputedStyle(panel.querySelector("h2"));return h.fontSize==="14px"&&h.fontWeight==="700"&&[...panel.querySelectorAll("tr")].every(r=>r.getBoundingClientRect().height>=36);})()');await screenshot('settlement-ready-'+width+'-'+theme);
  }
  await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false});
  await evaluate('window.__pm.lostSettlement=true');await click('[data-action="settle-postmatch-review"]');await confirm();
  await check('lost settlement reply retains unknown receipt and no duplicate request','window.__pm.writes.filter(r=>r.command==="settle_postmatch_review").length===1&&document.body.textContent.includes("结果待核实")');await cancel();
  await click('[data-action="recover-postmatch-settlement"]');
  await check('readback proves same settlement and never resends','window.__pm.writes.filter(r=>r.command==="settle_postmatch_review").length===1&&document.querySelector(".review-context").textContent.includes("正式结算：已完成")&&document.querySelector("[data-postmatch-panel=settlement]").textContent.includes("settlement-key-a")');await screenshot('settlement-recovered');
  await route('analytics');await history();
  await check('analytics reads real postmatch owner and does not imply H result','document.querySelector("[data-postmatch-panel=history]").textContent.includes("已有 1 条正式结算")&&document.querySelector("[data-postmatch-panel=history]").textContent.includes("尚无当前分区的监控读回")');
  await evaluate('window.__pm.failOverview=true');await click('[data-action="refresh-postmatch-overview"]');
  await check('overview failure retains data and announces failure','document.querySelector("[data-postmatch-panel=history]").textContent.includes("QA 赛后目录读取失败")&&document.querySelector("[data-postmatch-panel=history]").textContent.includes("已有 1 条正式结算")');await evaluate('window.__pm.failOverview=false');await click('[data-action="refresh-postmatch-overview"]');
  await click('[data-action="prepare-evidence-decision"][data-evidence-item-id="e1"]');
  await check('evidence choices are four distinct real actions','document.querySelectorAll("[data-action=postmatch-evidence-verdict]").length===4');await click('[data-action="postmatch-evidence-verdict"][data-verdict="not_verifiable"]');
  await check('evidence confirmation includes original settlement and editable note','document.querySelector("#evidence-decision-note")&&document.querySelector("#modal-root").textContent.includes("settlement-a")');
  await click('[data-action="submit-evidence-decision"]');
  await check('blank note submits no evidence command and preserves dialog','window.__pm.writes.filter(r=>r.command==="decide_evidence_scoring_item").length===0&&document.querySelector("#evidence-decision-note")');
  await click('#evidence-decision-note');await cdp.send('Input.insertText',{text:'已核对原始证据，当前结果无法独立验证'});
  await evaluate('window.__pm.lostEvidence=true');await click('[data-action="submit-evidence-decision"]');
  await check('unknown evidence decision remains bound to its original item','window.__pm.writes.filter(r=>r.command==="decide_evidence_scoring_item").length===1&&window.__pm.items[0].verdict==="not_verifiable"&&window.__pm.items[0].accuracy_score===null&&document.querySelector("#evidence-decision-note").value.includes("无法独立验证")&&document.querySelector("[data-postmatch-confirm]").disabled');await screenshot('evidence-unknown');await click('#modal-root [data-action="close-workspace-detail"]');
  await click('[data-action="recover-evidence-decision"][data-evidence-item-id="e1"]');
  await check('evidence recovery reads immutable record without a second score','window.__pm.writes.filter(r=>r.command==="decide_evidence_scoring_item").length===1&&!document.querySelector("[data-action=prepare-evidence-decision][data-evidence-item-id=e1]")');await click('#modal-root [data-action="close-workspace-detail"]');
  await select('postmatch-settlement-id','settlement-a');
  await check('settlement filter excludes other settlement evidence','!document.querySelector("[data-evidence-item-id=e2]")');
  await select('postmatch-settlement-id','__all__');
  await check('settlement filter returns to all without reloading the page','document.querySelector("[data-evidence-item-id=e2]")&&document.querySelector("#postmatch-settlement-id").textContent.includes("全部已载入结算")');
  await select('postmatch-competition-id','c1');
  await click('[data-action="refresh-postmatch-monitoring"]');
  await check('monitoring requires confirmation with exact target and sizes','document.querySelector("[data-confirmation-action=confirm]")&&document.body.textContent.includes("c1 / T-1h")&&document.body.textContent.includes("100 / 50")');await cancel();
  await check('monitor cancellation sends no command','window.__pm.writes.filter(r=>r.command==="refresh_postmatch_monitoring").length===0');
  await evaluate('window.__pm.waitMonitor=true');await click('[data-action="refresh-postmatch-monitoring"]');await confirm();
  await check('monitoring request is single and captured','window.__pm.releaseMonitor&&window.__pm.writes.filter(r=>r.command==="refresh_postmatch_monitoring").length===1');
  await evaluate('(()=>{const s=document.querySelector("select[data-postmatch-field=competition_id]");s.value="c2";s.dispatchEvent(new Event("change",{bubbles:true}));window.__pm.waitMonitor=false;window.__pm.releaseMonitor();})()');
  await check('delayed C1 report never appears under C2','!document.querySelector("[data-confirmation-action=confirm]")&&!document.querySelector(".postmatch-report")&&document.querySelector("#postmatch-competition-id").textContent.includes("QA 明确标识杯赛")');
  await click('[data-action="refresh-postmatch-monitoring"]');await confirm();
  await check('current partition reports insufficient actual samples instead of global stability','document.querySelector(".postmatch-report").textContent.includes("QA 分区 c2")&&document.querySelector(".postmatch-report").dataset.status==="insufficient"&&document.querySelector(".postmatch-report").textContent.includes("实际基线 0 / 当前 0")');
  for(const [width,height]of [[1440,900],[960,720],[1280,720],[1920,1080]])for(const theme of ['light','dark']){
   await cdp.send('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:1,mobile:false});
   await evaluate(`(async()=>{if(document.documentElement.dataset.theme!==${JSON.stringify(theme)})document.querySelector('[data-action=toggle-theme]').click();const collapsed=document.querySelector('.secondary-sidebar').hidden;if(collapsed!==${width<1100})[...document.querySelectorAll('[data-action=toggle-global-sidebar]')].find(e=>!e.closest('[hidden],[inert]')).click();await new Promise(r=>setTimeout(r,100));document.querySelector('[data-postmatch-panel=history]').scrollIntoView({block:'start'});})()`);
   await check(width+' '+theme+' H fields align and shrink with container','(()=>{const fields=[...document.querySelectorAll(".postmatch-fields .postmatch-control")].map(e=>e.getBoundingClientRect());const cols=getComputedStyle(document.querySelector(".postmatch-fields")).gridTemplateColumns.split(" ").length;return fields.every((r,i)=>i%cols===0||Math.abs((r.top+r.bottom)/2-(fields[i-1].top+fields[i-1].bottom)/2)<1)&&document.querySelector(".page-container").scrollWidth<=document.querySelector(".page-container").clientWidth+1;})()');
   await check(width+' '+theme+' H shared primitives are enhanced once','document.querySelectorAll(".postmatch-fields [role=combobox]").length===2&&document.querySelectorAll(".postmatch-fields [data-number-step]").length===4&&getComputedStyle(document.querySelector(".postmatch-fields")).gap==="8px"&&getComputedStyle(document.querySelector("[data-postmatch-panel=history]")).paddingTop==="12px"&&getComputedStyle(document.querySelector(".postmatch-panel-body")).gap==="8px"&&!document.querySelector(".postmatch-fields .searchable-select")');await check(width+' '+theme+' all postmatch controls and panels are reachable','(async()=>{for(const selector of ["#postmatch-current-size","#postmatch-settlement-id","[data-postmatch-panel=evidence] .postmatch-evidence:last-child button","[data-postmatch-panel=providers] h2"]){const e=document.querySelector(selector);if(!e)return false;e.scrollIntoView({block:"center"});await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));const r=e.getBoundingClientRect(),hit=document.elementFromPoint((r.left+r.right)/2,(r.top+r.bottom)/2);if(!hit||!(e===hit||e.contains(hit)))return false;}const details=document.querySelector("#analysis-history-step");return details.scrollHeight<=details.clientHeight+1&&document.querySelector(".topbar").getBoundingClientRect().top>=-1;})()');const wheel=await evaluate('(()=>{const s=document.querySelector(".analysis-module-workspace .workspace-module-view.active");s.scrollTop=0;const r=s.getBoundingClientRect();return{x:(r.left+r.right)/2,y:(r.top+r.bottom)/2};})()');await cdp.send('Input.dispatchMouseEvent',{type:'mouseWheel',...wheel,deltaX:0,deltaY:500});await check(width+' '+theme+' native wheel scrolls the stage without moving header','document.querySelector(".analysis-module-workspace .workspace-module-view.active").scrollTop>0&&document.querySelector(".topbar").getBoundingClientRect().top>=-1');await evaluate('document.querySelector("[data-postmatch-panel=history]").scrollIntoView({block:"start"})');await screenshot('h-monitor-'+width+'-'+theme);
  }
  await evaluate('(()=>{const input=document.querySelector("#postmatch-current-size");input.value="25";input.dispatchEvent(new Event("input",{bubbles:true}));})()');
  await check('window edit invalidates previous H result','!document.querySelector(".postmatch-report")&&document.querySelector("[data-postmatch-panel=history]").textContent.includes("尚无当前分区")');
  await evaluate('window.__pm.failMonitor=true');await click('[data-action="refresh-postmatch-monitoring"]');await confirm();
  await check('partial monitoring failure remains unknown','document.body.textContent.includes("QA 供应商评分后漂移刷新失败")&&document.querySelector("[data-action=refresh-postmatch-monitoring]").disabled');await cancel();await click('[data-action="recover-postmatch-monitoring"]');
  await check('readback never fakes all monitoring transactions complete','document.querySelector("[data-postmatch-panel=history]").textContent.includes("原请求继续保留")&&document.querySelector("[data-action=refresh-postmatch-monitoring]").disabled');await screenshot('h-monitor-unknown');
  await check('transport tests never report private model or real PG success','window.__pm.writes.filter(r=>r.command==="settle_postmatch_review").length===1&&window.__pm.writes.filter(r=>r.command==="decide_evidence_scoring_item").length===1&&window.__errors.length===0');
 },boundary);
 console.log('R14-12 mounted postmatch Windows browser checks passed: '+results.length);
}
