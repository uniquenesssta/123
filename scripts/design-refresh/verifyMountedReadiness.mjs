import assert from "node:assert/strict";
import {waitForMountedApplication} from "./mountedReadiness.mjs";

export async function verifyMountedReadiness() {
  const state=patch=>({dashboard:false,quickActions:false,fatal:null,errors:[],...patch});
  const cdp=read=>({send:async()=>({result:{value:await read()}})});
  let attempt=0;
  const ready=await waitForMountedApplication(cdp(()=>state(++attempt===3?{dashboard:true,quickActions:true}:{})),{timeoutMs:1000});
  assert.equal(ready.dashboard,true);
  assert.equal(attempt,3);
  await assert.rejects(waitForMountedApplication(cdp(()=>state({fatal:"broken bootstrap"}))),/broken bootstrap/);
  await assert.rejects(waitForMountedApplication(cdp(()=>state({errors:["module syntax failure"]}))),/module syntax failure/);
  await assert.rejects(waitForMountedApplication(cdp(()=>state({quickActions:true})),{timeoutMs:70}),/did not become ready/);
  let contexts=0;
  await waitForMountedApplication({send:async()=>{if(++contexts===1)throw Error("Execution context was destroyed");return{result:{value:state({dashboard:true,quickActions:true})}};}},{timeoutMs:1000});
  await assert.rejects(waitForMountedApplication({send:async()=>{throw Error("protocol failure");}}),/protocol failure/);
  console.log("Mounted startup gate: 6 readiness/error/context/deadline checks PASS");
}
