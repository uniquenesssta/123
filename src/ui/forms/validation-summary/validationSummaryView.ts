import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../icons/iconSlot";

export interface ValidationIssue { controlId: string; message: string }
export interface ValidationSummaryOptions { id: string; title?: string; issues: readonly ValidationIssue[] }

/** Host supplies results; this component never derives domain validation rules. */
export function validationSummaryView(o: ValidationSummaryOptions): string {
  if (!/^[A-Za-z][\w:.-]*$/.test(o.id)) throw new Error("A summary requires a stable document-unique id");
  return `<section class="ui-validation-summary" id="${escapeHtml(o.id)}" data-validation-summary tabindex="-1" aria-labelledby="${escapeHtml(o.id)}-title" ${o.issues.length ? "" : "hidden"}>${iconSlot("alert-circle", 16)}<div><h3 id="${escapeHtml(o.id)}-title">${escapeHtml(o.title ?? "请检查以下内容")}</h3><ul>${o.issues.map(issue => `<li><a href="#${encodeURIComponent(issue.controlId)}" data-validation-target="${escapeHtml(issue.controlId)}">${escapeHtml(issue.message)}</a></li>`).join("")}</ul></div></section>`;
}
