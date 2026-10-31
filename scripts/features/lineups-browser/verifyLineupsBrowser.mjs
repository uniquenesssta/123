import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
import {pathToFileURL,fileURLToPath} from 'node:url';
import {tmpdir} from 'node:os';
export async function verifyLineupsBrowser(root) {
const require=createRequire(root+'/package.json');
const {loadTeamsProduction}=await import(pathToFileURL(root+'/scripts/features/loadTeamsProduction.mjs'));
const fixtures=loadTeamsProduction(root)('scripts/design-refresh/fixtureData.ts');
const {createServer}=await import(pathToFileURL(require.resolve('vite')));
const {captureHtmlScreenshot,resolveChromiumExecutable}=await import(pathToFileURL(root+'/scripts/task-ui-screenshot-tools.mjs'));
const evidence=fs.mkdtempSync(path.join(tmpdir(),'r13-07-browser-'));
const htmlPath=path.join(evidence,'blank.html');fs.writeFileSync(htmlPath,'<!doctype html><html></html>');
const server=await createServer({root,configFile:false,server:{host:'127.0.0.1',port:15327,strictPort:true},clearScreen:false});await server.listen();
const setup=`window.__fixtures=${JSON.stringify(fixtures)};`+fs.readFileSync(new URL('./boundary.js',import.meta.url),'utf8');
try{
 const result=await captureHtmlScreenshot({browser:resolveChromiumExecutable(),htmlPath,outputPath:path.join(evidence,'lineups.png'),width:1440,height:900,async exercise(cdp){
 await cdp.send('Page.addScriptToEvaluateOnNewDocument',{source:setup});await cdp.send('Page.navigate',{url:'http://127.0.0.1:15327'});
 const ev=async expression=>{const r=await cdp.send('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});if(r.exceptionDetails){const d=await cdp.send('Runtime.evaluate',{expression:'JSON.stringify({body:document.body.innerText,calls:window.__calls,errors:window.__errors})',returnByValue:true});fs.writeFileSync(path.join(evidence,'failure.json'),d.result.value);console.log('Failure evidence: '+evidence);throw Error(r.exceptionDetails.exception?.description??r.exceptionDetails.text);}return r.result.value;};
 await ev(`new Promise((resolve,reject)=>{const end=Date.now()+15000;const run=()=>document.querySelector('.quick-actions')?resolve(true):Date.now()>end?reject(Error(document.body.innerText)):setTimeout(run,40);run();})`);


 await ev('(async()=>{document.querySelector(\'[data-primary-module="matches"]\').click();await new Promise(r=>setTimeout(r,400));})()');
 const matchLayouts=[];
 for(const [width,height] of [[1440,900],[960,720],[1280,720],[1920,1080]]) {
 await cdp.send('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:1,mobile:false});
 const shape=await ev('(async()=>{await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));const q=s=>document.querySelector(s),b=s=>{const r=q(s).getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height,bottom:r.bottom};};return {width:innerWidth,overflow:document.documentElement.scrollWidth>innerWidth,left:b(".match-browser-sidebar"),right:b(".match-browser-detail"),editor:b(".match-context-form")};})()');
 if(shape.overflow||Math.abs(shape.left.width-248)>1||Math.abs(shape.left.y-shape.right.y)>1||shape.left.height<height-300)throw Error('Match geometry '+JSON.stringify(shape));matchLayouts.push(shape);
 }
 await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false});
 const matchShot=await cdp.send('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});fs.writeFileSync(path.join(evidence,'matches-1440-light.png'),Buffer.from(matchShot.data,'base64'));
 const result=await ev(fs.readFileSync(new URL('./cases.js',import.meta.url),'utf8'));
 await ev('(async()=>{document.querySelector(\'[data-action="open-team-lineup-preset-editor"][data-preset-id="preset-1"]\').click();await new Promise(r=>setTimeout(r,200));})()');
 result.matches=matchLayouts;result.checks+=16;result.editor=[];
 for(const [width,height] of [[1440,900],[960,720],[1280,720],[1920,1080]])for(const theme of ['light','dark']) {
 await cdp.send('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:1,mobile:false});
 const shape=await ev('(async()=>{document.documentElement.dataset.theme='+JSON.stringify(theme)+';await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));const q=s=>document.querySelector(s),b=e=>{const r=e.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height,bottom:r.bottom};},scroll=q(".preset-editor-scroll"),row=q(".preset-member-row"),head=q(".preset-member-table-head"),footer=q(".workspace-detail-footer");scroll.scrollTop=scroll.scrollHeight;await new Promise(r=>requestAnimationFrame(r));return {width:innerWidth,overflow:document.documentElement.scrollWidth>innerWidth,scroll:b(scroll),footer:b(footer),last:b(q(".preset-member-row:last-child")),columns:[...head.children].map(e=>b(e).x),row:[...row.children].map(e=>b(e).x),textSize:getComputedStyle(q(".preset-member-position")).fontSize};})()');
 if(shape.overflow||shape.footer.bottom>height+1||shape.last.bottom>shape.scroll.bottom+1||shape.columns.some((x,i)=>Math.abs(x-shape.row[i])>1)||shape.textSize!=='13px')throw Error('Editor geometry '+JSON.stringify(shape));result.editor.push({theme,...shape});result.checks+=5;
 const shot=await cdp.send('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});fs.writeFileSync(path.join(evidence,'editor-'+width+'-'+theme+'.png'),Buffer.from(shot.data,'base64'));
 }
 await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false});
 await ev('document.documentElement.dataset.theme="light";document.querySelector(".preset-editor-scroll").scrollTop=0');
 const editorShot=await cdp.send('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});fs.writeFileSync(path.join(evidence,'editor-1440-light.png'),Buffer.from(editorShot.data,'base64'));
 await ev('document.querySelector(\'[data-action="close-workspace-detail"]\').click()');

 result.responsive=[];
 for(const [width,height] of [[1440,900],[960,720],[1280,720],[1920,1080]])for(const theme of ['light','dark']){
 await cdp.send('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:1,mobile:false});
 const layout=await ev('(async()=>{document.documentElement.dataset.theme='+JSON.stringify(theme)+';await document.fonts.ready;await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));const q=s=>document.querySelector(s),left=q(".lineup-preset-directory").getBoundingClientRect(),right=q(".lineup-preset-main").getBoundingClientRect(),footer=q(".entity-directory-footer").getBoundingClientRect();return {width:innerWidth,height:innerHeight,overflow:document.documentElement.scrollWidth>innerWidth,left:{x:left.x,y:left.y,width:left.width,bottom:left.bottom},right:{x:right.x,y:right.y,width:right.width,bottom:right.bottom},footerBottom:footer.bottom,font:document.fonts.check("12px \\\"Noto Sans SC\\\""),cards:q(".lineup-preset-main").querySelectorAll(".lineup-preset-card").length};})()');
 if(layout.overflow||Math.abs(layout.left.y-layout.right.y)>1||layout.footerBottom>height+1)throw Error('Responsive geometry failed '+JSON.stringify(layout));
 const shot=await cdp.send('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});fs.writeFileSync(path.join(evidence,'preset-'+width+'-'+theme+'.png'),Buffer.from(shot.data,'base64'));result.responsive.push({theme,...layout});result.checks+=4;
 }
 return result;
 }});
 fs.writeFileSync(path.join(evidence,'result.json'),JSON.stringify(result,null,2));console.log(JSON.stringify({result:'PASS',checks:result.checks,evidence}));
}finally{await server.close();}

}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url))await verifyLineupsBrowser(path.resolve(fileURLToPath(new URL("../../..",import.meta.url))));
