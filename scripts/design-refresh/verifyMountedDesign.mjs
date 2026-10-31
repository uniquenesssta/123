import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { mountedFixture } from "./mountedFixture.mjs";

/** Real main/Shell/Session/Workspace mount; only native transport uses labeled synthetic QA data. */
export async function verifyMountedDesign(root,directory) {
  const results=[];
  await mountedFixture(root,directory,async({cdp,evaluate,screenshot,route})=>{
    const check=async(name,expression)=>{const ok=await evaluate(`(async()=>{const probe=()=>(${expression});const end=Date.now()+2500;let ok=probe();while(!ok&&Date.now()<end){await new Promise(r=>setTimeout(r,30));ok=probe();}return Boolean(ok);})()`);results.push({name,ok});fs.writeFileSync(path.join(directory,"results.json"),JSON.stringify(results,null,2));if(!ok)await screenshot("assertion-failure");assert.ok(ok,name);};
    const navigationContrast=()=>{const luminance=color=>{const rgb=color.match(/[\d.]+/g).slice(0,3).map(Number).map(v=>{v/=255;return v<=.04045?v/12.92:((v+.055)/1.055)**2.4;});return rgb[0]*.2126+rgb[1]*.7152+rgb[2]*.0722;};return [...document.querySelectorAll('.workspace-task-anchor-nav button')].every(button=>{const fg=luminance(getComputedStyle(button.querySelector('strong')).color),bg=luminance(getComputedStyle(button).backgroundColor);return (Math.max(fg,bg)+.05)/(Math.min(fg,bg)+.05)>=4.5;});};
    const click=async selector=>evaluate(`(async()=>{const element=document.querySelector(${JSON.stringify(selector)});if(!element)throw Error("Missing "+${JSON.stringify(selector)});element.focus();element.click();await new Promise(r=>setTimeout(r,250));})()`);
    for(const [width,height] of [[1440,900],[960,720],[1280,720],[1920,1080]]) for(const theme of ["light","dark"]) {
      await cdp.send("Emulation.setDeviceMetricsOverride",{width,height,deviceScaleFactor:1,mobile:false});
      await evaluate(`(async()=>{if(document.documentElement.dataset.theme!==${JSON.stringify(theme)})document.querySelector('[data-action="toggle-theme"]').click();await new Promise(r=>setTimeout(r,100));const collapsed=document.querySelector('.secondary-sidebar').hidden;if(collapsed!==${width<1100})Array.from(document.querySelectorAll('[data-action="toggle-global-sidebar"]')).find(e=>!e.closest('[hidden],[inert]')).click();await new Promise(r=>setTimeout(r,100));})()`);
      for(const page of ["dashboard","teams","players","rules","database","architecture"]) {
        await route(page);
        if(["database","architecture"].includes(page))await click(`.workspace-task-anchor-nav [data-anchor-id="${page==='database'?'database-overview':'architecture-flow'}"]`);
        await check(`${page}-${width}-${theme}: viewport fit`, `(()=>{const p=document.querySelector('.page-container');return p.scrollWidth<=p.clientWidth+1&&document.documentElement.scrollWidth<=innerWidth+1;})()`);
        if(page==="dashboard") {
          await check(`dashboard-${width}-${theme}: approved primary heights`, `(()=>{const c=[...document.querySelectorAll('.dashboard-action')];return c.length===5&&c.slice(0,2).every(e=>e.getBoundingClientRect().height>=132)&&getComputedStyle(c[0].querySelector('strong')).fontSize==='18px';})()`);
        }
        if(page==="teams")await check(`teams-${width}-${theme}: directory first`, `!!document.querySelector('.team-directory-only')&&!document.querySelector('.team-detail-workspace')`);
        if(page==="players")await check(`players-${width}-${theme}: real rows and fields`, `(()=>{const q=s=>document.querySelector(s),b=q('.player-browser'),t=q('.player-table-wrap');return q('.player-directory-table').rows.length===19&&q('#player-filter-team')&&q('#player-filter-position')&&q('#player-filter-availability')&&q('#player-filter-status')&&t.clientHeight>100&&q('.player-directory-table').scrollWidth<=t.clientWidth+1&&q('.entity-main-footer').getBoundingClientRect().bottom<=b.getBoundingClientRect().bottom+1;})()`);
        if(page==="rules")await check(`rules-${width}-${theme}: only results scroll`, `(()=>{const q=s=>document.querySelector(s),s=q('.rules-table-scroll'),left=q('.rules-scope-list'),top=left.getBoundingClientRect().top;s.scrollTop=150;return s.scrollTop>0&&left.getBoundingClientRect().top===top&&getComputedStyle(left).overflowY!=='auto'&&getComputedStyle(q('.rules-region-list')).overflowY!=='auto';})()`);
        if(page==="database")await check(`database-${width}-${theme}: isolated safe overview`, `document.querySelectorAll('[data-navigation-panel]:not([hidden])').length===1&&document.querySelectorAll('.database-metrics .ui-stat-card').length===4&&document.querySelector('#database-danger').hidden&&document.querySelector('#database-connection').hidden`);
        if(page==="architecture")await check(`architecture-${width}-${theme}: runtime boundary`, `document.querySelectorAll('.architecture-flow > article').length===4&&document.querySelector('.architecture-capability').textContent.includes('外部运行时未捆绑')`);
        await screenshot(`${page}-${width}-${theme}`);
      }
    }
    await cdp.send("Emulation.setDeviceMetricsOverride",{width:1440,height:900,deviceScaleFactor:1,mobile:false});
    await route("teams");
    await evaluate(`document.querySelector('#team-filter-country').value='KR'`);
    await click('[data-action="search-teams"]');await click('.team-select-checkbox');await click('.entity-directory-open');
    await check("team explicit open loads real roster",`document.querySelectorAll('.roster-table tbody tr').length===6`);
    await screenshot("team-selected");
    await click('[data-action="toggle-workspace-pane"][data-pane="inspector"]');
    await check("team inspector owns a separate track",`(()=>{const m=document.querySelector('.team-detail-workspace > main').getBoundingClientRect(),i=document.querySelector('.entity-inspector').getBoundingClientRect();return i.left>=m.right&&i.right<=innerWidth+1;})()`);
    await click('.roster-table [data-action="preview-player-from-team"]');
    await check("roster quick view retains full profile route",`!!document.querySelector('.entity-inspector [data-action="open-player-profile-from-team"]')`);
    await click('.entity-inspector [data-action="open-player-profile-from-team"]');
    await check("team to player full profile",`document.querySelector('.application-shell').dataset.currentPage==='players'&&!!document.querySelector('[data-action="return-to-source-team-profile"]')`);
    await click('[data-action="return-to-source-team-profile"]');
    await check("return preserves source team",`document.querySelector('.application-shell').dataset.currentPage==='teams'&&document.querySelector('h1').textContent.includes('首尔')`);
    await route("teams");await click('.core-local-navigation [data-action="select-workspace-section"][data-section-id="directory"]');
    await check("team return retains filter selection and original card focus",`document.activeElement?.id==='team-directory-'+window.__fixtures.teams[0].id&&document.querySelector('#team-filter-country').value==='KR'&&document.querySelector('.team-select-checkbox').checked&&window.__calls.some(c=>c.command==='list_teams'&&c.args.query.country_code==='KR')`);
    await route("players");
    await evaluate(`document.querySelector('#player-filter-team').value=window.__fixtures.teams[0].id`);
    await click('[data-action="search-players"]');await click('[data-action="next-player-page"]');
    await click('.player-select-checkbox');await click('.player-directory-table [data-action="open-player"]');
    await click('.entity-inspector [data-action="select-workspace-section"][data-section-id="profile"]');
    await click('[data-action="select-workspace-section"][data-section-id="directory"]');
    await check("player return retains filter page selection and object focus",`document.querySelector('#player-filter-team').value===window.__fixtures.teams[0].id&&document.querySelector('[data-pagination-current]').textContent.includes('2')&&document.querySelector('.player-select-checkbox').checked&&document.activeElement?.id==='player-directory-fixture-player-0'`);
    for(const page of ["teams","players"]) {
      await route(page);
      if(page==="teams")await click('.core-local-navigation [data-section-id="directory"]');
      await click(page==="teams"?'.entity-directory-open':'.player-directory-table [data-action="open-player"]');
      await cdp.send("Emulation.setDeviceMetricsOverride",{width:960,height:720,deviceScaleFactor:1,mobile:false});
      for(const collapsed of [true,false]) {
        await evaluate(`(async()=>{if(document.querySelector('.secondary-sidebar').hidden!==${collapsed})[...document.querySelectorAll('[data-action="toggle-global-sidebar"]')].find(e=>!e.closest('[hidden],[inert]')).click();await new Promise(r=>setTimeout(r,120));const b=document.querySelector('.${page==='teams'?'team-detail-workspace':'player-browser'}');if(b.classList.contains('inspector-collapsed'))document.querySelector('[data-action="toggle-workspace-pane"][data-pane="inspector"]').click();await new Promise(r=>setTimeout(r,150));})()`);
        await check(`${page}: narrow inspector visible with sidebar ${collapsed}`,`(()=>{const m=document.querySelector('.entity-main').getBoundingClientRect(),i=document.querySelector('.entity-inspector').getBoundingClientRect(),c=document.querySelector('.entity-inspector-close').getBoundingClientRect();return m.width>=400&&i.width===300&&i.right<=innerWidth+1&&c.width>0&&c.bottom<=innerHeight+1;})()`);
        await screenshot(`${page}-inspector-960-${collapsed?'collapsed':'expanded'}`);
        await click('.entity-inspector-close');
        await check(`${page}: all columns accessible after closing narrow inspector`, `(()=>{const t=document.querySelector('.entity-main table'),s=t.closest('.entity-table-wrap');return t.scrollWidth<=s.clientWidth+1&&document.querySelector('.entity-main').getBoundingClientRect().width>=400;})()`);
      }
      await cdp.send("Emulation.setDeviceMetricsOverride",{width:1440,height:900,deviceScaleFactor:1,mobile:false});
    }
    await route("database");await click('.workspace-task-anchor-nav [data-anchor-id="database-connection"]');
    await evaluate(`document.querySelector('#database-url').value='postgres://draft:transient@localhost/draft';document.querySelector('#max-connections').value='17'`);
    await click('.workspace-task-anchor-nav [data-anchor-id="database-statistics"]');
    await check("statistics panel switches in place",`!document.querySelector('#database-statistics').hidden&&document.querySelector('#database-connection').hidden`);
    await click('.workspace-task-anchor-nav [data-anchor-id="database-connection"]');
    await check("section switch retains transient connection draft",`document.querySelector('#database-url').value.includes('draft:transient')&&document.querySelector('#max-connections').value==='17'`);
    await check("credentials never enter workspace writes",`window.__calls.filter(c=>c.command==='save_workspace_state').every(c=>!JSON.stringify(c.args).includes('draft:transient'))`);
    await click('.workspace-task-anchor-nav [data-anchor-id="database-danger"]');
    await check("database navigation remains readable after switching",`(${navigationContrast.toString()})()`);
    await screenshot("database-danger");await click('[data-action="request-reset-database"]');
    await check("reset remains typed and disabled initially",`!!document.querySelector('#database-reset-confirmation')&&document.querySelector('#database-reset-confirm-button').disabled`);
    await evaluate(`const input=document.querySelector('#database-reset-confirmation');input.value='wrong';input.dispatchEvent(new Event('input',{bubbles:true}));`);
    await check("wrong database name cannot reset",`document.querySelector('#database-reset-confirm-button').disabled&&window.__calls.every(c=>c.command!=='reset_database')`);
    await cdp.send("Input.dispatchKeyEvent",{type:"keyDown",key:"Escape",code:"Escape",windowsVirtualKeyCode:27});await cdp.send("Input.dispatchKeyEvent",{type:"keyUp",key:"Escape",code:"Escape",windowsVirtualKeyCode:27});
    await route("architecture");await click('.workspace-task-anchor-nav [data-anchor-id="architecture-modules"]');
    await check("modules start collapsed",`!document.querySelector('#architecture-modules details').open&&document.querySelector('#architecture-flow').hidden`);
    await click('#architecture-modules summary');
    await check("module explanation expands",`document.querySelector('#architecture-modules details').open`);
    await check("architecture navigation remains readable after switching",`(${navigationContrast.toString()})()`);
    await screenshot("architecture-modules");
    await route("rules");
    await evaluate(`window.__summary.data.competitions=Array.from({length:20},(_,i)=>({...window.__fixtures.state.data.competitions[0],id:'density-'+i,code:'D'+i,name:'很长的具体赛事名称与中文说明 '+i,metadata:{menu_region:'地区足联长名称 '+i}}))`);
    await click('[data-action="refresh-service-state"]');
    await cdp.send("Emulation.setDeviceMetricsOverride",{width:960,height:720,deviceScaleFactor:1,mobile:false});
    await evaluate(`window.dispatchEvent(new Event('resize'))`);
    await check("dense regions remain fixed and every choice stays accessible",`(()=>{const s=document.querySelector('[data-rules-region-select]'),r=document.querySelector('.rules-region-list');return !s.hidden&&s.options.length===21&&getComputedStyle(r).overflowY!=='auto'&&s.getBoundingClientRect().bottom<=r.getBoundingClientRect().bottom;})()`);
    await evaluate(`const s=document.querySelector('[data-rules-region-select]');s.value='地区足联长名称 19';s.dispatchEvent(new Event('change',{bubbles:true}));`);
    await check("last dense region filters the real catalogue",`document.querySelector('#rules-visible-count').textContent==='1'&&document.querySelector('[data-rules-competition-row]:not([hidden])').textContent.includes('19')`);
    await screenshot("rules-dense-960-expanded");
    await cdp.send("Emulation.setDeviceMetricsOverride",{width:1440,height:900,deviceScaleFactor:1,mobile:false});
    await route("database");await click('.workspace-task-anchor-nav [data-anchor-id="database-connection"]');
    await evaluate(`document.querySelector('#database-url').value='postgres://qa:secret@localhost/rejected';document.querySelector('#max-connections').value='17';window.__rejectConnection=true`);
    await click('[data-action="connect-database"]');
    await check("failed real connection flow clears secret and keeps safe controls",`document.querySelector('#database-url').value===''&&document.querySelector('#max-connections').value==='17'&&window.__calls.some(c=>c.command==='configure_database')`);
    await evaluate(`window.__summary.data.stats=null`);await click('[data-action="refresh-service-state"]');await click('.workspace-task-anchor-nav [data-anchor-id="database-overview"]');
    await check("unknown statistics remain unknown",`[...document.querySelectorAll('.database-metrics .ui-stat-value')].slice(0,3).every(e=>e.textContent==='—')`);
    await evaluate(`window.__summary.data.database_configured=false;window.__summary.data.database_health=null`);await click('[data-action="refresh-service-state"]');await click('.workspace-task-anchor-nav [data-anchor-id="database-danger"]');
    await check("unconfigured service cannot reset",`document.querySelector('[data-action="request-reset-database"]').disabled&&document.querySelector('.database-panel:not([hidden])').textContent.includes('等待连接')`);
    await check("full mount has no uncaught errors",`window.__errors.length===0`);
    await screenshot("database-unconfigured");
    fs.writeFileSync(path.join(directory,"results.json"),JSON.stringify(results,null,2));
  });
  console.log(`Mounted approved design PASS: ${results.length} checks; ${directory}`);
  return results;
}
