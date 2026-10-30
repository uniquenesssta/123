import { entityDirectoryView, entityDetailView, workbenchView, taskPageHeader, asyncBoundaryView, fileWorkflowView, DestructiveActionController } from "../index";
export function installPatternsFixture() {
  const root=document.querySelector<HTMLElement>("#patterns-root")!;
  const checks:string[]=[];
  const expect=(condition:unknown,message:string)=>{if(!condition)throw Error(message);checks.push(message);};
  const q=<T extends HTMLElement>(selector:string)=>document.querySelector<T>(selector)!;
  const controller=new DestructiveActionController();
  let calls=0, resolveAction:(()=>void)|null=null;
  const heading=taskPageHeader({eyebrow:"不得重复呈现",title:"交互结构验收",description:"合成数据测试夹具 · 不连接数据库",actions:'<button id="open-confirm">删除示例对象</button>'});
  const files=(preview:string|null)=>fileWorkflowView({heading:'<h2>文件预检</h2>',entry:'<button data-action="pick-file">选择文件</button>',preview,empty:'<p data-file-empty>尚未选择文件</p>'});
  function paint(){
    root.innerHTML=workbenchView({className:"pattern-test-workbench",heading,navigation:'<button data-section-id="directory">目录</button><button data-section-id="profile">完整档案</button>',context:'<p>合成对象A · 已知范围</p>',content:entityDirectoryView({entity:'team" unsafe',className:"test-directory",content:'<aside class="test-filter"><h2>筛选</h2><input id="filter-draft" value="保留筛选草稿"><button data-action="filter">应用筛选</button></aside><div class="test-results"><h2>当前结果</h2><button data-action="open-entity">示例对象A</button></div>',inspector:{collapsed:false,content:'<aside class="test-inspector"><h2>速览</h2><button data-action="close-inspector">关闭速览</button></aside>'}})+entityDetailView({title:"<示例对象A> · 完整档案",context:"当前对象",actions:'<button data-action="return-directory">返回目录</button>',content:'<label>名称<input id="entity-draft" value="待保存草稿"></label>'})+files(null)+files('<section data-real-preview><h3>需要核对的预检</h3><button data-action="commit-file" disabled>冲突未解决，禁止提交</button></section>')});
  }
  function structure(){
    expect(root.querySelectorAll('[data-ui-pattern="workbench"]').length===1,"one shared workbench composition");
    expect(root.querySelectorAll("h1").length===1&&!root.textContent!.includes("不得重复呈现"),"one heading without duplicate eyebrow");
    expect(q('[data-ui-pattern="entity-directory"]').getAttribute("data-entity-browser")==='team" unsafe',"entity identity escaped as an attribute");
    expect(q(".test-directory").classList.contains("inspector-open"),"host-provided inspector state is represented");
    const collapsed=document.createElement("div");collapsed.innerHTML=entityDirectoryView({entity:"players",content:"真实结果",inspector:{content:"速览",collapsed:true},selection:'<div data-selection>选中操作</div>'});
    expect(collapsed.firstElementChild!.classList.contains("inspector-collapsed"),"collapsed inspector uses approved layout class");
    expect(collapsed.querySelector('[data-selection]')!.parentElement===collapsed,"selection bar is outside independently scrolling result frame");
    const detail=q('[data-ui-pattern="entity-detail"]');
    expect(detail.querySelector("h2")!.textContent==="<示例对象A> · 完整档案"&&!detail.querySelector("示例对象A"),"entity title is text, never markup");
    expect(!!detail.querySelector('[data-action="return-directory"]'),"host return destination retained");
    expect(q<HTMLInputElement>("#entity-draft").value==="待保存草稿","detail keeps native edit fields");
    const workflows=root.querySelectorAll<HTMLElement>('[data-ui-pattern="file-workflow"]');
    expect(workflows[0].dataset.filePhase==="unselected"&&!!workflows[0].querySelector('[data-file-empty]'),"no preview presents only explicit empty state");
    expect(!workflows[0].querySelector('[data-action="commit-file"]'),"file selection cannot imply commit readiness");
    expect(workflows[1].dataset.filePhase==="review"&&!workflows[1].querySelector('[data-file-empty]'),"only actual host preview enters review");
    expect(q<HTMLButtonElement>('[data-action="commit-file"]').disabled,"host conflict guard remains authoritative");
    const host=document.createElement("div");
    host.innerHTML=asyncBoundaryView({content:'<input id="retained" value="原草稿">',pending:true,retryAction:"retry-real-read",logsAction:"logs"});
    expect(host.querySelector<HTMLInputElement>("#retained")!.value==="原草稿","pending retains original read content and draft markup");
    expect(host.querySelector<HTMLElement>(".page-container")!.inert&&host.querySelector<HTMLElement>("#modal-root")!.inert,"pending gates workspace and detail interactions");
    expect(host.querySelector(".page-container")!.getAttribute("aria-busy")==="true"&&!!host.querySelector('[role="status"]'),"pending announces one progress region");
    host.innerHTML=asyncBoundaryView({content:"stale result",pending:false,failed:true,retryAction:"retry-real-read",logsAction:"logs"});
    expect(!host.textContent!.includes("stale result")&&!!host.querySelector('[data-action="retry-real-read"]'),"failure exposes host read recovery, never stale result");
    expect(!!host.querySelector('[data-action="logs"]')&&host.querySelector(".page-container")!.hasAttribute("data-workspace-capture"),"failure exposes diagnostics and suspends draft capture");
    host.innerHTML=asyncBoundaryView({content:"loaded",pending:false,retryAction:"retry-real-read",logsAction:"logs"});
    expect(!host.querySelector<HTMLElement>(".page-container")!.inert&&!host.querySelector(".page-load-progress"),"settled state unlocks content without own requests");
    expect(root.scrollWidth<=innerWidth,"composed patterns have no document overflow");
  }
  function open(mode:"success"|"failure"|"deferred"="success"){
    q("#open-confirm").focus();
    controller.show({title:"删除示例对象",description:"合成测试对象，不读取实际资料。",facts:[["对象","示例对象A"]],confirmLabel:"确认删除",tone:"danger",typed:{expected:"示例对象A",label:"输入对象名",id:"pattern-confirm-name",hint:"核对名称后才可提交",confirmButtonId:"pattern-confirm-submit"},formatError:()=>"写入结果未知，需要核对",onConfirm:async()=>{calls++;if(mode==="failure")throw Error("unknown outcome");if(mode==="deferred")await new Promise<void>(r=>resolveAction=r);}});
  }
  paint();
  return {q,checks,expect,structure,controller,open,get calls(){return calls;},settle(){resolveAction?.();resolveAction=null;},get draft(){return q<HTMLInputElement>("#filter-draft").value;}};
}
