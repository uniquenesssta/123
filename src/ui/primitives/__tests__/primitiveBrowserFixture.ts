import { buttonView, mountButton } from "../button";
import { iconButtonView } from "../icon-button";
import { textInputView, mountTextInput } from "../text-input";
import { numberInputView, mountNumberInput } from "../number-input";
import { textAreaView } from "../text-area";
import { checkboxView, mountCheckbox } from "../checkbox";
import { radioView } from "../radio";
import { switchView } from "../switch";
import { selectView, mountSelect } from "../select";
import { badgeView } from "../badge";
import { dividerView } from "../divider";
import { spinnerView } from "../spinner";
import { skeletonView } from "../skeleton";
import { mountTooltip } from "../tooltip";
import { primitiveAttributes } from "../controlAttributes";

export function installPrimitiveFixture() {
  const root = document.querySelector<HTMLElement>("#primitive-root")!;
  const checks: string[] = [];
  const expect = (condition: unknown, name: string) => { if (!condition) throw new Error(name); checks.push(name); };
  const q = <T extends Element = HTMLElement>(selector: string) => root.querySelector<T>(selector)!;
  const field = (title: string, content: string) => `<section><h2>${title}</h2>${content}</section>`;
  const choices = [{ value: "a", label: "Alpha" }, { value: "b", label: "Beta (disabled)", disabled: true }, { value: "g", label: "Gamma" }, { value: "d", label: "Delta" }];
  const markup = `<form id="primitives-form"><div class="primitive-gallery">${field("Button · 32 / 40 / 48", (["primary", "secondary", "ghost", "danger"] as const).map(variant => `<div class="primitive-row">${([32, 40, 48] as const).map(size => buttonView({ label: "Continue", variant, size })).join("")}</div>`).join("") + `<div class="primitive-row">${buttonView({ label: "Continue", variant: "primary", disabled: true, attributes: { id: "disabled-button" } })}${buttonView({ label: "Loading…", variant: "primary", loading: true, attributes: { id: "busy-button" } })}</div>${buttonView({ label: "Preserve original action width", variant: "primary", attributes: { id: "load-button" } })}`)}
  ${field("Icon Button / Spinner", `<div class="primitive-row">${([32, 40, 48] as const).map(size => iconButtonView({ label: "数据总览", icon: "dashboard", size })).join("")}${iconButtonView({ label: "不可用操作", icon: "close", disabled: true })}</div><div class="primitive-row">${([16, 20, 24] as const).map(size => spinnerView(size, "Default", `加载 ${size}`)).join("")}</div><div class="primitive-row primitive-accent">${([16, 20, 24] as const).map(size => spinnerView(size, "On Accent")).join("")}</div>`)}
  ${field("Text Input / Textarea", textInputView({ label: "名称", value: "Draft 输入", clearable: true, leading: "info", attributes: { id: "text", name: "title" } }) + textInputView({ label: "不可用输入", value: "unchanged", disabled: true, clearable: true, attributes: { id: "text-disabled" } }) + textAreaView({ label: "备注", value: "多行草稿\n保留换行", attributes: { id: "notes", name: "notes" } }) + textAreaView({ label: "错误备注", height: 160, invalid: true, placeholder: "Error state" }))}
  ${field("Number Input", numberInputView({ label: "置信度", value: "0.80", min: 0, max: 1, step: 0.01, unit: "%", attributes: { id: "number", name: "confidence" } }) + numberInputView({ label: "只读数字", value: "2", readOnly: true, attributes: { id: "number-readonly" } }) + numberInputView({ label: "任意步长", value: "1.25", step: "any", attributes: { id: "number-any" } }))}
  ${field("Checkbox / Radio", `<div class="primitive-column">${checkboxView({ label: "复选", attributes: { id: "check", name: "checked" } })}${checkboxView({ label: "部分选择", indeterminate: true, attributes: { id: "mixed" } })}${checkboxView({ label: "禁用已选", checked: true, disabled: true, attributes: { id: "check-disabled" } })}${radioView({ label: "选项 A", name: "group", value: "a", checked: true, attributes: { id: "radio-a" } })}${radioView({ label: "选项 B", name: "group", value: "b", attributes: { id: "radio-b" } })}${radioView({ label: "选项 C（禁用）", name: "group", value: "c", disabled: true, attributes: { id: "radio-disabled" } })}</div>`)}
  ${field("Switch", switchView({ label: "附加当前只读上下文", description: "更改立即生效；长说明会换行并保留原生标签点击。", descriptionId: "switch-help", attributes: { id: "switch", name: "context" } }) + switchView({ label: "当前能力不可用", checked: true, disabled: true, attributes: { id: "switch-disabled" } }))}
  ${field("Select", '<span id="select-help" class="ui-tooltip-description">选择一个可用分类</span>' + selectView({ id: "choice", label: "分类", options: choices, value: "a", attributes: { name: "choice", "aria-describedby": "select-help" } }) + selectView({ id: "choice-disabled", label: "禁用分类", options: choices, value: "a", disabled: true }) + selectView({ id: "choice-required", label: "必选分类", options: choices, required: true, attributes: { name: "required-choice" } }))}
  ${field("Badge / Divider", (["Filled", "Outline"] as const).map(style => `<div class="primitive-row">${(["Neutral", "Info", "Success", "Warning", "Danger"] as const).map(tone => badgeView("Badge", tone, style)).join("")}</div>`).join("") + dividerView() + dividerView(true))}
  ${field("Skeleton · First load only", (["Compact", "Default"] as const).map(density => (["List", "Card", "Table"] as const).map(layout => `<div class="skeleton-sample" data-sample="${layout}-${density}">${skeletonView(layout, density)}</div>`).join("")).join(""))}
  </div><button id="after-controls" type="button">After controls</button></form>`;
  root.innerHTML = markup;
  const button = q<HTMLButtonElement>("#load-button"), buttonHandle = mountButton(button);
  const textRoot = q(".ui-text-input"), textHandle = mountTextInput(textRoot);
  const numberRoot = q(".ui-number-input"), numberHandle = mountNumberInput(numberRoot);
  const mixed = q<HTMLInputElement>("#mixed"), mixedHandle = mountCheckbox(mixed);
  const selectRoot = q(".ui-select"), selectHandle = mountSelect(selectRoot);
  const requiredHandle = mountSelect(q("#choice-required").parentElement!);
  const disabledHandle = mountSelect(q("#choice-disabled").parentElement!);
  const tooltipHandle = mountTooltip(button, "loading-action-description", "保存当前输入，不重复提交");
  const form = q<HTMLFormElement>("form");
  let clicks = 0, changes = 0, inputs = 0;
  const onClick = () => { clicks++; }, onInput = () => { inputs++; }, onChange = () => { changes++; };
  q("#disabled-button").addEventListener("click", onClick); q("#busy-button").addEventListener("click", onClick);
  q("#text").addEventListener("input", onInput); q("#number").addEventListener("change", onChange);
  const geometry = () => {
    for (const node of root.querySelectorAll<HTMLElement>(".ui-button,.ui-icon-button")) {
      if (node.closest(".ui-text-input")) continue;
      expect(Math.abs(node.getBoundingClientRect().height - Number(node.dataset.fipControlScale)) < 0.1, `control height ${node.dataset.fipControlScale}`);
      expect(getComputedStyle(node).boxShadow === "none", "no inherited button shadow");
    }
    expect(q("#text").parentElement!.getBoundingClientRect().height === 40, "Text Field 40px");
    expect(numberRoot.getBoundingClientRect().height === 32, "Number 32px");
    expect(q("#notes").getBoundingClientRect().height === 96, "Textarea 96px");
    expect(getComputedStyle(q("#notes")).fontSize === "14px", "Textarea keeps approved control typography");
    const tokenColor = (token: string) => { const probe = document.createElement("span"); probe.style.color = `var(${token})`; root.append(probe); const color = getComputedStyle(probe).color; probe.remove(); return color; };
    expect(getComputedStyle(q("#notes")).backgroundColor === tokenColor("--fip-color-bg-surface"), "Textarea keeps semantic surface");
    expect(getComputedStyle(q(".ui-text-area[aria-invalid]")).borderTopColor === tokenColor("--fip-color-border-danger"), "Textarea Error border survives old global important rules");
    expect(q(".ui-text-area[data-height='160']").getBoundingClientRect().height === 160, "Textarea 160px");
    for (const size of [16, 20, 24] as const) {
      const spinner = q(`.ui-spinner-${size}`), svg = spinner.querySelector("svg")!;
      expect(spinner.getBoundingClientRect().width === size, `Spinner ${size} slot`);
      expect(Math.abs(parseFloat(getComputedStyle(svg).width) - (size + { 16: 1.25, 20: 1.5, 24: 1.75 }[size]!)) < 0.05, `Spinner ${size} original export root`);
    }
    for (const [sample, expected] of [["List-Compact", 112], ["List-Default", 136], ["Card-Compact", 80], ["Card-Default", 112], ["Table-Compact", 56], ["Table-Default", 72]] as const) expect(q(`[data-sample='${sample}'] .ui-skeleton`).getBoundingClientRect().height === expected, `Skeleton ${sample} geometry`);
    for (const label of root.querySelectorAll<HTMLElement>(".ui-checkbox-mark,.ui-radio-mark")) expect(label.getBoundingClientRect().width === 20 && label.getBoundingClientRect().height === 20, "selection control 20px geometry");
    for (const label of root.querySelectorAll<HTMLElement>(".ui-checkbox-label,.ui-radio-label")) expect(getComputedStyle(label).fontSize === "14px" && getComputedStyle(label).fontWeight === "400", "selection label approved 14px regular typography");
    for (const ring of root.querySelectorAll<SVGElement>(".ui-radio-mark rect[stroke]")) expect(getComputedStyle(ring).strokeOpacity === "1", "radio border alpha applied exactly once");
    for (const label of root.querySelectorAll<HTMLElement>(".ui-switch-control")) expect(label.getBoundingClientRect().width === 40 && label.getBoundingClientRect().height === 24, "Switch 40x24 target");
    for (const control of root.querySelectorAll<HTMLElement>(".ui-switch-control,.ui-radio-mark")) expect([...control.children].filter(node => getComputedStyle(node).display !== "none").length === 1, "exactly one control state export is visible");
    expect(getComputedStyle(q("#switch-disabled").closest("label")!.querySelector("strong")!).color === tokenColor("--fip-color-text-disabled"), "Switch disabled label semantic color");
    for (const avatar of root.querySelectorAll<HTMLElement>(".ui-skeleton-block[data-shape='Avatar']")) expect(avatar.getBoundingClientRect().width === (avatar.dataset.density === "Compact" ? 32 : 40), "Skeleton Avatar keeps square geometry");
    expect(q(".ui-select-trigger").getBoundingClientRect().height === 40, "Select 40px");
    for (const badge of root.querySelectorAll<HTMLElement>(".ui-badge")) expect(badge.getBoundingClientRect().height === 24 && !badge.hasAttribute("tabindex") && !badge.querySelector("button"), "Badge status-only 24px");
    expect(q(".ui-divider").getBoundingClientRect().height === 1, "Divider 1px");
    expect([...root.querySelectorAll(".ui-skeleton")].every(node => node.getAttribute("aria-hidden") === "true" && !node.querySelector("button,input")), "Skeleton is decorative and noninteractive");
    for (const icon of root.querySelectorAll<HTMLElement>(".ui-icon-slot")) {
      const svg = icon.querySelector("svg");
      if (svg) { expect(parseFloat(getComputedStyle(svg).width) === 24.875, "Plus original SVG root is preserved"); expect(Math.abs(svg.getBoundingClientRect().width / icon.getBoundingClientRect().width - 24.875 / 24) < 0.001, "Plus outside stroke matches scaled master geometry"); }
      else expect(getComputedStyle(icon).maskImage !== "none", "real local Icon Slot asset");
    }
    expect(document.documentElement.scrollWidth <= innerWidth, "no horizontal window overflow");
  };
  const localBehavior = () => {
    expect(mixed.indeterminate, "native mixed selection initialized");
    q<HTMLButtonElement>("#disabled-button").click(); q<HTMLButtonElement>("#busy-button").click(); expect(clicks === 0, "disabled/loading click suppressed");
    const originalWidth = button.getBoundingClientRect().width;
    buttonHandle.setLoading(true, "Wait"); expect(button.disabled && button.getAttribute("aria-busy") === "true", "busy is native disabled and aria-busy");
    expect(button.getBoundingClientRect().width >= originalWidth, "loading preserves previous inline width");
    buttonHandle.setLoading(true, "Duplicate"); expect(button.querySelector(".ui-button-label")!.textContent === "Wait", "repeat busy projection is idempotent");
    buttonHandle.setLoading(false); expect(!button.disabled && button.textContent === "Preserve original action width", "loading restores exact action");
    expect(button.title === "保存当前输入，不重复提交" && button.getAttribute("aria-describedby") === "loading-action-description", "native tooltip and AT description");
    expect(q<HTMLInputElement>("#text-disabled").disabled && q<HTMLInputElement>("#number-readonly").readOnly, "disabled and readOnly semantics");
    expect(q<HTMLInputElement>("#number-any").step === "any" && q("#number-any").parentElement!.querySelectorAll(":disabled").length === 2, "any-step disables undefined native stepping");
    expect(q("#switch").getAttribute("role") === "switch" && q("#switch-help").textContent!.length > 0, "Switch role and description");
    expect(q<HTMLTextAreaElement>("#notes").value.includes("\n"), "Textarea native multiline value");
    expect(new FormData(form).get("confidence") === "0.80" && new FormData(form).get("choice") === "a", "native FormData values");
    expect(q("#choice").getAttribute("aria-describedby") === "select-help", "Select description is on its interactive trigger");
    const unsafe = document.createElement("div"); unsafe.innerHTML = textInputView({ label: '"<&', value: '"><img src=x onerror=alert(1)>' });
    expect(!unsafe.querySelector("img") && unsafe.querySelector("input")!.value === '"><img src=x onerror=alert(1)>', "primitive content and attributes are escaped");
    let rejected = false; try { primitiveAttributes({ onclick: "alert(1)" } as never); } catch { rejected = true; } expect(rejected, "active attribute injection rejected");
    rejected = false; try { buttonView({ label: "Wait", variant: "ghost", loading: true }); } catch { rejected = true; } expect(rejected, "unsupported Ghost loading variant rejected");
    button.disabled = true; button.querySelector(".ui-button-label")!.textContent = "Current host label";
    buttonHandle.setLoading(true); buttonHandle.setLoading(false);
    expect(button.disabled && button.textContent === "Current host label", "each busy cycle preserves current host state");
    button.disabled = false; button.querySelector(".ui-button-label")!.textContent = "Preserve original action width";
    const fieldset = document.createElement("fieldset"); fieldset.disabled = true; fieldset.innerHTML = numberInputView({ label: "Disabled ancestor", value: "1", attributes: { id: "ancestor-number" } }) + selectView({ id: "ancestor-choice", label: "Disabled ancestor choice", options: choices }); root.append(fieldset);
    const numberMount = mountNumberInput(fieldset.querySelector<HTMLElement>(".ui-number-input")!), selectMount = mountSelect(fieldset.querySelector<HTMLElement>(".ui-select")!);
    fieldset.querySelector("[data-number-step='up']")!.dispatchEvent(new MouseEvent("click", { bubbles: true })); fieldset.querySelector("[role=combobox]")!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(fieldset.querySelector<HTMLInputElement>("#ancestor-number")!.value === "1" && fieldset.querySelector("[role=combobox]")!.getAttribute("aria-expanded") === "false", "disabled ancestor cannot be bypassed by synthetic parent click"); numberMount.destroy(); selectMount.destroy(); fieldset.remove();
  };
  const cleanup = () => {
    textHandle.destroy(); textHandle.destroy(); numberHandle.destroy(); numberHandle.destroy(); mixedHandle.destroy(); selectHandle.destroy(); requiredHandle.destroy(); disabledHandle.destroy(); tooltipHandle.destroy(); tooltipHandle.destroy(); buttonHandle.destroy();
    const input = q<HTMLInputElement>("#text"); input.value = "after destroy"; q<HTMLButtonElement>("[data-clear-input]").click(); expect(input.value === "after destroy", "destroyed clear listener detached");
    const number = q<HTMLInputElement>("#number"); number.value = "0.5"; q<HTMLButtonElement>("[data-number-step='up']").click(); expect(number.value === "0.5", "destroyed step listener detached");
    q<HTMLButtonElement>("#choice").click(); expect(q("#choice").getAttribute("aria-expanded") === "false", "destroyed selection listener detached");
    expect(!document.getElementById("loading-action-description") && !button.hasAttribute("title") && !button.hasAttribute("aria-describedby"), "tooltip restores attributes and removes owned description");
    const remount = mountNumberInput(numberRoot); q<HTMLButtonElement>("[data-number-step='up']").click(); expect(number.value === "0.51", "remount has one step listener"); remount.destroy();
    q("#disabled-button").removeEventListener("click", onClick); q("#busy-button").removeEventListener("click", onClick); input.removeEventListener("input", onInput); number.removeEventListener("change", onChange);
  };
  return { checks, expect, q, geometry, localBehavior, cleanup, textHandle, numberHandle, selectHandle, form, inputs: () => inputs, changes: () => changes };
}
