import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { captureHtmlScreenshot, resolveChromiumExecutable } from "../task-ui-screenshot-tools.mjs";
import { loadTeamsProduction } from "../features/loadTeamsProduction.mjs";

/** Mount the real application; only the native transport is a synthetic test boundary. */
export async function mountedFixture(root, directory, exercise) {
  fs.mkdirSync(directory, {recursive:true});
  const require=createRequire(path.join(root,"package.json"));
  const {createServer}=await import(pathToFileURL(require.resolve("vite")));
  const server=await createServer({root,configFile:false,server:{host:"127.0.0.1",port:15328,strictPort:true},clearScreen:false});
  const fixtures=loadTeamsProduction(root)("scripts/design-refresh/fixtureData.ts");
  const base=fs.readFileSync(path.join(root,"scripts/features/lineups-browser/boundary.js"),"utf8");
  const boundary=fs.readFileSync(new URL("./mountedBoundary.js",import.meta.url),"utf8");
  const htmlPath=path.join(directory,"blank.html");fs.writeFileSync(htmlPath,"<!doctype html><html></html>");
  await server.listen();
  try {
    return await captureHtmlScreenshot({browser:resolveChromiumExecutable(),htmlPath,outputPath:path.join(directory,"last.png"),width:1440,height:900,async exercise(cdp){
      await cdp.send("Page.addScriptToEvaluateOnNewDocument",{source:"window.__fixtures="+JSON.stringify(fixtures)+";"+base+boundary});
      await cdp.send("Page.navigate",{url:"http://127.0.0.1:15328"});
      await cdp.send("Page.bringToFront");
      await cdp.send("Emulation.setFocusEmulationEnabled",{enabled:true});
      const evaluate=async expression=>{const value=await cdp.send("Runtime.evaluate",{expression,awaitPromise:true,returnByValue:true});if(value.exceptionDetails){const detail=await cdp.send("Runtime.evaluate",{expression:"JSON.stringify({body:document.body.innerText,calls:window.__calls,errors:window.__errors})",returnByValue:true});fs.writeFileSync(path.join(directory,"failure.json"),detail.result.value);throw Error(value.exceptionDetails.exception?.description??value.exceptionDetails.text);}return value.result.value;};
      await evaluate('new Promise((resolve,reject)=>{const end=Date.now()+15000;const check=()=>document.querySelector(".quick-actions")?resolve(true):Date.now()>end?reject(Error(document.body.innerText)):setTimeout(check,30);check();})');
      const screenshot=async name=>{
        await cdp.send("Page.bringToFront");
        let timer;
        try {
          const shot=await Promise.race([cdp.send("Page.captureScreenshot",{format:"png",captureBeyondViewport:false}),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error("Screenshot timed out: "+name)),20000);})]);
          fs.writeFileSync(path.join(directory,name+".png"),Buffer.from(shot.data,"base64"));
        } finally {clearTimeout(timer);}
      };
      const route=async page=>evaluate('(async()=>{const page='+JSON.stringify(page)+',module={dashboard:"home",teams:"resources",players:"resources",rules:"model",database:"management",architecture:"management"}[page],rail=document.querySelector("[data-primary-module="+module+"]");if(!rail)throw Error("Missing module "+module);const collapsed=document.querySelector(".secondary-sidebar").hidden;if(collapsed){Array.from(document.querySelectorAll("[data-action=toggle-global-sidebar]")).find(e=>!e.closest("[hidden],[inert]")).click();await new Promise(r=>setTimeout(r,100));}document.querySelector("[data-primary-module="+module+"]").click();await new Promise(r=>setTimeout(r,200));const link=document.querySelector(".secondary-sidebar [data-page="+page+"]");if(link)link.click();await new Promise((resolve,reject)=>{const end=Date.now()+10000;const ready=()=>document.querySelector(".application-shell")?.dataset.currentPage===page&&!document.querySelector(".page-container[inert]")?resolve():Date.now()>end?reject(Error("Route did not settle: "+page)):setTimeout(ready,30);ready();});await new Promise(r=>setTimeout(r,200));if(collapsed)Array.from(document.querySelectorAll("[data-action=toggle-global-sidebar]")).find(e=>!e.closest("[hidden],[inert]")).click();await document.fonts.ready;return {page:document.querySelector(".application-shell").dataset.currentPage,title:document.querySelector("h1")?.textContent};})()');
      return exercise({cdp,evaluate,screenshot,route});
    }});
  } finally { await server.close(); }
}
