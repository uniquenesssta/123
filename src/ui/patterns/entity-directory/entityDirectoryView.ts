import { escapeHtml } from "../../../components/format";
export interface EntityDirectoryOptions {
  readonly entity: string;
  readonly className?: string;
  /** Complete trusted panel fragments, rendered by their feature owners. */
  readonly content: string;
  readonly inspector?: { readonly content: string; readonly collapsed: boolean };
  readonly selection?: string;
}
/** One directory context; opening an inspector changes columns, never the result identity. */
export function entityDirectoryView(options: EntityDirectoryOptions): string {
  const inspector=options.inspector;
  const mode=inspector ? (inspector.collapsed ? "inspector-collapsed" : "inspector-open") : "";
  return `<section class="entity-browser master-detail-workspace ${escapeHtml(options.className ?? "")} ${mode}" data-entity-browser="${escapeHtml(options.entity)}" data-ui-pattern="entity-directory">${options.content}${inspector?.content ?? ""}</section>${options.selection ?? ""}`;
}
