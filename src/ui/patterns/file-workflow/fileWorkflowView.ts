import { escapeHtml } from "../../../components/format";
export interface FileWorkflowOptions {
  readonly className?: string;
  readonly heading: string;
  readonly entry: string;
  /** Only the host's actual preview unlocks review; a selected filename never implies success. */
  readonly preview: string | null;
  readonly empty: string;
}
/** File picker, preview, conflict decisions and commit I/O remain with the workflow owner. */
export function fileWorkflowView(options: FileWorkflowOptions): string {
  return `<section class="panel spreadsheet-panel ${escapeHtml(options.className ?? "")}" data-ui-pattern="file-workflow" data-file-phase="${options.preview === null ? "unselected" : "review"}">${options.heading}${options.entry}${options.preview ?? options.empty}</section>`;
}
