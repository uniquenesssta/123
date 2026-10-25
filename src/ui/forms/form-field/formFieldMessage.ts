import { fieldHintView } from "../field-hint";
import { fieldErrorView } from "../field-error";

/** Changes owned message references; host descriptions and native values survive. */
export function setFieldMessage(field: HTMLElement, error: string | null, hint = ""): void {
  const id = field.dataset.uiFormField;
  const control = id ? Array.from(field.querySelectorAll<HTMLElement>("[id]")).find(node => node.id === id) : null;
  const message = field.querySelector<HTMLElement>("[data-field-message]");
  if (!id || !control || !message) throw new Error("Invalid form field host");
  const descriptions = (control.getAttribute("aria-describedby") ?? "").split(/\s+/).filter(value => value && value !== `${id}-hint` && value !== `${id}-error`);
  const messageId = error ? `${id}-error` : hint ? `${id}-hint` : "";
  message.innerHTML = error ? fieldErrorView(messageId, error) : hint ? fieldHintView(messageId, hint, control.matches(":disabled")) : "";
  if (messageId) descriptions.push(messageId);
  if (descriptions.length) control.setAttribute("aria-describedby", [...new Set(descriptions)].join(" ")); else control.removeAttribute("aria-describedby");
  if (error) { control.setAttribute("aria-invalid", "true"); control.setAttribute("aria-errormessage", messageId); }
  else { control.removeAttribute("aria-invalid"); control.removeAttribute("aria-errormessage"); }
}
