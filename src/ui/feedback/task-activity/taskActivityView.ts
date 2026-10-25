import { spinnerView } from "../../primitives/spinner";
import { progressView } from "../../data-display/progress";

export function taskActivityView(): string {
  return `<div class="ui-task-activity" role="status" aria-live="polite" aria-hidden="true" hidden>${spinnerView(16)}<span>正在处理</span>${progressView({label:"任务进度"})}</div>`;
}
