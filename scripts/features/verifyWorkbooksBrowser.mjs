import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { mountedFixture } from "../design-refresh/mountedFixture.mjs";
export async function verifyWorkbooksBrowser(root,directory) {
 const results=[];
 const boundary=fs.readFileSync(new URL("./workbooksBoundary.js",import.meta.url),"utf8");
 await mountedFixture(root,directory,async({cdp,evaluate,screenshot,route})=>{
  const check=async(name,expression)=>{
   const ok=await evaluate("(async()=>{const until=Date.now()+4000;while(!("+expression+")){if(Date.now()>until)return false;await new Promise(r=>setTimeout(r,40));}return true;})()");
   results.push({name,ok});fs.writeFileSync(path.join(directory,"results.json"),JSON.stringify(results,null,2));if(!ok)await screenshot("failure-"+results.length);assert.ok(ok,name);
  };
  const click=async selector=>evaluate("(async()=>{const element=document.querySelector("+JSON.stringify(selector)+");if(!element)throw Error('Missing '+ "+JSON.stringify(selector)+");element.focus();element.click();await new Promise(r=>setTimeout(r,250));})()");
  const category=async id=>click('.workbook-category-nav [data-section-id="'+id+'"]');
  const preview=async(id,kind="ready",file="a.xlsx")=>{
   await category(id);await evaluate("Object.assign(window.__workbooks,"+JSON.stringify({kind,file})+")");
   await click('[data-action="preview-'+id+'-import"]');await check(id+" preview settled "+kind,"!document.querySelector('.page-container[inert]')&&document.querySelector('.workbook-state-message')?.dataset.importPhase!=='choosing'&&document.querySelector('.workbook-state-message')?.dataset.importPhase!=='previewing'");
  };
  const confirm='[data-confirmation-action="confirm"]';
  await route("workbooks");
  await check("direct workbooks entry loads managed match choices", "window.__calls.some(c=>c.command==='player_catalog_reference_data')");
  for(const [width,height] of [[1440,900],[960,720],[1280,720],[1920,1080]])for(const theme of ["light","dark"]){
   await cdp.send("Emulation.setDeviceMetricsOverride",{width,height,deviceScaleFactor:1,mobile:false});
   await evaluate("(async()=>{if(document.documentElement.dataset.theme!=="+JSON.stringify(theme)+")document.querySelector('[data-action=\"toggle-theme\"]').click();const collapsed=document.querySelector('.secondary-sidebar').hidden;if(collapsed!=="+(width<1100)+")[...document.querySelectorAll('[data-action=\"toggle-global-sidebar\"]')].find(e=>!e.closest('[hidden],[inert]')).click();await new Promise(r=>setTimeout(r,120));})()");
   for(const id of ["team","player","match"]){
    await category(id);
    await check(id+" "+width+" "+theme+" fixed categories and reachable footer","(()=>{const q=s=>document.querySelector(s),p=q('.page-container'),a=q('.module-sidebar').getBoundingClientRect(),b=q('.workbook-commit-bar').getBoundingClientRect(),sc=q('.workbook-content-scroll');const top=a.top;sc.scrollTop=150;return a.width===220&&p.scrollWidth<=p.clientWidth+1&&b.bottom<=innerHeight+1&&b.height>=40&&q('.module-sidebar').getBoundingClientRect().top===top&&!q('.workbook-content-scroll .workbook-commit-bar');})()");
   }
   await evaluate("document.querySelector('.workbook-content-scroll').scrollTop=0");
   await screenshot("workbooks-"+width+"-"+theme);
  }
  await cdp.send("Emulation.setDeviceMetricsOverride",{width:1440,height:900,deviceScaleFactor:1,mobile:false});
  await preview("team");
  await check("ready shows actual batch rows and pinned commit", "document.querySelectorAll('.workbook-table tbody tr').length===12&&!document.querySelector('[data-action=\"workbook-commit\"]').disabled");
  await check("approved computed typography and exact icon/table geometry","(()=>{const q=s=>document.querySelector(s),h=getComputedStyle(q('.workbook-scope h2')),c=getComputedStyle(q('.workbook-counts')),n=getComputedStyle(q('.workbook-preview .field-note')),i=q('.workbook-mode-control .ui-icon-slot').getBoundingClientRect();return h.fontSize==='14px'&&h.fontWeight==='700'&&c.fontSize==='14px'&&c.fontWeight==='700'&&n.fontSize==='11px'&&n.lineHeight==='16px'&&i.width===16&&i.height===16&&q('.workbook-table th').getBoundingClientRect().height===36&&q('.workbook-table td').getBoundingClientRect().height===56;})()");
  await screenshot("team-ready");
  await category("player");
  await check("player category cannot commit team ticket","document.querySelector('[data-action=\"workbook-commit\"]').disabled&&document.querySelector('[data-import-phase]').dataset.importPhase==='idle'");
  await category("team");
  await check("return preserves exact team batch","document.querySelector('.workbook-preview').textContent.includes('team-a.xlsx')");
  await click('[data-action="workbook-commit"]');
  await check("final confirmation contains file hash mode and exact batch","document.querySelector('dialog[open]')?.textContent.includes('team-a.xlsx')&&document.body.textContent.includes('a'.repeat(64))");
  await evaluate("document.querySelector('[data-confirmation-action=\"cancel\"]')?.click()");
  await check("cancel confirmation does not write","window.__workbooks.writes===0");
  await click('[data-action="workbook-commit"]');await click(confirm);
  await check("successful commit shows imported with consumed eligibility","window.__workbooks.writes===1&&document.querySelector('[data-import-phase]').dataset.importPhase==='imported'&&document.querySelector('[data-action=\"workbook-commit\"]').disabled");
  await preview("player","none");
  await check("no changes cannot submit","document.querySelector('[data-import-phase]').dataset.importPhase==='no_changes'&&document.querySelector('[data-action=\"workbook-commit\"]').disabled");
  await preview("player","conflict");
  await check("conflict exposes actual candidate row","!!document.querySelector('[data-action=\"resolve-import-conflict\"][data-row-id=\"player-a.xlsx-0\"]')");
  await click('[data-action="resolve-import-conflict"]');
  await check("decision uses original row and candidate","window.__calls.some(c=>c.command==='resolve_player_catalog_import_conflict'&&c.args.resolution.row_id==='player-a.xlsx-0'&&c.args.resolution.selected_entity_id==='candidate')");
  await evaluate("const e=document.querySelector('#spreadsheet-import-mode');e.focus();e.value='add_only';e.dispatchEvent(new Event('change',{bubbles:true}))");
  await check("mode change invalidates old player confirmation","document.querySelector('[data-action=\"workbook-commit\"]').disabled");
  await check("mode change retains exact mode and keyboard focus","document.activeElement?.id===\'spreadsheet-import-mode\'&&document.activeElement.value===\'add_only\'");
  await preview("match","error");
  await check("error stays blocked","document.querySelector('[data-import-phase]').dataset.importPhase==='blocked'&&document.querySelector('[data-action=\"workbook-commit\"]').disabled");
  await preview("match","ready");
  await click('[data-action="workbook-export-template"]');await click(confirm);
  await check("match template export reaches current native command","window.__calls.some(c=>c.command==='export_match_lineup_template')");

  await evaluate("{const e=document.querySelector('#exchange-match-id');e.value='layout-match';e.dispatchEvent(new Event('change',{bubbles:true}));window.__workbooks.save=null}");
  await click('[data-action="workbook-export-data"]');
  await check("match data confirmation contains exact selected ID and teams","document.querySelector('dialog[open]')?.textContent.includes('layout-match')&&document.querySelector('dialog[open]')?.textContent.includes('首尔 FC')");
  await click(confirm);
  await check("cancelled native save does not export match data","!window.__calls.some(c=>c.command==='export_match_lineup_data')");
  await check("cancelled save preserves selected match","document.querySelector('#exchange-match-id').value==='layout-match'");
  await category("team");await category("match");
  await check("category return preserves selected match","document.querySelector('#exchange-match-id').value==='layout-match'");
  await evaluate("window.__workbooks.save=undefined");
  await click('[data-action="workbook-export-data"]');await click(confirm);
  await check("selected match data reaches actual match domain API","window.__calls.some(c=>c.command==='export_match_lineup_data'&&c.args.matchId==='layout-match')");
  await evaluate("Object.assign(window.__workbooks,{timeout:true,applied:true})");
  await click('[data-action="workbook-commit"]');await click(confirm);
  await check("unknown state outranks earlier export success in footer","document.querySelector(\'.workbook-commit-bar\').textContent.includes(\'结果未核实\')");
  await check("interruption preserves unknown and blocks resubmit","document.querySelector('[data-import-phase]').dataset.importPhase==='unknown'&&document.querySelector('[data-action=\"workbook-commit\"]').disabled");
  await evaluate("document.querySelector('[data-confirmation-action=\"cancel\"]')?.click()");
  await route("dashboard");await route("workbooks");await category("match");
  await click('[data-action="workbook-readback"]');
  await check("reopen reads original batch with no duplicate write","window.__workbooks.writes===2&&document.querySelector('[data-import-phase]').dataset.importPhase==='imported'&&window.__calls.some(c=>c.command==='read_match_lineup_import_preview'&&c.args.batchId==='match-a.xlsx')");
  await category("team");await click('[data-action="workbook-export-template"]');
  await check("export is still only confirmation","window.__workbooks.exports===2");
  await click(confirm);
  await check("template export reaches exact native API","window.__calls.some(c=>c.command==='export_team_monthly_template')&&window.__workbooks.exports===3");
  await check("no uncaught browser exceptions","window.__errors.length===0");
  await screenshot("workbooks-final");
 },boundary);
 console.log("S09 mounted Windows browser checks passed: "+results.length);return results;
}
