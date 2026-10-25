import { escapeHtml } from "../../../components/format";
import { primitiveAttributes, primitiveName, type PrimitiveAttributes } from "../controlAttributes";
import { iconButtonView } from "../icon-button";

export interface NumberInputOptions {
  label: string; value?: string; min?: number; max?: number; step?: number | "any"; unit?: string;
  showStepper?: boolean; disabled?: boolean; readOnly?: boolean; required?: boolean; invalid?: boolean; attributes?: PrimitiveAttributes;
}

export function numberInputView(options: NumberInputOptions): string {
  for (const value of [options.min, options.max, options.step]) if (value !== undefined && value !== "any" && !Number.isFinite(value)) throw new Error("Numeric constraints must be finite");
  if (typeof options.step === "number" && options.step <= 0) throw new Error("step must be positive");
  if (options.min !== undefined && options.max !== undefined && options.min > options.max) throw new Error("min exceeds max");
  const limits = Object.entries({ min: options.min, max: options.max, step: options.step }).filter(([, value]) => value !== undefined).map(([key, value]) => `${key}="${value}"`).join(" ");
  const disabled = options.disabled || options.readOnly || options.step === "any";
  return `<span class="ui-number-input" data-ui-primitive="number-input" data-fip-control-scale="32"><input class="ui-number-input-control" type="number" aria-label="${primitiveName(options.label)}" value="${escapeHtml(options.value ?? "")}" ${limits} ${options.disabled ? "disabled" : ""} ${options.readOnly ? "readonly" : ""} ${options.required ? "required" : ""} ${options.invalid ? 'aria-invalid="true"' : ""} ${primitiveAttributes(options.attributes)}>${options.unit ? `<span class="ui-number-unit">${escapeHtml(options.unit)}</span>` : ""}${options.showStepper === false ? "" : `<span class="ui-number-stepper">${iconButtonView({ label: `减少${options.label}`, icon: "minus", disabled, attributes: { "data-number-step": "down" } })}${iconButtonView({ label: `增加${options.label}`, icon: "plus", disabled, attributes: { "data-number-step": "up" } })}</span>`}</span>`;
}
