import { escapeHtml } from "../../../components/format";
export interface EntityDetailOptions {
  readonly title: string;
  readonly context: string;
  readonly actions: string;
  readonly content: string;
}
/** A task remains in its entity context; the host supplies its actual return destination. */
export function entityDetailView(options: EntityDetailOptions): string {
  return `<section class="entity-task-workspace" data-ui-pattern="entity-detail"><header><div><span>${escapeHtml(options.context)}</span><h2>${escapeHtml(options.title)}</h2></div>${options.actions}</header><div class="entity-task-body">${options.content}</div></section>`;
}
