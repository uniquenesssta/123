import fs from "node:fs";
import path from "node:path";

const snapshotExpression = `(()=>({
  url:location.href,
  readyState:document.readyState,
  dashboard:document.querySelector('.application-shell')?.dataset.currentPage==='dashboard',
  quickActions:!!document.querySelector('.quick-actions'),
  fatal:document.querySelector('.fatal')?.textContent??null,
  body:document.body?.innerText?.slice(0,12000)??'',
  errors:window.__errors??[],
  commands:(window.__calls??[]).map(call=>call.command),
  resources:performance.getEntriesByType('resource').slice(-25).map(r=>({name:r.name,duration:r.duration}))
}))()`;

export async function mountedSnapshot(cdp) {
  const response=await cdp.send("Runtime.evaluate",{expression:snapshotExpression,returnByValue:true});
  if(response.exceptionDetails)throw Error(response.exceptionDetails.exception?.description??response.exceptionDetails.text);
  return response.result.value;
}

/** Wait outside the page context: navigation can replace the document while Vite compiles it. */
export async function waitForMountedApplication(cdp,{timeoutMs=30000}={}) {
  const deadline=Date.now()+timeoutMs;
  let last=null;
  do {
    try {
      last=await mountedSnapshot(cdp);
      if(last.fatal||last.errors.length)throw Error("Mounted application startup failed: "+(last.fatal||JSON.stringify(last.errors)));
      if(last.dashboard&&last.quickActions)return last;
    } catch(error) {
      if(!/Execution context was destroyed|Cannot find context|Inspected target navigated/i.test(String(error)))throw error;
    }
    await new Promise(resolve=>setTimeout(resolve,50));
  } while(Date.now()<deadline);
  throw Error("Mounted dashboard did not become ready within "+timeoutMs+"ms: "+JSON.stringify(last));
}

/** Always upload failure evidence in CI; temporary fixture directories are not included by that workflow. */
export async function reportMountedFailure(cdp,{root,directory,error}) {
  const state=await mountedSnapshot(cdp).catch(cause=>({diagnosticError:String(cause)}));
  const report={error:String(error),...state};
  fs.writeFileSync(path.join(directory,"failure.json"),JSON.stringify(report,null,2));
  const shot=await cdp.send("Page.captureScreenshot",{format:"png",captureBeyondViewport:false}).catch(()=>null);
  if(shot)fs.writeFileSync(path.join(directory,"failure.png"),Buffer.from(shot.data,"base64"));
  if(process.env.CI){
    const target=path.resolve(root,"..","logs","mounted-failure-"+path.basename(directory)+"-"+Date.now());
    fs.mkdirSync(target,{recursive:true});
    for(const name of fs.readdirSync(directory)){
      if(/^(?:failure(?:[-\w]*)?|results)\.(?:json|txt|png)$/.test(name))fs.copyFileSync(path.join(directory,name),path.join(target,name));
    }
    console.error("Mounted browser failure evidence: "+target);
  }
}
