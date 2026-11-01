import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {mountedFixture} from "../design-refresh/mountedFixture.mjs";
export async function verifyResearchBrowser(root,directory){
 const boundary=["predictionBoundary.js","researchBoundary.js"].map(p=>fs.readFileSync(new URL(p,import.meta.url),"utf8")).join("\n"),results=[];
 await mountedFixture(root,directory,async({cdp,evaluate,screenshot,route})=>{
  const check=async(name,expr)=>{const ok=await evaluate("(async()=>{const end=Date.now()+5000;while(!("+expr+")){if(Date.now()>end)return false;await new Promise(r=>setTimeout(r,30));}return true;})()");results.push({name,ok});fs.writeFileSync(path.join(directory,"results.json"),JSON.stringify(results,null,2));if(!ok)await screenshot("failure-"+results.length);assert.ok(ok,name);};
  const click=async selector=>evaluate("(async()=>{const e=document.querySelector("+JSON.stringify(selector)+");if(!e)throw Error('Missing '+ "+JSON.stringify(selector)+");e.focus();e.click();await new Promise(r=>setTimeout(r,150));})()");
  const change=async(id,value)=>evaluate("{const e=document.getElementById("+JSON.stringify(id)+");e.focus();e.value="+JSON.stringify(value)+";e.dispatchEvent(new Event('change',{bubbles:true}));}");
  const taskButton=i=>'[data-action="open-p4-task"][data-task-id="layout-match-research-'+i+'"]';
  await route("prediction");await click('[data-action="select-workspace-section"][data-section-id="p4"]');
  await check("real application mounts research owner and task", "document.querySelector('[data-research-task-id]')?.dataset.researchTaskId==='layout-match-research-0'");
  for(const [width,height] of [[1440,900],[960,720],[1280,720],[1920,1080]])for(const theme of ["light","dark"]){
   await cdp.send("Emulation.setDeviceMetricsOverride",{width,height,deviceScaleFactor:1,mobile:false});
   await evaluate("(async()=>{if(document.documentElement.dataset.theme!=="+JSON.stringify(theme)+")document.querySelector('[data-action=\\\"toggle-theme\\\"]').click();const hidden=document.querySelector('.secondary-sidebar').hidden;if(hidden!=="+(width<1100)+")[...document.querySelectorAll('[data-action=\\\"toggle-global-sidebar\\\"]')].find(e=>!e.closest('[hidden],[inert]')).click();await document.fonts.ready;})()");
   await check(width+" "+theme+" geometry, equal metrics and single viewport scroll", "(()=>{const vp=document.querySelector('.prediction-viewport'),r=vp.getBoundingClientRect(),m=[...document.querySelectorAll('.p4-metric')].map(e=>e.getBoundingClientRect()),f=[...document.querySelectorAll('.p4-plan-fields .field')].map(e=>e.getBoundingClientRect());return r.bottom<=innerHeight+1&&vp.scrollHeight>vp.clientHeight&&document.documentElement.scrollWidth<=innerWidth+1&&m.length===4&&Math.max(...m.map(r=>r.width))-Math.min(...m.map(r=>r.width))<1&&Math.abs(f[0].y-f[1].y)<1;})()");
   await evaluate("document.querySelector('.prediction-viewport').scrollTop=0");await screenshot("research-"+width+"-"+theme);
  }
  await check("real zero preserved as source value","document.querySelector('.p4-evidence-choice span').textContent==='0'");
  await click('#p4-all-source-evidence > summary');await check("all evidence remains available","document.querySelector('#p4-all-source-evidence').open&&document.querySelectorAll('#p4-all-source-evidence tbody tr').length===2");
  await click(taskButton(1));await check("verified frozen snapshot is readonly","document.querySelector('[data-research-task-state]')?.dataset.researchTaskState==='FROZEN'&&!!document.querySelector('[data-action=\\\"show-p4-snapshot\\\"]')&&[...document.querySelectorAll('.p4-conflict-card button')].every(b=>b.disabled)");
  await click('[data-action="show-p4-snapshot"]');await check("actual snapshot opens shared detail history","document.querySelector('.workspace-detail-page')?.textContent.includes('P4 冻结快照')");
  await click('[data-action="close-workspace-detail"]');
  await click(taskButton(2));await check("provider unavailable stays a failed research event","document.querySelector('[data-research-task-state]')?.dataset.researchTaskState==='FAILED'&&document.querySelector('.p4-events').textContent.includes('ModelProvider unavailable')&&!document.querySelector('[data-action=\\\"show-p4-snapshot\\\"]')");
  await click(taskButton(0));
  for(const state of ["RESEARCH_QUEUED","FREEZING","BLOCKED"]){
   await evaluate("(async()=>{const id='layout-match-research-0',w=await window.__TAURI_INTERNALS__.invoke('read_p4_task_workspace',{taskId:id});w.task.state="+JSON.stringify(state)+";w.task.data_cutoff_at='2000-01-01T00:00:00Z';window.__research.rows[id]=w;})()");
   await click('[data-action="refresh-p4-workbench"]');await check(state+" cannot manufacture freeze permission","document.querySelector('[data-research-task-state]')?.dataset.researchTaskState==="+JSON.stringify(state)+"&&!document.querySelector('[data-action=\\\"show-p4-snapshot\\\"]')&&[...document.querySelectorAll('.p4-conflict-card button')].every(b=>b.disabled)");
  }
  await evaluate("delete window.__research.rows['layout-match-research-0']");await click('[data-action="refresh-p4-workbench"]');
  await click('.p4-evidence-choice input[value="qa-evidence-0"]');
  await evaluate("document.querySelector('[id^=\\\"p4-conflict-note-\\\"]').value='Actual decision note';document.querySelector('[id^=\\\"p4-conflict-note-\\\"]').dispatchEvent(new Event('input',{bubbles:true}))");
  await click('[data-action="resolve-p4-conflict-select"]');await check("decision first opens protective confirmation","!!document.querySelector('dialog[open]')&&window.__research.writes===0&&document.querySelector('dialog[open]').textContent.includes('layout-match-research-0')");
  await evaluate("document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}))");
  await check("cancel writes nothing","window.__research.writes===0&&!document.querySelector('dialog[open]')");
  await click('[data-action="resolve-p4-conflict-select"]');
  await check("confirmation provides actual submit action","!!document.querySelector('dialog[open] [data-confirmation-action=\\\"confirm\\\"]')");
  await click('dialog[open] [data-confirmation-action="confirm"]');
  await check("decision records actual note without inferring frozen","window.__research.writes===1&&document.querySelector('.p4-conflict-card').textContent.includes('Actual decision note')&&document.querySelector('[data-research-task-state]').dataset.researchTaskState==='RESEARCH_PARTIAL'&&!document.querySelector('[data-action=\\\"show-p4-snapshot\\\"]')");
  await change("p4-workbench-match-id","layout-match-b");await check("match switch replaces task identity","document.querySelector('[data-research-task-id]')?.dataset.researchTaskId==='layout-match-b-research-0'");
  await evaluate("window.__research.failAfterWrite=true");await click('[data-action="resolve-p4-conflict-unknown"]');await click('dialog[open] [data-confirmation-action="confirm"]');
  await check("lost response preserves recoverable original operation","window.__research.writes===2&&!!document.querySelector('.p4-recovery')&&document.querySelector('[data-action=\\\"resolve-p4-conflict-unknown\\\"]').disabled");
  await evaluate("document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}))");await click('[data-action="recover-p4-research"]');
  await check("readback finds immutable unknown decision without another write","window.__research.writes===2&&!document.querySelector('.p4-recovery')&&document.querySelector('.p4-conflict-card').textContent.includes('已接受未知')");
  await click('[data-action="plan-p4-horizons"]');await check("planning verifies three actual returned windows","window.__research.plans===1&&!document.querySelector('.p4-recovery')&&document.querySelectorAll('.p4-horizon-grid button').length===3");
  await check("no uncaught production browser exception","window.__errors.length===0");await screenshot("research-final");
 },boundary);
 console.log("S03 research mounted Windows browser checks passed: "+results.length);return results;
}
