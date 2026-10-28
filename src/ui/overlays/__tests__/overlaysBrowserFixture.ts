import { EventScope } from "../../../app/lifecycle";
import { ConfirmationDialogController } from "../confirmation-dialog";
import { CommandMenuController, type UiCommand } from "../command-menu";
import { PopoverController } from "../popover";
import { DialogController } from "../dialog";
import { drawerView } from "../drawer";
import { toastView } from "../../feedback/toast/toastView";
import { buttonView } from "../../primitives/button";
import { alertView, inlineMessageView, operationStateView, blockingMessageView } from "../../feedback";
import { ModalController } from "../../../app/modal";
import { ContextMenuAdapter } from "../../../app/context-menu";
import { PRIMARY_NAVIGATION } from "../../../app/router";

export function installOverlaysFixture() {
  const root=document.querySelector<HTMLElement>('#overlays-root')!;
  const checks: string[]=[];const expect=(ok: unknown,label: string)=>{if(!ok)throw new Error(label);checks.push(label);};
  const q=<T extends HTMLElement=HTMLElement>(selector: string)=>{const n=document.querySelector<T>(selector);if(!n)throw new Error("Missing "+selector);return n;};
  root.innerHTML='<button id="trigger">操作入口</button><button id="outside">外部操作</button><main class="main-content"><div class="page-container"><input id="page-draft" value="保留的页面草稿"></div><div id="modal-root"></div></main><div data-confirmation-host></div><div id="drawer-host"></div><div id="gallery" class="overlay-gallery"></div>';
  const gallery=q('#gallery');
  gallery.innerHTML=["info","success","warning","danger"].map(tone=>'<section>'+alertView({tone:tone as "info",title:({info:"资料说明",success:"保存完成",warning:"资料尚未完整",danger:"数据未能载入"} as Record<string,string>)[tone],message:"真实业务状态由调用方提供；提示关闭不会解除保存门禁。"})+inlineMessageView({tone:tone as "info",message:"当前输入已保留，可以核对后继续。",dismissAction:"fixture-dismiss"})+'</section>').join('')+["unavailable","blocked","error","retrying","fatal"].map((state,index)=>'<section>'+blockingMessageView({state:state as "error",title:["请先连接数据库","请先处理缺失项","本页数据未能载入","正在重新检查","应用未能启动"][index],message:["填写连接信息后再使用数据功能。","检查阻断原因后重新检查。","检查连接并查看问题日志；不要重复提交写入。","正在检查连接，请稍候。","请查看启动日志中的恢复指引，保留现有资料。 "][index],primary:state==="fatal"?undefined:{label:"重新检查",attributes:{"data-fixture-check":true}},secondary:state==="fatal"?undefined:{label:"查看问题日志"}})+'</section>').join('')+operationStateView({state:"success",message:"操作已完成"});
  const events=new EventScope(),confirmation=new ConfirmationDialogController(root),popover=new PopoverController(),command=new CommandMenuController(document,()=>"连接读取失败，请检查后重试"),adapter=new ContextMenuAdapter();
  const modal=new ModalController(()=>"操作结果待核对");
  const drawer=new DialogController(root,()=>q('#trigger'),()=>drawer.close(),{dismissOnBackdrop:false});
  events.listen(root,"click",e=>{if(confirmation.handleClick(e)){e.stopPropagation();return;}const t=e.target instanceof Element?e.target:null;if(t?.closest('[data-fixture-drawer-close]'))drawer.close();if(t?.closest('[data-fixture-nested-menu]'))nested();if(t?.closest('[data-action="fixture-dismiss"]'))t.closest('.ui-alert')?.remove();});
  events.listen(root,"cancel",e=>{if(!confirmation.handleCancel(e))drawer.handleCancel(e);},{capture:true});
  events.listen(root,"keydown",e=>{if(!confirmation.handleKeydown(e))drawer.handleKeydown(e);},{capture:true});
  let resolve: (()=>void)|null=null,reject: ((e: Error)=>void)|null=null,requests=0,navigated: string[]=[];
  const wait=()=>new Promise<void>((yes,no)=>{resolve=yes;reject=no;});
  function startConfirmation(typed=false,neutral=false) {
    q('#trigger').focus();confirmation.show({title:"确认当前操作",description:"核对影响后再提交；失败时保留当前输入。",facts:[["对象","测试专用示例"],["影响","仅验证 UI 请求生命周期，不调用数据库"]],confirmLabel:"确认提交",tone:neutral?"neutral":"danger",typed:typed?{id:"typed-name",confirmButtonId:"typed-submit",expected:"football_local",label:"输入完整名称",hint:"大小写和名称必须一致"}:undefined,onConfirm:()=>{requests++;return wait();},formatError:()=>"写入结果暂时无法确认"});
  }
  function showDrawer(long=false) {
    q('#trigger').focus();drawer.restore(q('#drawer-host'),drawerView({title:"告警中心",titleId:"fixture-drawer-title",description:"本次会话",descriptionId:"fixture-drawer-description",bodyClassName:"fixture-scroll-body",body:Array.from({length:long?50:2},(_,i)=>alertView({tone:i%2?"danger":"warning",title:i%2?"数据未能载入":"资料尚未完整",message:"请检查连接并查看问题日志，保留当前输入。"})).join('')+buttonView({label:"打开局部菜单",attributes:{"data-fixture-nested-menu":true}}),actions:buttonView({label:"关闭",attributes:{"data-fixture-drawer-close":true,autofocus:true}}),closeAttributes:{"data-fixture-drawer-close":true}}));
  }
  function nested() {popover.open({anchor:q('[data-fixture-nested-menu]'),label:"局部操作",items:[{id:"first",label:"查看当前告警",onSelect:()=>navigated.push("warning")},{id:"disabled",label:"暂不可用",disabled:true},{id:"last",label:"查看问题日志",onSelect:()=>navigated.push("logs")} ]});}
  const commands: UiCommand[]=PRIMARY_NAVIGATION.flatMap(module=>module.items.map(item=>({id:item.page,label:item.label,description:item.description,keywords:module.label,run:()=>{navigated.push(item.page);}})));
  return {
    checks,expect,q,confirmation,command,popover,modal,drawer,
    get requests(){return requests;},get navigated(){return navigated;},
    structure(){
      expect(q('#gallery').querySelectorAll('.ui-blocking-message').length===5,"five blocking states render");expect(!q('[data-state="fatal"]').querySelector('button'),"fatal does not invent a restart action");
      expect(Array.from(q('[data-state="retrying"]').querySelectorAll('button')).every(b=>b.disabled),"retrying disables all recovery actions");
      const body=getComputedStyle(q('.ui-feedback-copy p'));expect(body.fontSize==="12px"&&body.lineHeight==="18px"&&body.fontWeight==="400","body 12/18 regular");
      const title=getComputedStyle(q('.ui-feedback-copy strong'));expect(title.fontSize==="14px"&&title.lineHeight==="20px"&&title.fontWeight==="700","feedback title 14/20 bold");
      expect(getComputedStyle(q('.ui-alert')).borderRadius==="12px","approved feedback surface radius");
      expect(!q('.ui-alert').textContent?.includes('<script>'),"visible copy is plain text");
    },
    startConfirmation,
    checkOpen(){expect(!!document.querySelector('dialog:modal'),"native dialog creates modality");expect(document.activeElement?.getAttribute('data-confirmation-action')==="cancel","safe initial cancel focus");expect(q('dialog').getBoundingClientRect().width<=Math.min(480,innerWidth-24)+1,"dialog width clamps to viewport");expect(getComputedStyle(q('dialog h2')).fontSize==="18px","dialog title matches Figma 18px");},
    checkBusy(){expect(requests>0&&confirmation.busy,"real promise is pending");expect(Array.from(q('dialog').querySelectorAll('button,input')).every(n=>n.matches(':disabled')),"busy locks all controls");expect(q('dialog').getAttribute('aria-busy')==="true","busy ARIA synchronized");},
    complete(){resolve?.();},fail(){reject?.(new Error("test only"));},
    checkFailure(){expect(confirmation.active&&!confirmation.busy,"failure retains native dialog");expect(q('[data-confirmation-feedback]').textContent?.includes("核对实际结果"),"destructive unknown result requires review");expect(q<HTMLButtonElement>('[data-confirmation-action="confirm"]').disabled,"unknown write result cannot be blindly retried");expect(q<HTMLInputElement>('#typed-name').value==="football_local","failure preserves exact typed draft");},
    showDrawer,nested,
    openPopover(){q('#trigger').focus();popover.open({anchor:q('#trigger'),label:"记录操作",point:{x:innerWidth-2,y:innerHeight-2},items:[{id:"one",label:"查看记录",onSelect:()=>navigated.push("one")},{id:"disabled",label:"不可用操作",disabled:true},{id:"last",label:"删除测试记录",danger:true,icon:"alert-circle",onSelect:()=>navigated.push("last")} ]});},
    showToast(){const host=document.createElement("div");host.id="capture-toast";host.className="warning-feedback-stack";host.innerHTML=toastView({tone:"danger",message:"操作结果尚未确认，请先核对当前记录并查看问题日志。关闭通知不会取消正在执行的任务，也不会自动重试写入。",actionLabel:"查看问题日志"});document.body.append(host);},
    removeToast(){document.querySelector('#capture-toast')?.remove();},
    contextAdapter(){const target=q('#trigger');target.dataset.contextKind="run";target.dataset.runId="actual-identity";target.dataset.runLabel="上下文记录";adapter.open(new MouseEvent("contextmenu",{clientX:innerWidth-1,clientY:innerHeight-1,cancelable:true}),target);expect(q('#app-context-menu [data-action="request-hide-run-history"]').dataset.runId==="actual-identity","real context adapter preserves request action and identity");adapter.close();},
    openCommand(async=false){q('#trigger').focus();command.open(async?[{id:"deferred",label:"读取页面",description:"真实 promise 生命周期",run:()=>{requests++;return wait();}},{id:"disabled",label:"禁止执行",description:"disabled",disabled:true,run:()=>{throw new Error("disabled executed");}}]:commands);},
    async lifecycle(){
      confirmation.destroy();popover.close();command.close();drawer.close();
      modal.showHtml("原详情","草稿",'<input id="detail-draft" value="未保存草稿">');q<HTMLInputElement>('#detail-draft').value="保留的详情输入";
      modal.confirm("确认操作","保留原详情",[],"提交",()=>wait());q('[data-confirmation-action="confirm"]').click();modal.close();expect(!!document.querySelector('dialog:modal'),"host close cannot dismiss pending write");reject?.(new Error("failure"));await new Promise(r=>setTimeout(r,0));
      expect(q<HTMLInputElement>('#detail-draft').value==="保留的详情输入","business confirmation failure retains underlying detail draft");q('[data-confirmation-action="cancel"]').click();expect(!!document.querySelector('#detail-draft'),"cancel leaves detail history intact");
      modal.confirm("成功操作","成功后关闭旧详情",[],"提交",async()=>{});q('[data-confirmation-action="confirm"]').click();await new Promise(r=>setTimeout(r,0));expect(!document.querySelector('#detail-draft')&&!document.querySelector('dialog:modal'),"successful business confirmation closes only unchanged old detail");
      startConfirmation();q('[data-confirmation-action="confirm"]').click();confirmation.destroy();resolve?.();await new Promise(r=>setTimeout(r,0));expect(!document.querySelector('dialog:modal'),"late resolution after destroy cannot reopen dialog");
      this.openCommand(true);q<HTMLInputElement>('#command-search').dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true}));command.destroy();resolve?.();await Promise.resolve();expect(!document.querySelector('#command-menu'),"destroy invalidates pending command completion");
      modal.reset();
    },
    cleanup(){confirmation.destroy();popover.destroy();command.destroy();drawer.close(false);modal.reset();adapter.destroy();events.destroy();},
  };
}
