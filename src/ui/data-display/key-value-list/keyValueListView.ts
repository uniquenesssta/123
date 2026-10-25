import { escapeHtml } from "../../../components/format";
export interface KeyValueItem {
  label: string; value?: string | number | boolean | null;
  /** Trusted application-rendered content; never unescaped external HTML. */
  renderValue?: () => string;
}
export function keyValueListView(items: readonly KeyValueItem[]): string {
  return `<dl class="ui-key-value-list" data-ui-display="key-value-list">${items.map(item => `<div><dt>${escapeHtml(item.label)}</dt><dd>${item.renderValue ? item.renderValue() : escapeHtml(item.value ?? "—")}</dd></div>`).join("")}</dl>`;
}
