import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {createRequire} from "node:module";
import {pathToFileURL} from "node:url";
import {createHash} from "node:crypto";

// One immutable application build per verification process, never a previous dist or ready-state cache.
const builds=new Map();
const viteFor=async root=>{
  const require=createRequire(path.join(root,"package.json"));
  return import(pathToFileURL(require.resolve("vite")));
};

export function prepareMountedApplication(root) {
  const project=fs.realpathSync(root);
  if(!builds.has(project)) {
    const task=(async()=>{
      const vite=await viteFor(project);
      const directory=fs.mkdtempSync(path.join(os.tmpdir(),"football-mounted-app-"));
      const outDir=path.join(directory,"dist");
      await vite.build({root:project,clearScreen:false,build:{outDir,emptyOutDir:false}});
      const htmlPath=path.join(outDir,"index.html"),html=fs.readFileSync(htmlPath,"utf8");
      const entry=[...html.matchAll(/<script\b[^>]*\bsrc="([^"]+)"[^>]*>/g)].map(m=>m[1]);
      if(!entry.length||entry.some(url=>!url.startsWith("/assets/")||!url.endsWith(".js")))throw Error("Mounted verification requires built application scripts");
      const info={root:project,directory,outDir,htmlPath,entry,indexSHA256:createHash("sha256").update(html).digest("hex"),viteVersion:vite.version};
      fs.writeFileSync(path.join(directory,"build.json"),JSON.stringify(info,null,2));
      console.log("Mounted production application prepared: "+directory);
      return Object.freeze(info);
    })();
    builds.set(project,task);
    task.catch(()=>{if(builds.get(project)===task)builds.delete(project);});
  }
  return builds.get(project);
}

/** Serve exactly the current production build; a fresh native boundary and browser still mount each suite. */
export async function startMountedApplication(root) {
  const application=await prepareMountedApplication(root),vite=await viteFor(application.root);
  const server=await vite.preview({root:application.root,clearScreen:false,build:{outDir:application.outDir},preview:{host:"127.0.0.1",port:15328,strictPort:true}});
  let closed=false;
  return {
    application,origin:"http://127.0.0.1:15328",
    async close(){
      if(closed)return;closed=true;
      const closing=new Promise((resolve,reject)=>server.httpServer.close(error=>error?reject(error):resolve()));
      server.httpServer.closeAllConnections();
      await closing;
    },
  };
}
