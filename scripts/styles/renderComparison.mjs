/** Runs in the real browser over the unchanged production view DOM. */
export async function compareRenderedStyles(screen,theme,collapsed,featureSelectors){
 const before=document.querySelector("#baseline-css"),after=document.querySelector("#current-css");
 after.disabled=true;before.disabled=false;renderFixture(screen,theme,collapsed);const sentinel=document.createElement("div");sentinel.id="style-toggle-sentinel";sentinel.style.cssText="position:fixed;left:-9999px;top:0;height:0";document.body.append(sentinel);await document.fonts.ready;await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
 if(!after.disabled||before.disabled||getComputedStyle(document.querySelector("#style-toggle-sentinel")).width!=="31px")throw Error("Baseline stylesheet was not exclusively active");
 const elements=[...document.querySelectorAll(".application-shell,.application-shell *,[data-confirmation-host]")];
 const snapshot=()=>elements.map(element=>{const style=getComputedStyle(element),rect=element.getBoundingClientRect();return {name:element.tagName+"#"+element.id+"."+element.className,style:Object.fromEntries([...style].map(p=>[p,style.getPropertyValue(p)])),geometry:[rect.x,rect.y,rect.width,rect.height,element.scrollWidth,element.scrollHeight]};});
 const baseline=snapshot();before.disabled=true;after.disabled=false;await document.fonts.ready;await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
 if(!before.disabled||after.disabled||getComputedStyle(document.querySelector("#style-toggle-sentinel")).width!=="47px")throw Error("Current stylesheet was not exclusively active");
 const current=snapshot(),differences=[];let properties=0;
 for(let i=0;i<baseline.length;i++){const left=baseline[i],right=current[i];for(const key of Object.keys(left.style)){properties++;if(left.style[key]!==right.style[key]&&differences.length<30)differences.push({element:left.name,property:key,before:left.style[key],after:right.style[key]});}if(left.geometry.some((n,j)=>Math.abs(n-right.geometry[j])>.05)&&differences.length<30)differences.push({element:left.name,property:"geometry",before:left.geometry,after:right.geometry});}
 const shell=document.querySelector(".application-shell"),owner=shell.dataset.currentPage,selectors=featureSelectors[owner];let confined=0;
 if(selectors){const active=selectors.reduce((n,s)=>n+document.querySelectorAll(s).length,0);if(!active)throw Error("No real feature selectors matched "+owner);shell.dataset.currentPage="style-isolation-probe";for(const selector of selectors){if(document.querySelectorAll(selector).length)throw Error("Feature selector escaped its actual route: "+selector);confined++;}shell.dataset.currentPage=owner;}
 scrollTo(0,0);await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
 return {elements:elements.length,properties,confined,differences,windows:navigator.userAgent.includes("Windows")};
}
