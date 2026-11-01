import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {mountedFixture} from '../design-refresh/mountedFixture.mjs';

export async function verifyRunsBrowser(root,directory){
 const boundary=fs.readFileSync(new URL('./runsBoundary.js',import.meta.url),'utf8'),results=[];
 await mountedFixture(root,directory,async({cdp,evaluate,screenshot,route})=>{
  const check=async(name,expr)=>{const ok=await evaluate('(async()=>{const end=Date.now()+5000;while(!('+expr+')){if(Date.now()>end)return false;await new Promise(r=>setTimeout(r,30));}return true;})()');results.push({name,ok});fs.writeFileSync(path.join(directory,'results.json'),JSON.stringify(results,null,2));if(!ok){fs.writeFileSync(path.join(directory,'failure-body.txt'),await evaluate('document.body.innerText'));await screenshot('failure-'+results.length);}assert.ok(ok,name);};
  const click=async selector=>evaluate('(async()=>{const e=document.querySelector('+JSON.stringify(selector)+');if(!e)throw Error("Missing control");e.focus();e.click();await new Promise(r=>setTimeout(r,250));})()');
  const open=id=>click('[data-action="open-run"][data-run-id="'+id+'"]');
  await evaluate('window.__runs.listPending=true');
  const navigation=route('runs');
  await new Promise(r=>setTimeout(r,700));
  await check('first loading is unknown rather than a false zero','document.querySelector(".runs-toolbar")?.textContent.includes("—")&&document.querySelector(".runs-viewport")?.textContent.includes("首次读取")');
  await evaluate('window.__runs.releaseList()');await navigation;
  await check('all six samples use real production list rendering','document.querySelectorAll(".runs-history-table tbody tr").length===6');
  for(const [width,height] of [[1440,900],[960,720],[1280,720],[1920,1080]])for(const theme of ['light','dark']){
   await cdp.send('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:1,mobile:false});
   await evaluate('(async()=>{if(document.documentElement.dataset.theme!=='+JSON.stringify(theme)+')document.querySelector("[data-action=toggle-theme]").click();const hidden=document.querySelector(".secondary-sidebar").hidden;if(hidden!=='+(width<1100)+')[...document.querySelectorAll("[data-action=toggle-global-sidebar]")].find(e=>!e.closest("[hidden],[inert]")).click();await new Promise(r=>setTimeout(r,150));})()');
   await check(width+' '+theme+' seven aligned columns stay in viewport','(()=>{const p=document.querySelector(".page-container"),t=document.querySelector(".runs-history-table"),rows=[...t.tBodies[0].rows];return p.scrollWidth<=p.clientWidth+1&&t.scrollWidth<=t.clientWidth+1&&rows.every(r=>r.cells.length===7)&&rows.every(r=>[...r.cells].every((c,i)=>Math.abs(c.getBoundingClientRect().left-t.tHead.rows[0].cells[i].getBoundingClientRect().left)<2));})()');
   await check(width+' '+theme+' typography and row controls preserve Figma metrics','(()=>{const h=getComputedStyle(document.querySelector(".runs-heading h1")),row=document.querySelector(".runs-history-table tbody tr"),b=row.querySelector("button");return h.fontSize==="26px"&&h.fontWeight==="700"&&row.getBoundingClientRect().height>='+ (width<1100?112:88)+'&&Math.abs(b.getBoundingClientRect().height-32)<1;})()');
   await check(width+' '+theme+' header stays fixed while only body scrolls','(()=>{const v=document.querySelector(".runs-viewport"),h=document.querySelector(".runs-heading").getBoundingClientRect().top;v.scrollTop=10000;return Math.abs(document.querySelector(".runs-heading").getBoundingClientRect().top-h)<1&&v.getBoundingClientRect().bottom<=innerHeight+1&&document.documentElement.scrollHeight<=innerHeight+1;})()');
   await check(width+' '+theme+' table and footnote remain inside their own layout bounds','(()=>{const t=document.querySelector(".runs-history-table").getBoundingClientRect(),p=document.querySelector(".run-history-panel").getBoundingClientRect(),f=document.querySelector(".runs-footnote").getBoundingClientRect();return t.bottom<=p.bottom-11&&f.top>=p.bottom+11;})()');await evaluate('document.querySelector(".runs-viewport").scrollTop=0');await screenshot('runs-'+width+'-'+theme);
  }
  await check('missing matrix/model/rule and actual zero remain distinct','(()=>{const c=document.querySelector("tr[data-run-id=run-c]"),d=document.querySelector("tr[data-run-id=run-d]"),e=document.querySelector("tr[data-run-id=run-e]");return c.textContent.includes("0%")&&c.textContent.includes("暂无矩阵")&&d.textContent.includes("— / — / —")&&e.textContent.includes("模型未记录")&&e.textContent.includes("规则未记录")&&!e.textContent.includes("P4");})()');
  await evaluate('window.__runs.listPending=true');await click('[data-action="refresh-runs"]');
  await check('refresh preserves rows and blocks duplicate refresh','document.querySelectorAll(".runs-history-table tbody tr").length===6&&document.querySelector("[data-action=refresh-runs]").disabled');
  await evaluate('window.__runs.listFail=true;window.__runs.releaseList()');
  await check('refresh error keeps prior six records','document.querySelector(".runs-viewport").textContent.includes("保留原记录")&&document.querySelectorAll(".runs-history-table tbody tr").length===6');await screenshot('runs-refresh-error');
  await evaluate('window.__runs.listFail=false');await click('[data-action="refresh-runs"]');
  await evaluate('window.__runs.detailPending="run-a"');await open('run-a');
  await check('detail loading clears previous payload','document.querySelector(".runs-viewport").textContent.includes("读取")&&!document.querySelector(".runs-facts")');
  await click('[data-action="runs-back"]');await evaluate('window.__runs.detailFail="run-b"');await open('run-b');await evaluate('window.__runs.releaseDetail()');
  await check('late A cannot replace failed B or show previous payload','document.querySelector(".runs-toolbar").textContent.includes("run-b")&&!!document.querySelector("[data-action=runs-retry-detail]")&&!document.querySelector(".runs-facts")');
  await evaluate('window.__runs.detailFail=null');await click('[data-action="runs-retry-detail"]');
  await check('retry reads exact B and shows four saved metrics','document.querySelector(".runs-identity").textContent.includes("QA主队 b")&&document.querySelectorAll(".runs-metrics .ui-stat-card").length===4');
  for(const width of [1440,960]){await cdp.send('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});await check('detail '+width+' responsive aligned metrics','getComputedStyle(document.querySelector(".runs-metrics")).gridTemplateColumns.split(" ").length==='+ (width<1100?2:4));await screenshot('runs-detail-'+width);}
  await click('[data-action="runs-technical"]');
  await check('complete stored matrix and original fingerprints remain accessible','document.querySelector(".workspace-detail-page.prediction-result-modal")?.textContent.includes("20.00%")&&document.querySelector(".workspace-detail-page.prediction-result-modal")?.textContent.includes("a".repeat(64))');
  await click('[data-action="close-workspace-detail"]');await click('[data-action="runs-back"]');
  await check('return restores focus to original list record','document.activeElement.id==="run-open-run-b"');
  await click('[data-action="request-hide-run-history"][data-run-id="run-a"]');
  await check('confirmation binds actual run and retention boundary','document.querySelector("dialog[open]").textContent.includes("run-a")&&document.querySelector("dialog[open]").textContent.includes("保留")');
  await click('[data-confirmation-action="cancel"]');
  await check('cancel sends zero history mutations','window.__runs.writes.length===0&&document.querySelectorAll(".runs-history-table tbody tr").length===6');
  await click('[data-action="request-hide-run-history"][data-run-id="run-a"]');await click('[data-confirmation-action="confirm"]');
  await check('successful hide reads original object and only removes list visibility','window.__runs.writes.join() ==="run-a"&&!document.querySelector("tr[data-run-id=run-a]")&&window.__runs.docs["run-a"].input.source==="explicit-QA-boundary"&&window.__runs.docs["run-a"].output.scorelines.length===1&&window.__runs.reads.filter(id=>id==="run-a").length>=3');
  await evaluate('window.__runs.hideFail=true');await click('[data-action="request-hide-run-history"][data-run-id="run-b"]');await click('[data-confirmation-action="confirm"]');
  await check('lost response keeps receipt and disables blind replay','document.querySelector(".runs-viewport").textContent.includes("结果待核实")&&document.querySelector("tr[data-run-id=run-b] [data-action=request-hide-run-history]").disabled');await screenshot('runs-hide-unknown');
  await click('[data-action="refresh-runs"]');
  await check('refresh verifies original hidden object without another write','window.__runs.writes.join() ==="run-a,run-b"&&!document.querySelector("tr[data-run-id=run-b]")&&!document.querySelector(".runs-viewport").textContent.includes("结果待核实")');
  await route('prediction');await click('[data-action="select-workspace-section"][data-section-id="history"]');await open('run-c');
  await check('compact prediction history opens the exact run through public feature route','document.querySelector(".application-shell").dataset.currentPage==="runs"&&document.querySelector(".runs-toolbar").textContent.includes("run-c")&&document.querySelector(".runs-metrics").textContent.includes("0%")');
  await route('prediction');await click('[data-action="select-workspace-section"][data-section-id="history"]');await evaluate('window.__runs.hideFail=false');
  await click('[data-action="request-hide-run-history"][data-run-id="run-c"]');await click('[data-confirmation-action="confirm"]');
  await check('compact prediction history updates its sole Session list after hide','!document.querySelector("tr[data-run-id=run-c]")&&window.__runs.writes.join()==="run-a,run-b,run-c"');
  await evaluate('window.__runs.hideFail=true');await click('[data-action="request-hide-run-history"][data-run-id="run-d"]');await click('[data-confirmation-action="confirm"]');
  await check('compact history preserves unknown response and blocks replay','document.querySelector(".prediction-history-panel").textContent.includes("结果待核实")&&document.querySelector("tr[data-run-id=run-d] [data-action=request-hide-run-history]").disabled');
  await click('[data-action="refresh-runs"]');
  await check('compact refresh recovers exact original ID without replay','!document.querySelector("tr[data-run-id=run-d]")&&!document.querySelector(".prediction-history-panel").textContent.includes("结果待核实")&&window.__runs.writes.join()==="run-a,run-b,run-c,run-d"');
  await check('history never grants new model execution','!window.__calls.some(c=>["execute_prediction_from_match","execute_shadow_prediction_from_match"].includes(c.command))');
  await check('no uncaught browser exceptions','window.__errors.length===0');
 },boundary);
 console.log('S05 mounted Windows browser checks passed: '+results.length);return results;
}
