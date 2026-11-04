import {waitForMountedApplication,reportMountedFailure} from "./mountedReadiness.mjs";
import fs from "node:fs";
import path from "node:path";
import {startMountedApplication} from "./mountedApplication.mjs";

import { captureHtmlScreenshot, resolveChromiumExecutable } from "../task-ui-screenshot-tools.mjs";
import { loadTeamsProduction } from "../features/loadTeamsProduction.mjs";

export function mountedBoundarySource(root,extraBoundary="") {
  const fixtures=loadTeamsProduction(root)("scripts/design-refresh/fixtureData.ts");
  const base=fs.readFileSync(path.join(root,"scripts/features/lineups-browser/boundary.js"),"utf8");
  const boundary=fs.readFileSync(new URL("./mountedBoundary.js",import.meta.url),"utf8");
  return "window.__fixtures="+JSON.stringify(fixtures)+";"+base+boundary+extraBoundary;
}
/** Mount the real application; only the native transport is a synthetic test boundary. */
export async function mountedFixture(root, directory, exercise, extraBoundary="") {
  fs.mkdirSync(directory, {recursive:true});
  const server=await startMountedApplication(root);
  const htmlPath=path.join(directory,"blank.html");fs.writeFileSync(htmlPath,"<!doctype html><html></html>");

  try {
    return await captureHtmlScreenshot({browser:resolveChromiumExecutable(),htmlPath,outputPath:path.join(directory,"last.png"),width:1440,height:900,async exercise(cdp){
      try {
      await cdp.send("Page.addScriptToEvaluateOnNewDocument",{source:mountedBoundarySource(root,extraBoundary)});
      const navigation=await cdp.send("Page.navigate",{url:server.origin});
      if(navigation.errorText)throw Error("Mounted navigation failed: "+navigation.errorText);
      await cdp.send("Page.bringToFront");
      await cdp.send("Emulation.setFocusEmulationEnabled",{enabled:true});
      const evaluate=async expression=>{const value=await cdp.send("Runtime.evaluate",{expression,awaitPromise:true,returnByValue:true});if(value.exceptionDetails){const detail=await cdp.send("Runtime.evaluate",{expression:"JSON.stringify({body:document.body.innerText,calls:window.__calls,errors:window.__errors})",returnByValue:true});fs.writeFileSync(path.join(directory,"failure.json"),detail.result.value);throw Error(value.exceptionDetails.exception?.description??value.exceptionDetails.text);}return value.result.value;};
      const startup=await waitForMountedApplication(cdp);
      fs.writeFileSync(path.join(directory,"application-startup.json"),JSON.stringify({application:server.application,...startup},null,2));
      const screenshot=async name=>{
        await cdp.send("Page.bringToFront");
        let timer;
        try {
          const shot=await Promise.race([cdp.send("Page.captureScreenshot",{format:"png",captureBeyondViewport:false}),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error("Screenshot timed out: "+name)),20000);})]);
          fs.writeFileSync(path.join(directory,name+".png"),Buffer.from(shot.data,"base64"));
        } finally {clearTimeout(timer);}
      };
      const route=async page=>evaluate('(async()=>{const page='+JSON.stringify(page)+',module={dashboard:"home",runs:"matches",review:"matches",prediction:"matches",lineups:"matches",workbooks:"resources",teams:"resources",players:"resources",rules:"model",release:"model",database:"management",architecture:"management",analytics:"analysis"}[page],rail=document.querySelector("[data-primary-module="+module+"]");if(!rail)throw Error("Missing module "+module);const collapsed=document.querySelector(".secondary-sidebar").hidden;if(collapsed){Array.from(document.querySelectorAll("[data-action=toggle-global-sidebar]")).find(e=>!e.closest("[hidden],[inert]")).click();await new Promise(r=>setTimeout(r,100));}document.querySelector("[data-primary-module="+module+"]").click();await new Promise(r=>setTimeout(r,200));const link=document.querySelector(".secondary-sidebar [data-page="+page+"]");if(link)link.click();await new Promise((resolve,reject)=>{const end=Date.now()+10000;const ready=()=>document.querySelector(".application-shell")?.dataset.currentPage===page&&!document.querySelector(".page-container[inert]")?resolve():Date.now()>end?reject(Error("Route did not settle: "+page)):setTimeout(ready,30);ready();});await new Promise(r=>setTimeout(r,200));if(collapsed)Array.from(document.querySelectorAll("[data-action=toggle-global-sidebar]")).find(e=>!e.closest("[hidden],[inert]")).click();await document.fonts.ready;return {page:document.querySelector(".application-shell").dataset.currentPage,title:document.querySelector("h1")?.textContent};})()');
      return await exercise({cdp,evaluate,screenshot,route});
      } catch(error) {
        await reportMountedFailure(cdp,{root,directory,error});
        throw error;
      }
    }});
  } finally { await server.close(); }
}
