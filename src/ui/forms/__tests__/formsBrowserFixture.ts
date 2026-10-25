import { formFieldView, setFieldMessage } from "../form-field";
import { validationSummaryView, mountValidationSummary } from "../validation-summary";
import { confirmationInputView, mountConfirmationInput } from "../confirmation-input";
import { formActionsView } from "../form-actions";
import { textInputView } from "../../primitives/text-input";
import { textAreaView } from "../../primitives/text-area";
import { selectView, mountSelect } from "../../primitives/select";
import { inlineDatabaseSetup } from "../../../components/databaseSetup";
import { ModalController } from "../../../app/modal";

export function installFormsFixture() {
  const root = document.querySelector<HTMLElement>("#forms-root")!;
  const checks: string[] = [];
  const expect = (value: unknown, name: string) => { if (!value) throw new Error(name); checks.push(name); };
  root.innerHTML = `<form id="sample-form"><div class="forms-gallery">
    <section><h2>字段与原生输入</h2><span id="external-help">外部说明</span>${formFieldView({ id: "name", label: "名称", hint: "请输入名称", required: true, describedBy: ["external-help", "external-help"], control: b => textInputView({ ...b, attributes: { ...b.attributes, name: "name" } }) })}${formFieldView({ id: "disabled", label: "不可用字段", hint: "当前不可编辑", disabled: true, control: textInputView })}${formFieldView({ id: "invalid", label: "错误字段", error: "请核对 <名称>", control: textInputView })}${formFieldView({ id: "hidden-label", label: "隐藏但可访问的标签", hiddenLabel: true, control: textInputView })}</section>
    <section><h2>组合控件与摘要</h2>${formFieldView({ id: "notes", label: "说明", hint: "保留多行文本", control: b => textAreaView({ ...b, attributes: { ...b.attributes, name: "notes" } }) })}${formFieldView({ id: "choice", label: "选择", error: "请选择有效项", control: b => selectView({ ...b, options: [{ value: "a", label: "甲" }, { value: "b", label: "乙" }], attributes: { ...b.attributes, id: undefined, name: "choice" } }) })}${validationSummaryView({ id: "summary", issues: [{ controlId: "name", message: "请填写名称" }, { controlId: "choice", message: "请选择有效项" }, { controlId: "disabled", message: "禁用目标" }, { controlId: "missing", message: "不存在的目标" }] })}</section>
    <section><h2>名称确认</h2>${confirmationInputView({ id: "confirm-name", expected: "football_local", label: "输入数据库名称 football_local 以继续", hint: "名称必须完全一致；保留当前输入。", attributes: { name: "confirmation" } })}${formActionsView({ framed: true, cancel: { label: "取消", attributes: { id: "cancel" } }, confirm: { label: "确认永久清空", variant: "danger", disabled: true, attributes: { id: "confirm" } } })}${formActionsView({ framed: true, busy: true, cancel: { label: "取消" }, confirm: { label: "正在提交…", variant: "primary" } })}${validationSummaryView({ id: "empty-summary", issues: [] })}</section>
  </div></form><div id="inline-setup">${inlineDatabaseSetup("连接数据库", "保留当前页", "never expose postgres://secret")}</div><main class="main-content"><div class="page-container"></div><div id="modal-root"></div></main><button id="after-forms">后续内容</button>`;
  const q = <T extends HTMLElement = HTMLElement>(selector: string): T => { const node=root.querySelector<T>(selector); if (!node) throw new Error(selector); return node; };
  const form=q<HTMLFormElement>("#sample-form"), input=q<HTMLInputElement>("#confirm-name"), button=q<HTMLButtonElement>("#confirm");
  let allowed = true;
  const confirmation=mountConfirmationInput(input,button,()=>allowed), summary=mountValidationSummary(q("#summary"),form), select=mountSelect(q(".ui-select"));
  const modal=new ModalController();
  function structure() {
    const ids=Array.from(root.querySelectorAll<HTMLElement>("[id]")).map(n=>n.id);
    expect(ids.length===new Set(ids).size,"document-unique field and message ids");
    for(const c of form.querySelectorAll<HTMLElement>("[aria-labelledby],[aria-describedby],[aria-errormessage]")) for(const a of ["aria-labelledby","aria-describedby","aria-errormessage"]) for(const id of (c.getAttribute(a)??"").split(/\s+/).filter(Boolean)) expect(!!document.getElementById(id),"resolved reference "+a+" "+id);
    const label=q("#name-label"), hint=q("#name-hint"), error=q("#invalid-error");
    expect(getComputedStyle(label).fontSize==="13px"&&getComputedStyle(label).lineHeight==="18px","Figma label 13/18");
    expect(getComputedStyle(hint).fontSize==="12px"&&getComputedStyle(hint).lineHeight==="16px","Figma hint 12/16");
    expect(getComputedStyle(error).fontSize==="12px"&&getComputedStyle(error).lineHeight==="16px","Figma error 12/16");
    expect(getComputedStyle(q('[data-ui-form-field="name"]')).gap==="6px","Figma field gap 6px");
    expect(Math.abs(q<HTMLInputElement>("#name").parentElement!.getBoundingClientRect().height-40)<0.1,"40px Text Input reused");
    expect(q<HTMLInputElement>("#name").required&&q("#name-label").getAttribute("for")==="name","required native field and linked label");
    expect(q<HTMLInputElement>("#disabled").disabled&&getComputedStyle(q("#disabled-label")).color!==getComputedStyle(label).color,"disabled label semantic color");
    expect(q("#invalid-error").textContent!.includes("<名称>")&&!q("#invalid-error").querySelector("名称"),"error string escaped");
    expect(q("#hidden-label-label").getBoundingClientRect().width===1,"hidden label remains accessible");
    expect(q("#empty-summary").hidden,"empty summary absent from layout");
    const busy=root.querySelectorAll<HTMLButtonElement>('.ui-form-actions:has([aria-busy="true"]) button');
    expect(busy.length===2&&Array.from(busy).every(b=>b.disabled),"busy disables cancel and submit");
    expect(button.disabled&&input.autocomplete==="off"&&input.spellcheck===false,"empty confirmation disabled and native field options");
    const loadingOnly=document.createElement("div");loadingOnly.innerHTML=formActionsView({cancel:{label:"取消"},confirm:{label:"处理中",loading:true}});
    expect(Array.from(loadingOnly.querySelectorAll("button")).every(button=>button.disabled),"loading confirm also disables cancel");
    expect(q("#inline-setup").textContent!.includes("上次连接未成功")&&!q("#inline-setup").textContent!.includes("never expose"),"connection error summary does not echo credentials");
    expect(q<HTMLInputElement>("#database-url").type==="password"&&q<HTMLInputElement>("#max-connections").max==="100"&&q<HTMLInputElement>("#connect-timeout").max==="120","production database field contracts retained");
    expect(document.documentElement.scrollWidth<=innerWidth,"no horizontal viewport overflow");
  }
  function messages() {
    const field=q('[data-ui-form-field="name"]'), name=q<HTMLInputElement>("#name");
    name.value="保留的草稿";name.focus();const active=document.activeElement;
    setFieldMessage(field,"错误 <script>alert(1)</script>","提示");
    expect(name.value==="保留的草稿"&&document.activeElement===active,"error preserves native value and focus");
    expect(name.getAttribute("aria-describedby")==="external-help name-error"&&name.getAttribute("aria-errormessage")==="name-error","error owns only its description reference");
    expect(!field.querySelector("script")&&field.textContent!.includes("<script>"),"dynamic error HTML escaped");
    setFieldMessage(field,null,"恢复提示");setFieldMessage(field,null,"恢复提示");
    expect(name.getAttribute("aria-describedby")==="external-help name-hint"&&!name.hasAttribute("aria-invalid")&&!name.hasAttribute("aria-errormessage"),"clear restores hint and deduplicates descriptions");
    setFieldMessage(field,null);expect(name.getAttribute("aria-describedby")==="external-help","empty message preserves external description");
    for(const call of [()=>formFieldView({id:"bad id",label:"名称",control:textInputView}),()=>formFieldView({id:"ok",label:" ",control:textInputView}),()=>confirmationInputView({id:"ok",expected:" ",label:"名称",hint:""})]) { let rejected=false;try{call()}catch{rejected=true}expect(rejected,"invalid public contract rejected"); }
  }
  function gates() {
    input.value=" Football_local ";confirmation.sync();expect(button.disabled,"case-sensitive mismatch");
    input.value=" football_local ";confirmation.sync();expect(!button.disabled,"native trimmed exact name accepted");
    allowed=false;confirmation.sync();expect(button.disabled,"host protection gate cannot be bypassed");allowed=true;
    input.disabled=true;confirmation.sync();expect(button.disabled,"disabled input blocks confirmation");input.disabled=false;
    const fieldset=document.createElement("fieldset"), parent=input.closest<HTMLElement>("[data-confirmation-input]")!, previous=parent.parentElement!, next=parent.nextSibling;previous.insertBefore(fieldset,parent);fieldset.append(parent);fieldset.disabled=true;confirmation.sync();expect(button.disabled,"disabled ancestor blocks confirmation");fieldset.disabled=false;previous.insertBefore(parent,next);fieldset.remove();
    button.setAttribute("aria-busy","true");confirmation.sync();expect(button.disabled,"busy blocks repeated confirmation");button.removeAttribute("aria-busy");
    input.dispatchEvent(new CompositionEvent("compositionstart"));expect(button.disabled,"IME composition cannot enable command");input.dispatchEvent(new CompositionEvent("compositionend"));expect(!button.disabled,"compositionend recomputes gate");
    input.value="错误";input.dispatchEvent(new FocusEvent("blur"));expect(input.getAttribute("aria-invalid")==="true"&&q("#confirm-name-error").textContent!.includes("football_local"),"blur shows expected name and owned error");
    const style=document.documentElement.dataset.theme;input.value="输入保留";input.focus();document.documentElement.dataset.theme=style==="light"?"dark":"light";expect(input.value==="输入保留"&&document.activeElement===input,"theme retains draft and focus");document.documentElement.dataset.theme=style;
  }
  function workspace() {
    const content=confirmationInputView({id:"database-reset-confirmation",confirmButtonId:"workspace-confirm",expected:"real_name",label:"实际名称",hint:"精确输入"});
    const footer=formActionsView({confirm:{label:"确认",variant:"danger",disabled:true,attributes:{"data-action":"execute-database-reset",id:"workspace-confirm"}}});
    modal.showHtml("确认","保持工作区 region",'<div class="database-reset-confirmation">'+content+'</div>',footer);
    const old=q<HTMLInputElement>("#database-reset-confirmation"), oldButton=q<HTMLButtonElement>('[data-action="execute-database-reset"]');
    expect(getComputedStyle(q("#database-reset-confirmation-label")).fontWeight==="500","workspace label uses Forms typography");
    old.value="real_name";old.dispatchEvent(new Event("input",{bubbles:true}));expect(!oldButton.disabled,"production workspace mounts shared confirmation gate");
    modal.showHtml("详情","测试","<p>第二页</p>");oldButton.disabled=true;old.dispatchEvent(new Event("input",{bubbles:true}));expect(oldButton.disabled,"workspace replacement aborts old input listeners");
    modal.back();expect(q<HTMLInputElement>("#database-reset-confirmation").value==="real_name"&&!q<HTMLButtonElement>('[data-action="execute-database-reset"]').disabled,"workspace history restores draft then recomputes gate");
    expect(q(".workspace-detail-page").getAttribute("role")==="region","workspace semantics unchanged");modal.close();
  }
  function cleanup() {
    form.reset();confirmation.destroy();confirmation.destroy();summary.destroy();select.destroy();input.value="football_local";button.disabled=true;input.dispatchEvent(new Event("input",{bubbles:true}));expect(button.disabled,"destroy removes confirmation input listeners and pending reset projection");
    const remount=mountConfirmationInput(input,button);expect(!button.disabled,"remount evaluates native value once");remount.destroy();
  }
  return { q,form,input,button,checks,expect,structure,messages,gates,workspace,cleanup,summary,confirmation };
}
