import { iconSlot } from "../../icons/iconSlot";

export function taskActivityView(): string {
  return `<div class="ui-task-activity" role="status" aria-live="polite" aria-hidden="true" hidden>${iconSlot("spinner", 16)}<span>正在处理</span></div>`;
}
