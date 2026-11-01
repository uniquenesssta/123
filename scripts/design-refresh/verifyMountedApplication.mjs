import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {createRequire} from "node:module";
import {pathToFileURL} from "node:url";
import {captureHtmlScreenshot,resolveChromiumExecutable} from "../task-ui-screenshot-tools.mjs";
import {prepareMountedApplication,startMountedApplication} from "./mountedApplication.mjs";
import {mountedBoundarySource} from "./mountedFixture.mjs";
import {waitForMountedApplication,mountedSnapshot,reportMountedFailure} from "./mountedReadiness.mjs";

/** Actual old dev-module failure, the built application's cold startup, and a still-blocked bundle negative. */
export async function verifyMountedApplication(root,directory) {
  fs.mkdirSync(directory,{recursive:true});
  const [first,second]=await Promise.all([prepareMountedApplication(root),prepareMountedApplication(root)]);
  assert.equal(first,second,"all suites share one current immutable build, not duplicate temporary outputs");
  const require=createRequire(path.join(root,"package.json")),vite=await import(pathToFileURL(require.resolve("vite")));
  let dev=await vite.createServer({root,configFile:false,server:{host:"127.0.0.1",port:15329,strictPort:true},clearScreen:false}),preview;
  await dev.listen();
  const htmlPath=path.join(directory,"blank.html");fs.writeFileSync(htmlPath,"<!doctype html><html></html>");
  try {
    await captureHtmlScreenshot({browser:resolveChromiumExecutable(),htmlPath,outputPath:path.join(directory,"last.png"),width:1440,height:900,async exercise(cdp){
      try {
        await cdp.send("Page.addScriptToEvaluateOnNewDocument",{source:mountedBoundarySource(root)});
        await cdp.send("Network.enable");
        await cdp.send("Network.setCacheDisabled",{cacheDisabled:true});
        await cdp.send("Network.setBlockedURLs",{urls:["*/src/*"]});
        await cdp.send("Page.navigate",{url:"http://127.0.0.1:15329"});
        await assert.rejects(waitForMountedApplication(cdp,{timeoutMs:1500}),/did not become ready|startup failed/);
        const sourceBlocked=await mountedSnapshot(cdp);
        assert.equal(sourceBlocked.dashboard,false);assert.equal(sourceBlocked.commands.length,0);
        await dev.environments.client.depsOptimizer?.scanProcessing;
        await dev.close();dev=null;

        preview=await startMountedApplication(root);
        const start=Date.now();await cdp.send("Page.navigate",{url:preview.origin});
        const ready=await waitForMountedApplication(cdp);
        const elapsedMs=Date.now()-start;
        assert.equal(ready.dashboard,true);assert.equal(ready.quickActions,true);assert.ok(ready.commands.includes("bootstrap"));
        assert.ok(ready.resources.some(r=>/\/assets\/[^/?]+\.js/.test(r.name)),"real compiled application requested");
        assert.ok(ready.resources.every(r=>!new URL(r.name).pathname.startsWith("/src/")&&!r.name.includes("/@vite/client")),"cold startup has no dev-module compilation waterfall");
        const gate=fs.readFileSync(new URL("./mountedReadiness.mjs",import.meta.url),"utf8");
        assert.ok(gate.includes("timeoutMs=30000"),"unchanged production readiness deadline");

        await cdp.send("Network.setBlockedURLs",{urls:["*/src/*","*/assets/*.js"]});
        await cdp.send("Page.navigate",{url:preview.origin});
        await assert.rejects(waitForMountedApplication(cdp,{timeoutMs:1500}),/did not become ready|startup failed/);
        const bundleBlocked=await mountedSnapshot(cdp);assert.equal(bundleBlocked.dashboard,false);assert.equal(bundleBlocked.commands.length,0);

        await cdp.send("Network.setBlockedURLs",{urls:["*/src/*"]});
        await cdp.send("Page.navigate",{url:preview.origin});
        await waitForMountedApplication(cdp);
        fs.writeFileSync(path.join(directory,"results.json"),JSON.stringify({status:"PASS",sharedBuild:true,cacheDisabled:true,sourceBlocked,ready,elapsedMs,defaultDeadlineMs:30000,bundleBlocked},null,2));
      }catch(error){await reportMountedFailure(cdp,{root,directory,error});throw error;}
    }});
  } finally {if(dev)await dev.close();if(preview)await preview.close();}
  console.log("Mounted application: 5 build-sharing/cold-module/bundle/error/readiness regressions PASS");
}
