import assert from "node:assert/strict";
import { writeFileSync, mkdtempSync } from "node:fs";
import { resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";
import { productionFixture } from "./productionFixture.mjs";
import { captureHtmlScreenshot, resolveChromiumExecutable } from "../task-ui-screenshot-tools.mjs";

const root=resolve(fileURLToPath(new URL("../..",import.meta.url)));
const directory=mkdtempSync(join(tmpdir(),"football-approved-design-"));
const {css,script}=productionFixture(root),htmlPath=join(directory,"production-views.html");
writeFileSync(htmlPath,`<!doctype html><html lang="zh-CN" data-theme="light"><head><meta charset="utf-8"><style>${css}</style></head><body><script>${script.replace(/<\/script/gi,"<\\/script")}</script></body></html>`);
const results=[],browser=resolveChromiumExecutable();assert.ok(browser,"Windows Chromium required");
const allScreens=["dashboard","teams","lineups","rules","profile","team-open","team-closed"];
const screenArgument=process.argv.find(argument=>argument.startsWith('--screens='));
const screens=screenArgument?[...new Set(screenArgument.slice('--screens='.length).split(','))]:allScreens;
assert.ok(screens.length&&screens.every(screen=>allScreens.includes(screen)),"Select known production screens for a focused rerun");
for(const [width,height] of [[1440,900],[960,720],[1280,720],[1920,1080]]) for(const theme of ["light","dark"]) for(const screen of screens) {
  const name=`${screen}-${width}-${theme}`;
  const result=await captureHtmlScreenshot({browser,htmlPath,outputPath:join(directory,`${name}.png`),width,height,async exercise(cdp){
    const response=await cdp.send("Runtime.evaluate",{expression:`(async()=>{try{renderFixture(${JSON.stringify(screen)},${JSON.stringify(theme)},${width<1100});await document.fonts.ready;await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));return (${checks.toString()})(${JSON.stringify(screen)});}catch(e){return {failures:[e.stack],assertions:0};}})()`,awaitPromise:true,returnByValue:true});
    if(response.exceptionDetails) return {failures:[response.exceptionDetails.text],assertions:0};
    return response.result.value;
  }});
  results.push({name,...result});console.log(`${name}: ${result.assertions} checks; ${result.failures.length} failures`);
}
writeFileSync(join(directory,"results.json"),JSON.stringify(results,null,2));
const failures=results.flatMap(r=>r.failures.map(f=>`${r.name}: ${f}`));
console.log(`Approved design integration evidence: ${directory}`);
assert.equal(failures.length,0,failures.join("\n"));
console.log(`Approved design integration PASS: ${results.length} views, ${results.reduce((n,r)=>n+r.assertions,0)} checks`);

function checks(screen) {
  const failures=[];let assertions=0;
  const check=(ok,message)=>{assertions++;if(!ok)failures.push(message);};
  const q=s=>document.querySelector(s),rect=e=>e.getBoundingClientRect(),style=e=>getComputedStyle(e),near=(a,b)=>Math.abs(a-b)<1;
  check(document.fonts.check('500 13px "Noto Sans SC"'),"Noto Sans SC loaded");
  check([...document.fonts].every(f=>f.status!=="error"),"all bundled fonts are valid");
  check(near(rect(q(".primary-rail")).width,60)&&near(rect(q(".topbar")).height,44),"shell geometry 60 / 44");
  check(rect(q(".page-container")).right<=innerWidth+1,"workspace fits viewport");
  check(q(".page-container").scrollWidth<=q(".page-container").clientWidth+1,"no horizontal page overflow");
  const title=q("h1");check(style(title).fontWeight==="700", "page title Bold 700");
  const action=q(".core-local-navigation button");if(action)check(near(rect(action).height,32),"page navigation uses 32px controls");
  if(screen==="dashboard") {
    const cards=[...document.querySelectorAll(".dashboard-action")];check(cards.length===5,"all five real routes retained");
    check(near(rect(cards[0]).top,rect(cards[1]).top)&&near(rect(cards[2]).top,rect(cards[4]).top),"two primary and three secondary actions");
    check(style(cards[1]).backgroundImage.includes("gradient"),"prediction has approved emphasis");
    check(q(".compact-metrics").textContent.includes("约 712"),"actual supplied count and estimate marker preserved");
    check(style(q(".dashboard-metric:nth-child(2) strong")).fontSize==="28px","numeric metrics use approved 28px hierarchy");
  }
  if(screen==="teams") {
    const cards=[...document.querySelectorAll(".team-catalogue-results .entity-directory-item")];
    check(cards.length===4&&near(rect(cards[0]).top,rect(cards[1]).top)&&rect(cards[2]).top>rect(cards[0]).top,"directory is a two-column fluid grid");
    check(near(rect(q(".team-catalogue-layout > aside")).width,248),"fixed 248px filters");
    const filters=[...document.querySelectorAll(".team-catalogue-layout .entity-filter-field")];
    check(style(q(".entity-directory-copy > strong")).fontSize==="14px","directory title uses section hierarchy");
    check(style(q(".entity-filter-field > span")).fontSize==="12px","filter labels use label hierarchy");
    check(filters.every(f=>rect(f).width>200)&&rect(filters[1]).top>rect(filters[0]).bottom,"filters keep one readable vertical column");
    check(rect(q(".team-catalogue-layout")).height>innerHeight/2,"catalogue fills available viewport height");
    const footer=rect(q(".team-catalogue-results footer")),prev=rect(q('[data-action="previous-team-page"]')),next=rect(q('[data-action="next-team-page"]'));
    check(near((prev.left+next.right)/2,(footer.left+footer.right)/2),"pagination group centered");
    const box=rect(q(".entity-select-check")),open=rect(q(".entity-directory-open"));check(near(box.top+box.height/2,open.top+open.height/2),"selection vertically centered with identity");
  }
  if(screen==="rules") {
    const left=q(".rules-scope-list"),scroller=q(".rules-table-scroll"),scopeTop=rect(left).top;
    check(near(rect(left).width,138)||near(rect(left).width,130),"scope rail width is bounded");
    check(scroller.scrollHeight>scroller.clientHeight&&scroller.clientHeight>100,"competition results own the scroll");
    scroller.scrollTop=200;check(scroller.scrollTop>0&&near(rect(left).top,scopeTop),"results scroll without moving filters");
    const cards=[...document.querySelectorAll(".rules-competition-card")];check(near(rect(cards[0]).top,rect(cards[1]).top)&&near(rect(cards[0]).width,rect(cards[1]).width),"competition cards in equal two-column grid");
    check(cards.every(c=>c.dataset.search&&c.querySelector('[data-action="show-competition-path"]')&&c.querySelector('[data-action="request-delete-competition"]')),"data, rule path and guarded delete bindings retained");
    const search=q("#rules-competition-search");search.value="示例赛事 30";filterRulesCompetitionCatalogue();
    check(cards.filter(c=>!c.hidden).length===1&&q("#rules-visible-count").textContent==="1","actual production filter updates cards and count");
    search.value="不存在的比赛";filterRulesCompetitionCatalogue();check(!q("#rules-empty-filter").hidden&&cards.every(c=>c.hidden),"empty filter state and count stay synchronized");
    const kind=q('#rules-competition-kind'),kindControl=kind.closest('.searchable-select');
    kindControl.querySelector('.searchable-select-toggle').click();kindControl.querySelector('[data-value="league"]').click();q('.rules-clear').focus();
    check(kind.value==='league'&&kindControl.querySelector('input').value===kind.selectedOptions[0].label,"actual enhanced kind selection updates its native value and display");
    clearRulesCatalogueFilters(filterRulesCompetitionCatalogue);check(cards.every(c=>!c.hidden)&&search.value==="","actual clear action restores original catalogue");
    check(kind.value===''&&kindControl.querySelector('input').value===kind.options[0].label,"clear synchronizes the enhanced select's displayed value");scroller.scrollTop=0;
  }
  if(screen==="lineups") {
    const sides=[...document.querySelectorAll(".paired-lineup-side.balanced")];check(near(rect(sides[0]).width,rect(sides[1]).width)&&near(rect(sides[0]).top,rect(sides[1]).top),"home and away equal aligned columns");
    check(sides.every(s=>s.scrollWidth<=s.clientWidth+1),"runtime controls do not expand the lineup side's intrinsic grid width");
    for(const row of document.querySelectorAll(".balanced-lineup-row")) {
      check(near(rect(row).height,44),"roster row 44px");
      const controls=[...row.children].filter(e=>e.matches("select:not(.searchable-select-native),button,.searchable-select"));const center=rect(row).top+rect(row).height/2;
      check(controls.every(e=>near(rect(e).top+rect(e).height/2,center)),"roster controls share a horizontal center");
      check(controls.every(e=>rect(e).right<=rect(row).right+1),"roster controls fit row");
    }
    check(q('[data-lineup-field="expected_minutes"][type="hidden"]')&&q('[data-action="create-lineup-pair"]'),"advanced values and actual paired submission retained");
    const position=q('.balanced-lineup-position').closest('.searchable-select');check(position.querySelector('input').value==='GK',"production searchable position control displays its short native label");
    check([...document.querySelectorAll('.balanced-lineup-row .searchable-select-control')].every(e=>near(rect(e).height,40)),"enhanced roster select controls retain the approved 40px height");
    q('.balanced-lineup-row').scrollIntoView({block:'center'});
  }
  if(screen==="profile") {
    const menu=q(".team-profile-directory");check(near(rect(menu).width,248),"profile menu 248px");
    check(rect(menu).height<innerHeight-150,"profile menu hugs content");
    const buttons=[...menu.querySelectorAll("button")];check(buttons.every(b=>near(rect(b).height,32)&&near(rect(b).width,224)),"profile menu controls equal-width 32px rows");
    check(q('#team-localized-name')&&q('[data-action="update-team"]')&&q('#team-profile-presets'),"identity, aliases and historical workflows retained");
  }
  if(screen.startsWith("team-")) {
    const main=q(".team-detail-workspace > .entity-main"),layout=q(".team-detail-workspace");
    check(style(layout).gridTemplateColumns.split(" ").length===(screen==="team-open"?3:2),"inspector removes its grid track on close");
    check(rect(main).width>200,"team center retains useful width");
    check(rect(main).height>innerHeight/2,"team detail fills available viewport height");
    const groups=q(".team-detail-filter .entity-filter-groups"),summary=q(".team-detail-filter .entity-active-filters");
    check(rect(summary).top>=rect(groups).bottom,"detail filter summary follows fields without overlap");
    check(q(".roster-table").scrollWidth<=q(".entity-table-wrap").clientWidth+1,"roster table adapts to center column");
    check(q('.roster-table').querySelectorAll('tr[data-player-id]').length===6,"roster layout exercises populated bilingual and long-name rows");
    check([...q('.roster-table').querySelectorAll('td :is(button,.position-chip,.status-chip)')].every(e=>rect(e).right<=rect(e.closest('td')).right+1),"populated roster actions and statuses stay within their cells");
    check(style(q('.roster-table .entity-table-person strong')).fontSize==='12px'&&style(q('.roster-table .entity-table-person small')).fontSize==='11px',"roster identity and secondary names use the approved readable hierarchy");
    if(screen==="team-open") { const inspector=rect(q(".entity-inspector"));check(inspector.left>=rect(main).right&&inspector.right<=innerWidth+1,"inspector occupies its own fluid column without covering center"); }
  }
  return {failures,assertions};
}
